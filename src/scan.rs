//! Discovery of Rust projects and measurement of their `target` folders.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use anyhow::Context as _;
use indicatif::ParallelProgressIterator as _;
use rayon::iter::{
    IntoParallelRefIterator as _, IntoParallelRefMutIterator as _, ParallelIterator as _,
};

use crate::{cli::Options, project::Project, report, size::Size};

/// Find the Rust projects in the search directory and its direct children,
/// sorted by name.
pub fn discover(options: &Options) -> anyhow::Result<Vec<Project>> {
    let root = resolve_root(&options.path)?;
    let excluded = parse_excludes(options.exclude.as_deref());

    let mut candidates = child_dirs(&root);
    candidates.push(root);

    let mut projects: Vec<Project> = candidates
        .par_iter()
        .filter_map(|dir| project_at(dir, &excluded))
        .collect();
    projects.sort_by_cached_key(|project| (project.name.to_lowercase(), project.path.clone()));
    Ok(projects)
}

/// Fill in the size of the `target` folder of every project.
pub fn measure(projects: &mut [Project]) {
    let progress = report::progress_bar(projects.len(), "Measuring target folders...");
    projects
        .par_iter_mut()
        .progress_with(progress.clone())
        .for_each(|project| project.size = dir_size(&project.target_dir()));
    progress.finish_and_clear();
}

/// Resolve the directory the search starts from to an absolute path.
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

/// Split the `--exclude` value into the folder names it lists.
fn parse_excludes(exclude: Option<&str>) -> Vec<&str> {
    exclude
        .into_iter()
        .flat_map(|list| list.split(','))
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect()
}

fn child_dirs(dir: &Path) -> Vec<PathBuf> {
    match fs::read_dir(dir) {
        Ok(entries) => entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect(),
        Err(error) => {
            log::warn!("Could not read {}: {error}", dir.display());
            vec![]
        }
    }
}

fn project_at(dir: &Path, excluded: &[&str]) -> Option<Project> {
    let project = Project::new(dir);
    if excluded.contains(&project.name.as_str()) {
        log::debug!("Excluding folder: {}", project.name);
        return None;
    }
    if !dir.join("Cargo.toml").is_file() {
        log::debug!("Skipping non-Rust project: {}", project.name);
        return None;
    }
    Some(project)
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

    use super::{dir_size, discover, measure, parse_excludes, resolve_root};
    use crate::cli::Options;

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

    fn names(options: &Options) -> Vec<String> {
        discover(options)
            .unwrap()
            .into_iter()
            .map(|project| project.name)
            .collect()
    }

    fn options(path: &Path, exclude: Option<&str>) -> Options {
        Options {
            path: path.to_path_buf(),
            exclude: exclude.map(str::to_owned),
            ..Default::default()
        }
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
    fn splits_excludes_on_commas() {
        assert_eq!(parse_excludes(None), [""; 0]);
        assert_eq!(parse_excludes(Some("a, b ,,c")), ["a", "b", "c"]);
    }

    #[test]
    fn finds_the_root_and_its_children_sorted_by_name() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let root = create_project(tmp_dir.path(), "root", 0);
        create_project(&root, "Zeta", 0);
        create_project(&root, "alpha", 0);
        create_project(&root.join("nested"), "too-deep", 0);
        fs::create_dir_all(root.join("not-rust")).unwrap();

        assert_eq!(names(&options(&root, None)), ["alpha", "root", "Zeta"]);
    }

    #[test]
    fn exclude_matches_folder_names_exactly() {
        let tmp_dir = tempfile::tempdir().unwrap();
        create_project(tmp_dir.path(), "alpha", 0);
        create_project(tmp_dir.path(), "beta", 0);
        let excluding = |exclude| names(&options(tmp_dir.path(), Some(exclude)));

        // A substring of a project name (or of a parent directory) must not exclude anything.
        assert_eq!(excluding("a"), ["alpha", "beta"]);
        assert_eq!(excluding("alpha"), ["beta"]);
        assert_eq!(excluding("alpha, beta"), [""; 0]);
    }

    #[test]
    fn measures_target_folders() {
        let tmp_dir = tempfile::tempdir().unwrap();
        create_project(tmp_dir.path(), "built", 2048);
        create_project(tmp_dir.path(), "fresh", 0);

        let mut projects = discover(&options(tmp_dir.path(), None)).unwrap();
        assert_eq!(projects.iter().map(|p| p.size.bytes()).sum::<u64>(), 0);
        measure(&mut projects);
        let sizes: Vec<u64> = projects.iter().map(|p| p.size.bytes()).collect();
        assert_eq!(sizes, [2048, 0]);
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
