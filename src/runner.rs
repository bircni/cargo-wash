//! Runs a cargo command in every project.

use core::fmt::Write as _;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Context as _;
use indicatif::{ParallelProgressIterator as _, ProgressIterator as _};
use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};

use crate::{project::Project, report};

/// A cargo command that is run in a number of projects
pub struct Job<'a> {
    /// The cargo subcommand, e.g. `build`
    pub command: &'a str,
    /// Arguments passed after the subcommand
    pub args: &'a [String],
    /// Directory to write one log file per project to
    pub log_dir: Option<&'a Path>,
    /// Run the command in several projects at the same time
    pub parallel: bool,
}

/// The projects a [`Job`] ran in, split by whether the command succeeded
pub struct Outcome<'a> {
    pub succeeded: Vec<&'a Project>,
    pub failed: Vec<&'a Project>,
}

impl Job<'_> {
    /// Run the command in all projects, reporting every failure as it happens.
    pub fn run_all<'p>(&self, projects: &'p [Project]) -> anyhow::Result<Outcome<'p>> {
        if let Some(dir) = self.log_dir {
            fs::create_dir_all(dir)
                .with_context(|| format!("Could not create log directory: {}", dir.display()))?;
        }

        let progress = report::progress_bar(
            projects.len(),
            format!("Executing cargo {}...", self.command),
        );
        let run = |project: &'p Project| (project, self.run(project));
        let results: Vec<(&Project, bool)> = if self.parallel {
            projects
                .par_iter()
                .progress_with(progress.clone())
                .map(run)
                .collect()
        } else {
            projects
                .iter()
                .progress_with(progress.clone())
                .map(run)
                .collect()
        };
        progress.finish_and_clear();

        let (succeeded, failed): (Vec<_>, Vec<_>) =
            results.into_iter().partition(|(_, success)| *success);
        Ok(Outcome {
            succeeded: succeeded.into_iter().map(|(project, _)| project).collect(),
            failed: failed.into_iter().map(|(project, _)| project).collect(),
        })
    }

    /// Run the command in a single project and return whether it succeeded.
    fn run(&self, project: &Project) -> bool {
        let Self { command, args, .. } = self;
        log::debug!(
            "Running `cargo {command} {}` for project: {}",
            args.join(" "),
            project.name
        );

        let output = match Command::new("cargo")
            .arg(command)
            .args(*args)
            .current_dir(&project.path)
            .output()
        {
            Ok(output) => output,
            Err(error) => {
                log::error!(
                    "Could not run `cargo {command}` on {}: {error}",
                    project.name
                );
                return false;
            }
        };

        if let Some(log_dir) = self.log_dir {
            match self.write_log(log_dir, project, &output) {
                Ok(file) => log::debug!("Wrote log for {} to {}", project.name, file.display()),
                Err(error) => log::warn!("Could not write log for {}: {error}", project.name),
            }
        }

        if !output.status.success() {
            log::error!(
                "`cargo {command}` failed on {} ({}):\n{}",
                project.name,
                output.status,
                String::from_utf8_lossy(&output.stderr).trim_end()
            );
        }
        output.status.success()
    }

    fn write_log(&self, dir: &Path, project: &Project, output: &Output) -> anyhow::Result<PathBuf> {
        let file = dir.join(format!(
            "{}-{}.log",
            sanitize_for_filename(&project.name),
            sanitize_for_filename(self.command)
        ));
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs());

        let mut content = String::new();
        writeln!(content, "timestamp: {timestamp}")?;
        writeln!(content, "project: {}", project.name)?;
        writeln!(content, "command: cargo {}", self.command)?;
        if !self.args.is_empty() {
            writeln!(content, "args: {}", self.args.join(" "))?;
        }
        writeln!(content, "status: {}\n", output.status)?;
        writeln!(content, "--- stdout ---")?;
        writeln!(content, "{}\n", String::from_utf8_lossy(&output.stdout))?;
        writeln!(content, "--- stderr ---")?;
        writeln!(content, "{}", String::from_utf8_lossy(&output.stderr))?;

        fs::write(&file, content)?;
        Ok(file)
    }
}

fn sanitize_for_filename(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::sanitize_for_filename;

    #[test]
    fn sanitizes_file_names() {
        assert_eq!(sanitize_for_filename("my-project_1"), "my-project_1");
        assert_eq!(sanitize_for_filename("a b/c.d"), "a_b_c_d");
    }
}
