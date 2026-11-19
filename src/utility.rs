use std::{
    fs, io,
    path::{Path, PathBuf},
};

use anyhow::Context as _;

use crate::{data::Project, extensions::PathBufExt as _};

/// Resolve the directory the search starts from to an absolute path.
pub fn resolve_root(dir: &Path) -> anyhow::Result<PathBuf> {
    let root = dunce::canonicalize(dir)
        .with_context(|| format!("Could not resolve path: {}", dir.display()))?;
    anyhow::ensure!(root.is_dir(), "The provided path is not a directory.");
    Ok(root)
}

/// Split the `--exclude` value into the folder names it lists.
pub fn parse_excludes(exclude: Option<&String>) -> Vec<String> {
    exclude
        .into_iter()
        .flat_map(|list| list.split(','))
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Recursively calculate the size of a folder.
///
/// Symlinks are not followed, and entries that cannot be read are skipped.
pub fn get_folder_size<P: AsRef<Path>>(dir: P) -> u64 {
    let dir = dir.as_ref();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            if error.kind() != io::ErrorKind::NotFound {
                log::warn!("Could not read {}: {error}", dir.display());
            }
            return 0;
        }
    };

    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            Some(if metadata.is_dir() {
                get_folder_size(entry.path())
            } else {
                metadata.len()
            })
        })
        .sum()
}

pub fn get_project(path: &PathBuf, excluded: &[String]) -> anyhow::Result<Option<Project>> {
    let name = &path.get_name()?;

    if excluded.contains(name) {
        log::debug!("Excluding folder: {name}");
        return Ok(None);
    }

    // Skip non Rust projects
    if !path.join("Cargo.toml").exists() {
        log::debug!("Skipping non-Rust project: {name}");
        return Ok(None);
    }

    let size = get_folder_size(path.join("target"));
    Ok(Some(Project::new(name, path, size)))
}
