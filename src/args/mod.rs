use std::path::{Path, PathBuf};

use anyhow::Context;

pub mod commands;
mod opts;

pub fn clean_path(dir: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    let path = dir.unwrap_or_else(|| PathBuf::from("."));
    if path == Path::new("/") || path == Path::new(".") {
        std::env::current_dir().context("Failed to get current directory")
    } else if path == Path::new("..") {
        std::env::current_dir()
            .context("Failed to get current directory")?
            .parent()
            .context("Failed to get parent directory")
            .map(std::path::Path::to_path_buf)
            .context("Failed to convert parent directory to path")
    } else {
        Ok(path)
    }
}
