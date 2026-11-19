use std::path::{Path, PathBuf};

use crate::size::Size;

/// A Rust project found on disk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Project {
    /// Name of the folder containing the project
    pub name: String,
    /// Absolute path to the project
    pub path: PathBuf,
    /// Folder cargo writes the build artifacts of the project to
    pub target_dir: PathBuf,
    /// Size of the target folder, zero until it has been measured
    pub size: Size,
}

impl Project {
    /// Describe the project at `path`.
    ///
    /// `target_dir` is the value of `CARGO_TARGET_DIR` if it is set, a
    /// relative value is resolved against the project like cargo does.
    pub fn new(path: &Path, target_dir: Option<&Path>) -> Self {
        let name = path.file_name().map_or_else(
            || path.to_string_lossy().into_owned(),
            |name| name.to_string_lossy().into_owned(),
        );
        Self {
            name,
            path: path.to_path_buf(),
            target_dir: path.join(target_dir.unwrap_or_else(|| Path::new("target"))),
            size: Size::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::Project;

    #[test]
    fn name_is_the_folder_name() {
        assert_eq!(Project::new(Path::new("foo/bar/baz"), None).name, "baz");
    }

    #[cfg(unix)]
    #[test]
    fn name_falls_back_to_the_path() {
        assert_eq!(Project::new(Path::new("/"), None).name, "/");
    }

    #[test]
    fn target_dir_follows_the_cargo_override() {
        let path = Path::new("/code/foo");
        let target_dir = |value: Option<&str>| Project::new(path, value.map(Path::new)).target_dir;
        assert_eq!(target_dir(None), path.join("target"));
        assert_eq!(target_dir(Some("out")), path.join("out"));
        assert_eq!(target_dir(Some("/shared")), Path::new("/shared"));
    }
}
