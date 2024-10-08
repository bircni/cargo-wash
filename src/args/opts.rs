use std::{fs, path::PathBuf};

use clap::Parser;
use log::warn;

use crate::{
    data::{Language, Project},
    extensions::PathBufExt,
    utils,
};

/// Represents the command line options
#[derive(Parser)]
pub struct Opts {
    /// Path to a directory
    #[clap(long, short)]
    pub path: Option<PathBuf>,
    /// Run the program without making any changes
    #[clap(short, long)]
    pub dry_run: bool,
    /// Exclude specific projects (by name)
    #[clap(long, short = 'e')]
    pub exclude: Vec<String>, // Use a vector to allow multiple exclusions
    /// Language to filter by
    #[clap(long, short)]
    pub language: Option<Language>,
}

impl Opts {
    pub fn check_args(&self) -> anyhow::Result<(Vec<Project>, bool)> {
        let mut projects: Vec<Project> = vec![];
        let path = super::clean_path(self.path.clone())?;

        if path.is_dir() {
            self.check_project(&path, self.language).map(|p| {
                if let Some(p) = p {
                    projects.push(p);
                }
            })?;

            match fs::read_dir(&path) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            self.check_project(&path, self.language).map(|p| {
                                if let Some(p) = p {
                                    projects.push(p);
                                }
                            })?;
                        }
                    }
                }
                Err(e) => warn!("Error reading directory: {}", e),
            }
        } else {
            anyhow::bail!("The provided path is not a directory.");
        }

        Ok((projects, self.dry_run))
    }

    fn check_project(
        &self,
        path: &PathBuf,
        lang: Option<Language>,
    ) -> anyhow::Result<Option<Project>> {
        let detected_lang = utils::get_project(path);
        let name = &path.get_name()?;

        if self.exclude.contains(name) {
            return Ok(None);
        }

        if let Some(expected_lang) = lang {
            if detected_lang != expected_lang {
                return Ok(None);
            }
        }

        match detected_lang {
            Language::Rust => {
                let size = utils::get_folder_size(path.join("target"))?;
                if size > 0 {
                    return Ok(Some(Project::new(name, path, size, Language::Rust)));
                }
                Ok(None)
            }
            Language::NodeJS => {
                let size = utils::get_folder_size(path.join("node_modules"))?;
                if size > 0 {
                    return Ok(Some(Project::new(name, path, size, Language::NodeJS)));
                }
                Ok(None)
            }
            Language::Other => Ok(None),
        }
    }
}
