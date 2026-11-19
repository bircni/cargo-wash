use std::path::{Path, PathBuf};

use crate::size::Size;

/// A Rust project found on disk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Project {
    /// Name of the folder containing the project
    pub name: String,
    /// Absolute path to the project
    pub path: PathBuf,
    /// Size of the `target` folder, zero until it has been measured
    pub size: Size,
}

impl Project {
    pub fn new(path: &Path) -> Self {
        let name = path.file_name().map_or_else(
            || path.to_string_lossy().into_owned(),
            |name| name.to_string_lossy().into_owned(),
        );
        Self {
            name,
            path: path.to_path_buf(),
            size: Size::default(),
        }
    }

    pub fn target_dir(&self) -> PathBuf {
        self.path.join("target")
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::Project;

    #[test]
    fn name_is_the_folder_name() {
        assert_eq!(Project::new(Path::new("foo/bar/baz")).name, "baz");
    }

    #[cfg(unix)]
    #[test]
    fn name_falls_back_to_the_path() {
        assert_eq!(Project::new(Path::new("/")).name, "/");
    }
}
