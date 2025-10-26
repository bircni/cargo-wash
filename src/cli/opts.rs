use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use parking_lot::RwLock;
use rayon::iter::{ParallelBridge as _, ParallelIterator as _};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::{data::Project, utility};

/// Represents general command line options.
#[derive(Parser, Default, Clone)]
pub struct Options {
    /// Path to the directory from which to start the search for Rust projects
    #[clap(short, long, default_value = ".")]
    pub path: PathBuf,

    /// Exclude the provided folder from the size calculation and cleaning
    #[clap(long, short)]
    pub exclude: Option<String>,

    /// Enable parallel processing of projects
    /// ATTENTION: This may lead to high CPU usage!
    #[clap(long, default_value_t = false, verbatim_doc_comment)]
    pub parallel: bool,

    /// Additional arguments to pass to the cargo command
    #[clap(last = true)]
    pub args: Vec<String>,
}

/// A trait defining common fields shared between options structs.
pub trait CommonOptions {
    fn path(&self) -> &PathBuf;
    fn exclude(&self) -> Option<&String>;
}

impl CommonOptions for Options {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn exclude(&self) -> Option<&String> {
        self.exclude.as_ref()
    }
}

/// A trait for validating options and collecting Rust projects.
pub trait OptionsTrait {
    fn check_args(&self) -> anyhow::Result<Vec<Project>>;
}

impl<T> OptionsTrait for T
where
    T: CommonOptions + Send + Sync,
{
    fn check_args(&self) -> anyhow::Result<Vec<Project>> {
        let projects: Arc<RwLock<Vec<Project>>> = Arc::new(RwLock::new(vec![]));
        let path = utility::sanitize_path_input(self.path())?;

        if path.is_dir() {
            utility::get_project(&path, self.exclude()).map(|p_opt| {
                if let Some(project) = p_opt {
                    projects.write().push(project);
                }
            })?;

            match fs::read_dir(&path) {
                Ok(entries) => {
                    let entries_vec: Vec<_> = entries.flatten().collect();

                    // Create progress bar for scanning directories
                    let pb = ProgressBar::new(entries_vec.len() as u64);
                    pb.set_style(
                        ProgressStyle::default_bar()
                            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                            .unwrap_or_else(|_| ProgressStyle::default_bar())
                            .progress_chars("#>-"),
                    );
                    pb.set_message("Scanning projects...");

                    entries_vec.iter().par_bridge().for_each(|entry| {
                        let pathbuf = entry.path();
                        if pathbuf.is_dir() {
                            match utility::get_project(&pathbuf, self.exclude()) {
                                Ok(p_opt) => {
                                    if let Some(project) = p_opt {
                                        projects.write().push(project);
                                    }
                                }
                                Err(error) => log::warn!("Error checking project: {error}"),
                            }
                        }
                        pb.inc(1);
                    });

                    pb.finish_and_clear();
                }
                Err(error) => log::warn!("Error reading directory: {error}"),
            }
        } else {
            anyhow::bail!("The provided path is not a directory.");
        }

        Ok(projects.read().to_vec())
    }
}
