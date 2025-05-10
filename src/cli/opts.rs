use clap::Parser;
use parking_lot::RwLock;
use rayon::iter::{ParallelBridge as _, ParallelIterator as _};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::{data::Project, logic};

/// Represents the command line options
#[derive(Parser, Default, Clone)]
pub struct Opts {
    /// Path to the directory from which to start the search for Rust projects
    /// default is `.`
    #[clap(long, short)]
    pub path: Option<PathBuf>,
    /// Exclude the provided folder from the size calculation and cleaning
    /// default is `None` - you can specify multiple folders separated by `,`
    #[clap(long, short)]
    pub exclude: Option<String>,
}

impl Opts {
    pub fn check_args(&self) -> anyhow::Result<Vec<Project>> {
        let projects: Arc<RwLock<Vec<Project>>> = Arc::new(RwLock::new(vec![]));
        let path = logic::sanitize_path_input(self.path.clone())?;

        if path.is_dir() {
            logic::check_project(&path, self.exclude.as_ref()).map(|p_opt| {
                if let Some(project) = p_opt {
                    projects.write().push(project);
                }
            })?;

            match fs::read_dir(&path) {
                Ok(entries) => {
                    entries.flatten().par_bridge().for_each(|entry| {
                        let pathbuf = entry.path();
                        if pathbuf.is_dir() {
                            match logic::check_project(&pathbuf, self.exclude.as_ref()) {
                                Ok(p_opt) => {
                                    if let Some(project) = p_opt {
                                        projects.write().push(project);
                                    }
                                }
                                Err(error) => log::warn!("Error checking project: {error}"),
                            }
                        }
                    });
                }
                Err(error) => log::warn!("Error reading directory: {error}"),
            }
        } else {
            anyhow::bail!("The provided path is not a directory.");
        }

        Ok(projects.read().to_vec())
    }
}
