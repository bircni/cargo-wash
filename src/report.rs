//! Everything that is shown to the user: progress, tables and summaries.

use core::time::Duration;
use std::{collections::HashSet, sync::LazyLock};

use comfy_table::{Table, presets::UTF8_FULL_CONDENSED};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::{Log, Metadata, Record};

use crate::{project::Project, size::Size};

/// Owner of all progress bars, so that log messages can be printed above them
static PROGRESS: LazyLock<MultiProgress> = LazyLock::new(MultiProgress::new);

/// A logger that hides the progress bars while a message is written, which
/// keeps log lines from being mixed into a bar.
pub struct ProgressLogger<L>(pub L);

impl<L: Log> Log for ProgressLogger<L> {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.0.enabled(metadata)
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            PROGRESS.suspend(|| self.0.log(record));
        }
    }

    fn flush(&self) {
        self.0.flush();
    }
}

/// Create the progress bar used by all long running steps.
pub fn progress_bar(len: usize, message: impl Into<String>) -> ProgressBar {
    let style =
        ProgressStyle::with_template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("#>-");
    PROGRESS.add(
        ProgressBar::new(len as u64)
            .with_style(style)
            .with_message(message.into()),
    )
}

/// Combined size of the target folders, counting a shared folder only once
pub fn total_size<'a>(projects: impl IntoIterator<Item = &'a Project>) -> Size {
    let mut seen = HashSet::new();
    projects
        .into_iter()
        .filter(|project| seen.insert(&project.target_dir))
        .map(|project| project.size)
        .sum()
}

/// Comma separated names of the projects
pub fn names<'a>(projects: impl IntoIterator<Item = &'a Project>) -> String {
    projects
        .into_iter()
        .map(|project| project.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Table listing the projects with their size and a total row
pub fn table(projects: &[Project]) -> Table {
    let mut table = Table::new();
    table.load_style(UTF8_FULL_CONDENSED);
    table.set_header(["Project", "Size", "Path"]);
    for project in projects {
        table.add_row([
            project.name.clone(),
            project.size.to_string(),
            project.path.display().to_string(),
        ]);
    }
    table.add_row([
        "Total".to_owned(),
        total_size(projects).to_string(),
        format!("{} projects", projects.len()),
    ]);
    table
}

pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let (hours, minutes, seconds) = (secs / 3600, (secs % 3600) / 60, secs % 60);
    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;
    use std::path::Path;

    use super::{format_duration, names, table, total_size};
    use crate::{project::Project, size::Size};

    fn project(name: &str, bytes: u64) -> Project {
        Project {
            size: Size(bytes),
            ..Project::new(&Path::new("/tmp").join(name), None)
        }
    }

    #[test]
    fn formats_durations() {
        assert_eq!(format_duration(Duration::from_secs(3665)), "1h 1m 5s");
        assert_eq!(format_duration(Duration::from_secs(125)), "2m 5s");
        assert_eq!(format_duration(Duration::from_secs(45)), "45s");
    }

    #[test]
    fn summarizes_projects() {
        let projects = [project("foo", 1024), project("bar", 2048)];
        assert_eq!(total_size(&projects).bytes(), 3072);
        assert_eq!(names(&projects), "foo, bar");

        let table = table(&projects).to_string();
        assert!(table.contains("1.00 KB"), "{table}");
        assert!(table.contains("3.00 KB"), "{table}");
        assert!(table.contains("2 projects"), "{table}");
    }

    #[test]
    fn counts_a_shared_target_folder_once() {
        let shared = |name: &str| Project {
            size: Size(1024),
            ..Project::new(&Path::new("/tmp").join(name), Some(Path::new("/shared")))
        };
        assert_eq!(total_size(&[shared("foo"), shared("bar")]).bytes(), 1024);
    }
}
