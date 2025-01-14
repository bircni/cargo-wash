use log::warn;
use std::{fs, path::PathBuf};

use crate::cli::Opts;
use crate::{cli::Language, data::Project, extensions::PathBufExt as _, utils};

impl Opts {
    pub fn check_args(&self) -> anyhow::Result<(Vec<Project>, bool)> {
        let mut projects: Vec<Project> = vec![];
        let path = utils::clean_path(self.path.clone())?;

        if path.is_dir() {
            self.check_project(&path, self.language).map(|p_opt| {
                if let Some(project) = p_opt {
                    projects.push(project);
                }
            })?;

            match fs::read_dir(&path) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let pathbuf = entry.path();
                        if pathbuf.is_dir() {
                            self.check_project(&pathbuf, self.language).map(|p_opt| {
                                if let Some(project) = p_opt {
                                    projects.push(project);
                                }
                            })?;
                        }
                    }
                }
                Err(error) => warn!("Error reading directory: {error}"),
            }
        } else {
            anyhow::bail!("The provided path is not a directory.");
        }

        Ok((projects, self.dry_run))
    }

    pub(crate) fn check_project(
        &self,
        path: &PathBuf,
        lang: Option<Language>,
    ) -> anyhow::Result<Option<Project>> {
        let detected_lang = utils::get_language(path);
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
