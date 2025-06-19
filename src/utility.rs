use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::Context as _;

use crate::{data::Project, extensions::PathBufExt as _};

pub fn sanitize_path_input(dir: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    let path = dir.unwrap_or_else(|| PathBuf::from("."));
    if path == Path::new("/") || path == Path::new(".") {
        env::current_dir().context("Failed to get current directory")
    } else if path == Path::new("..") {
        env::current_dir()
            .context("Failed to get current directory")?
            .parent()
            .context("Failed to get parent directory")
            .map(Path::to_path_buf)
            .context("Failed to convert parent directory to path")
    } else {
        Ok(path)
    }
}

/// Recursively calculate the size of a folder
pub fn get_folder_size<P: AsRef<Path>>(dir: P) -> anyhow::Result<u64> {
    if !dir.as_ref().exists() {
        return Ok(0);
    }
    let mut total_size = 0;

    for entry_res in fs::read_dir(dir)? {
        let entry = entry_res?;
        let path = entry.path();
        let metadata = fs::metadata(&path)?;

        if metadata.is_dir() {
            total_size += get_folder_size(&path)?;
        } else {
            total_size += metadata.len();
        }
    }

    Ok(total_size)
}

pub fn check_project(
    path: &PathBuf,
    exclude_folder: Option<&String>,
) -> anyhow::Result<Option<Project>> {
    if let Some(exclude) = exclude_folder {
        if path.to_string_lossy().contains(exclude) {
            log::debug!("Excluding folder: {exclude}");
            return Ok(None);
        }
        log::debug!("Checking folder: {}", path.to_string_lossy());
    }

    let name = &path.get_name()?;
    let size = get_folder_size(path.join("target"))?;

    if size > 0 {
        return Ok(Some(Project::new(name, path, size)));
    }

    Ok(None)
}
