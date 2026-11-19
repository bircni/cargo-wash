//! Discovery of Rust projects and measurement of their `target` folders.

use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use anyhow::Context as _;
use glob::Pattern;
use indicatif::ParallelProgressIterator as _;
use rayon::iter::{
    IntoParallelRefIterator as _, IntoParallelRefMutIterator as _, ParallelIterator as _,
};

use crate::{cli::ScanArgs, project::Project, report, size::Size};

/// Folders that never contain projects worth searching for
const SKIPPED_DIRS: [&str; 2] = ["target", "node_modules"];

/// Find the Rust projects below the search paths, sorted by name.
pub fn discover(args: &ScanArgs) -> anyhow::Result<Vec<Project>> {
    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let projects = discover_with(args, target_dir.as_deref())?;

    if let Some(target_dir) = target_dir.filter(|dir| dir.is_absolute() && projects.len() > 1) {
        log::warn!(
            "CARGO_TARGET_DIR is set, all projects share the target folder {}",
            target_dir.display()
        );
    }
    Ok(projects)
}

/// [`discover`] with an explicit value for `CARGO_TARGET_DIR`
fn discover_with(args: &ScanArgs, target_dir: Option<&Path>) -> anyhow::Result<Vec<Project>> {
    let mut projects = Vec::new();
    for path in &args.paths {
        let root = resolve_root(path)?;
        let walk = Walk {
            root: &root,
            exclude: &args.exclude,
            target_dir,
        };
        projects.extend(walk.projects_in(&root, args.depth));
    }

    projects.sort_by_cached_key(|project| (project.name.to_lowercase(), project.path.clone()));
    // Overlapping search paths find the same project more than once.
    projects.dedup_by(|a, b| a.path == b.path);
    Ok(projects)
}

/// Fill in the size of the target folder of every project.
pub fn measure(projects: &mut [Project]) {
    let progress = report::progress_bar(projects.len(), "Measuring target folders...");
    projects
        .par_iter_mut()
        .progress_with(progress.clone())
        .for_each(|project| project.size = dir_size(&project.target_dir));
    progress.finish_and_clear();
}

/// Resolve a search path to an absolute directory.
fn resolve_root(dir: &Path) -> anyhow::Result<PathBuf> {
    let root = dunce::canonicalize(dir)
        .with_context(|| format!("Could not resolve path: {}", dir.display()))?;
    anyhow::ensure!(
        root.is_dir(),
        "The provided path is not a directory: {}",
        root.display()
    );
    Ok(root)
}

/// The search below a single search path
struct Walk<'a> {
    root: &'a Path,
    exclude: &'a [Pattern],
    target_dir: Option<&'a Path>,
}

impl Walk<'_> {
    /// Collect the projects in `dir` and up to `depth` levels below it.
    ///
    /// A project ends the search of its folder: nested manifests are members
    /// of its workspace and share the target folder.
    fn projects_in(&self, dir: &Path, depth: usize) -> Vec<Project> {
        if self.is_excluded(dir) {
            log::debug!("Excluding folder: {}", dir.display());
            return vec![];
        }
        if dir.join("Cargo.toml").is_file() {
            return vec![Project::new(dir, self.target_dir)];
        }
        if depth == 0 {
            return vec![];
        }
        child_dirs(dir)
            .par_iter()
            .flat_map_iter(|child| self.projects_in(child, depth - 1))
            .collect()
    }

    /// Whether the name of `dir` or its path relative to the search path
    /// matches one of the exclude patterns
    fn is_excluded(&self, dir: &Path) -> bool {
        let name = dir.file_name().map(|name| name.to_string_lossy());
        let relative = dir.strip_prefix(self.root).ok().map(|relative| {
            relative
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/")
        });
        self.exclude.iter().any(|pattern| {
            [name.as_deref(), relative.as_deref()]
                .into_iter()
                .flatten()
                .any(|candidate| pattern.matches(candidate))
        })
    }
}

/// The folders in `dir` that are searched for projects.
///
/// Symlinks are not followed so that the search cannot leave the search path
/// or run into a cycle.
fn child_dirs(dir: &Path) -> Vec<PathBuf> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            log::warn!("Could not read {}: {error}", dir.display());
            return vec![];
        }
    };
    entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|file_type| file_type.is_dir()))
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            !name.starts_with('.') && !SKIPPED_DIRS.contains(&name.as_ref())
        })
        .map(|entry| entry.path())
        .collect()
}

/// Recursively calculate the size of a folder.
///
/// Symlinks are not followed, and entries that cannot be read are skipped.
fn dir_size(dir: &Path) -> Size {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            if error.kind() != io::ErrorKind::NotFound {
                log::warn!("Could not read {}: {error}", dir.display());
            }
            return Size::default();
        }
    };

    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            Some(if metadata.is_dir() {
                dir_size(&entry.path())
            } else {
                Size(metadata.len())
            })
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use glob::Pattern;

    use super::{dir_size, discover_with, measure, resolve_root};
    use crate::cli::ScanArgs;

    /// Create a folder with a manifest and a `target` folder of `target_bytes` bytes.
    fn create_project(parent: &Path, name: &str, target_bytes: usize) -> PathBuf {
        let project = parent.join(name);
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("Cargo.toml"), "").unwrap();
        if target_bytes > 0 {
            fs::create_dir_all(project.join("target").join("debug")).unwrap();
            fs::write(
                project.join("target").join("debug").join("artifact"),
                vec![0; target_bytes],
            )
            .unwrap();
        }
        project
    }

    fn scan(paths: &[&Path], depth: usize, exclude: &[&str]) -> ScanArgs {
        ScanArgs {
            paths: paths.iter().map(|path| path.to_path_buf()).collect(),
            depth,
            exclude: exclude
                .iter()
                .map(|pattern| Pattern::new(pattern).unwrap())
                .collect(),
        }
    }

    fn names(args: &ScanArgs) -> Vec<String> {
        discover_with(args, None)
            .unwrap()
            .into_iter()
            .map(|project| project.name)
            .collect()
    }

    #[test]
    fn resolves_relative_roots() {
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(resolve_root(Path::new(".")).unwrap(), cwd);
        assert_eq!(
            resolve_root(Path::new("..")).unwrap(),
            cwd.parent().unwrap()
        );
        resolve_root(Path::new("not_existing")).unwrap_err();
        resolve_root(Path::new("Cargo.toml")).unwrap_err();
    }

    #[cfg(unix)]
    #[test]
    fn keeps_the_filesystem_root() {
        assert_eq!(
            resolve_root(Path::new("/")).unwrap(),
            PathBuf::from("/"),
            "the filesystem root must not be replaced by the current directory"
        );
    }

    #[test]
    fn finds_projects_sorted_by_name() {
        let tmp_dir = tempfile::tempdir().unwrap();
        create_project(tmp_dir.path(), "Zeta", 0);
        create_project(tmp_dir.path(), "alpha", 0);
        fs::create_dir_all(tmp_dir.path().join("not-rust")).unwrap();

        assert_eq!(names(&scan(&[tmp_dir.path()], 1, &[])), ["alpha", "Zeta"]);
    }

    #[test]
    fn depth_limits_the_search() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let root = tmp_dir.path();
        create_project(root, "one", 0);
        create_project(&root.join("group"), "two", 0);
        create_project(&root.join("group").join("sub"), "three", 0);
        let with_depth = |depth| names(&scan(&[root], depth, &[]));

        assert_eq!(with_depth(0), [""; 0]);
        assert_eq!(with_depth(1), ["one"]);
        assert_eq!(with_depth(2), ["one", "two"]);
        assert_eq!(with_depth(3), ["one", "three", "two"]);
        assert_eq!(with_depth(usize::MAX), ["one", "three", "two"]);
    }

    #[test]
    fn depth_zero_checks_only_the_path_itself() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let project = create_project(tmp_dir.path(), "project", 0);

        assert_eq!(names(&scan(&[&project], 0, &[])), ["project"]);
    }

    #[test]
    fn does_not_search_inside_projects_or_skipped_folders() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let root = tmp_dir.path();
        let workspace = create_project(root, "workspace", 0);
        create_project(&workspace, "member", 0);
        create_project(&root.join(".hidden"), "hidden", 0);
        create_project(&root.join("node_modules"), "dependency", 0);
        create_project(&root.join("target"), "artifact", 0);

        assert_eq!(names(&scan(&[root], 5, &[])), ["workspace"]);
    }

    #[cfg(unix)]
    #[test]
    fn does_not_search_symlinked_folders() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let root = tmp_dir.path().join("root");
        let elsewhere = create_project(tmp_dir.path(), "elsewhere", 0);
        create_project(&root, "project", 0);
        std::os::unix::fs::symlink(&elsewhere, root.join("link")).unwrap();
        std::os::unix::fs::symlink(&root, root.join("cycle")).unwrap();

        assert_eq!(names(&scan(&[&root], 5, &[])), ["project"]);
    }

    #[test]
    fn searches_all_paths_without_duplicates() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let first = tmp_dir.path().join("first");
        let second = tmp_dir.path().join("second");
        create_project(&first, "a", 0);
        create_project(&second, "b", 0);

        let args = scan(&[&first, &second, tmp_dir.path()], 2, &[]);
        assert_eq!(names(&args), ["a", "b"]);
        discover_with(&scan(&[&first, Path::new("not_existing")], 1, &[]), None).unwrap_err();
    }

    #[test]
    fn exclude_matches_names_and_relative_paths() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let root = tmp_dir.path();
        create_project(root, "alpha", 0);
        create_project(root, "beta", 0);
        create_project(&root.join("vendor"), "alpha-fork", 0);
        let excluding = |exclude: &[&str]| names(&scan(&[root], 2, exclude));

        // A substring of a project name (or of a parent directory) must not exclude anything.
        assert_eq!(excluding(&["a"]), ["alpha", "alpha-fork", "beta"]);
        assert_eq!(excluding(&["alpha"]), ["alpha-fork", "beta"]);
        assert_eq!(excluding(&["alpha*"]), ["beta"]);
        assert_eq!(excluding(&["alpha", "beta"]), ["alpha-fork"]);
        assert_eq!(excluding(&["vendor"]), ["alpha", "beta"]);
        assert_eq!(excluding(&["vendor/alpha-*"]), ["alpha", "beta"]);
        assert_eq!(excluding(&["*"]), [""; 0]);
    }

    #[test]
    fn measures_target_folders() {
        let tmp_dir = tempfile::tempdir().unwrap();
        create_project(tmp_dir.path(), "built", 2048);
        create_project(tmp_dir.path(), "fresh", 0);

        let mut projects = discover_with(&scan(&[tmp_dir.path()], 1, &[]), None).unwrap();
        assert_eq!(projects.iter().map(|p| p.size.bytes()).sum::<u64>(), 0);
        measure(&mut projects);
        let sizes: Vec<u64> = projects.iter().map(|p| p.size.bytes()).collect();
        assert_eq!(sizes, [2048, 0]);
    }

    #[test]
    fn measures_the_overridden_target_folder() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let project = create_project(tmp_dir.path(), "project", 2048);
        fs::create_dir_all(project.join("out")).unwrap();
        fs::write(project.join("out").join("artifact"), vec![0; 512]).unwrap();

        let args = scan(&[tmp_dir.path()], 1, &[]);
        let mut projects = discover_with(&args, Some(Path::new("out"))).unwrap();
        measure(&mut projects);
        assert_eq!(
            projects[0].target_dir,
            project.canonicalize().unwrap().join("out")
        );
        assert_eq!(projects[0].size.bytes(), 512);
    }

    #[test]
    fn missing_folders_are_empty() {
        assert_eq!(dir_size(Path::new("not_existing")).bytes(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn does_not_follow_symlinks() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let outside = tmp_dir.path().join("outside");
        let inside = tmp_dir.path().join("inside");
        fs::create_dir_all(&outside).unwrap();
        fs::create_dir_all(&inside).unwrap();
        fs::write(outside.join("big"), vec![0; 4096]).unwrap();
        fs::write(inside.join("small"), vec![0; 10]).unwrap();
        std::os::unix::fs::symlink(&outside, inside.join("dir_link")).unwrap();
        std::os::unix::fs::symlink(&inside, inside.join("loop")).unwrap();

        let size = dir_size(&inside).bytes();
        assert!(size < 4096, "symlinked content was counted: {size}");
        assert!(size >= 10, "regular files must be counted: {size}");
    }
}
