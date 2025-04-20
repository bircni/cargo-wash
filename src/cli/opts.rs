use clap::Parser;
use parking_lot::RwLock;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::{data::Project, utils};

/// Represents the command line options
#[derive(Parser, Default, Clone)]
pub struct Opts {
    /// Run the program without making any changes
    #[clap(short, long)]
    pub dry_run: bool,
    /// Path to a directory
    #[clap(long, short)]
    pub path: Option<PathBuf>,
    /// Custom additional build folder which should be used for size calculation and cleaning
    /// default is `target`
    /// (e.g. `dist`, `build`, `target`, etc.)
    #[clap(long, short)]
    pub build_folder: Option<String>,
    /// Exclude the provided folder from the size calculation and cleaning
    /// default is `None` - you can specify multiple folders separated by `,`
    #[clap(long, short)]
    pub exclude: Option<String>,
}

impl Opts {
    pub fn check_args(&self) -> anyhow::Result<(Vec<Project>, bool)> {
        let projects: Arc<RwLock<Vec<Project>>> = Arc::new(RwLock::new(vec![]));
        let path = utils::sanitize_path_input(self.path.clone())?;

        if path.is_dir() {
            utils::check_project(&path, self.build_folder.as_ref(), self.exclude.as_ref()).map(
                |p_opt| {
                    if let Some(project) = p_opt {
                        projects.write().push(project);
                    }
                },
            )?;

            match fs::read_dir(&path) {
                Ok(entries) => {
                    entries.flatten().par_bridge().for_each(|entry| {
                        let pathbuf = entry.path();
                        if pathbuf.is_dir() {
                            match utils::check_project(
                                &pathbuf,
                                self.build_folder.as_ref(),
                                self.exclude.as_ref(),
                            ) {
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

        Ok((projects.read().to_vec(), self.dry_run))
    }
}
