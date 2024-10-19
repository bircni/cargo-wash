use std::path::PathBuf;

use anyhow::Context;

pub trait PathBufExt {
    /// Get the name of the file or directory
    fn get_name(&self) -> anyhow::Result<String>;
}

impl PathBufExt for PathBuf {
    fn get_name(&self) -> anyhow::Result<String> {
        Ok(self
            .file_name()
            .context(format!("Could not find filename for path: {self:?}"))?
            .to_string_lossy()
            .to_string())
    }
}
