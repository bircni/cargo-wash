use clap::{Parser, ValueEnum};
use parking_lot::RwLock;
use rayon::iter::{ParallelBridge as _, ParallelIterator as _};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::data::Project;
use crate::utility;

#[derive(ValueEnum, Default, Clone)]
pub enum CargoCommand {
    /// Execute `cargo build`
    Build,
    /// Execute `cargo check`
    Check,
    /// Execute `cargo doc`
    Doc,
    /// Execute `cargo clean`
    Clean,
    /// Execute `cargo run`
    Run,
    /// Execute `cargo test`
    Test,
    /// Execute `cargo bench`
    Bench,
    /// Execute `cargo update`
    Update,
    /// No command, used for default or no operation
    #[default]
    None,
}

impl CargoCommand {
    pub const fn to_command(&self) -> &str {
        match self {
            Self::Build => "build",
            Self::Check => "check",
            Self::Doc => "doc",
            Self::Clean => "clean",
            Self::Run => "run",
            Self::Test => "test",
            Self::Bench => "bench",
            Self::Update => "update",
            Self::None => "",
        }
    }
}

/// Represents the command line options
#[derive(Parser, Default, Clone)]
pub struct Options {
    /// Path to the directory from which to start the search for Rust projects
    /// default is `.`
    #[clap(long, short)]
    pub path: Option<PathBuf>,
    /// Exclude the provided folder from the size calculation and cleaning
    /// default is `None` - you can specify multiple folders separated by `,`
    #[clap(long, short)]
    pub exclude: Option<String>,
    /// The command to execute on the Rust projects
    /// It is only used for the `execute` command
    #[clap(long, short)]
    pub command: Option<CargoCommand>,
}

impl Options {
    pub fn check_args(&self) -> anyhow::Result<Vec<Project>> {
        let projects: Arc<RwLock<Vec<Project>>> = Arc::new(RwLock::new(vec![]));
        let path = utility::sanitize_path_input(self.path.clone())?;

        if path.is_dir() {
            utility::check_project(&path, self.exclude.as_ref()).map(|p_opt| {
                if let Some(project) = p_opt {
                    projects.write().push(project);
                }
            })?;

            match fs::read_dir(&path) {
                Ok(entries) => {
                    entries.flatten().par_bridge().for_each(|entry| {
                        let pathbuf = entry.path();
                        if pathbuf.is_dir() {
                            match utility::check_project(&pathbuf, self.exclude.as_ref()) {
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
