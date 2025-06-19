use clap::{Parser, ValueEnum};
use parking_lot::RwLock;
use rayon::iter::{ParallelBridge as _, ParallelIterator as _};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::data::Project;
use crate::utility;

/// Represents supported Cargo subcommands.
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

/// Represents the command line options for the `execute` command.
#[derive(Parser, Default, Clone)]
pub struct ExecuteOptions {
    /// Path to the directory from which to start the search for Rust projects
    #[clap(long, short)]
    pub path: Option<PathBuf>,

    /// Exclude the provided folder from the size calculation and cleaning
    /// You can specify multiple folders separated by commas
    #[clap(long, short)]
    pub exclude: Option<String>,

    /// The command to execute on the Rust projects
    #[clap(long, short)]
    pub command: Option<CargoCommand>,
}

/// Represents general command line options.
#[derive(Parser, Default, Clone)]
pub struct Options {
    /// Path to the directory from which to start the search for Rust projects
    #[clap(long, short)]
    pub path: Option<PathBuf>,

    /// Exclude the provided folder from the size calculation and cleaning
    #[clap(long, short)]
    pub exclude: Option<String>,
}

/// A trait defining common fields shared between options structs.
pub trait CommonOptions {
    fn path(&self) -> Option<&PathBuf>;
    fn exclude(&self) -> Option<&String>;
}

impl CommonOptions for Options {
    fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    fn exclude(&self) -> Option<&String> {
        self.exclude.as_ref()
    }
}

impl CommonOptions for ExecuteOptions {
    fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
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
        let path = utility::sanitize_path_input(self.path().cloned())?;

        if path.is_dir() {
            utility::check_project(&path, self.exclude()).map(|p_opt| {
                if let Some(project) = p_opt {
                    projects.write().push(project);
                }
            })?;

            match fs::read_dir(&path) {
                Ok(entries) => {
                    entries.flatten().par_bridge().for_each(|entry| {
                        let pathbuf = entry.path();
                        if pathbuf.is_dir() {
                            match utility::check_project(&pathbuf, self.exclude()) {
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
