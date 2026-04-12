use std::{
    fs,
    path::{Path, PathBuf},
    process::Output,
    time::{SystemTime, UNIX_EPOCH},
};

use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};

use crate::data::{Project, Size};

pub mod clean;
pub mod executor;
pub mod stats;

pub fn total_size_of_projects(projects: &[Project]) -> u64 {
    projects
        .par_iter()
        .map(|project| project.size.size_in_bytes())
        .sum()
}

pub fn print_status(projects: &[Project], cleaned: &[Project], exclude: Option<&String>) {
    let total_size = total_size_of_projects(cleaned);

    let skipped = exclude.map_or_else(String::new, |skip| format!("\n(Skipped: {skip})"));
    log::info!(
        "Cleaned {} ({} Projects)\nProjects: {}{skipped}",
        Size::to_size(total_size),
        projects.len(),
        cleaned
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<String>>()
            .join(", ")
    );
}

pub fn ensure_log_dir(log_dir: Option<&Path>) -> anyhow::Result<()> {
    if let Some(dir) = log_dir {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

#[expect(
    clippy::format_push_string,
    reason = "Using push_str for better performance when building log content"
)]
pub fn write_project_log(
    log_dir: Option<&Path>,
    project_name: &str,
    command: &str,
    args: &[String],
    output: &Output,
) -> anyhow::Result<Option<PathBuf>> {
    let Some(dir) = log_dir else {
        return Ok(None);
    };

    let file_name = format!(
        "{}-{}.log",
        sanitize_for_filename(project_name),
        sanitize_for_filename(command)
    );
    let file_path = dir.join(file_name);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());

    let mut buffer = String::new();
    buffer.push_str(&format!("timestamp: {timestamp}\n"));
    buffer.push_str(&format!("project: {project_name}\n"));
    buffer.push_str(&format!("command: cargo {command}\n"));
    if !args.is_empty() {
        buffer.push_str(&format!("args: {}\n", args.join(" ")));
    }
    buffer.push_str(&format!("status: {}\n\n", output.status));
    buffer.push_str("--- stdout ---\n");
    buffer.push_str(&String::from_utf8_lossy(&output.stdout));
    buffer.push_str("\n\n--- stderr ---\n");
    buffer.push_str(&String::from_utf8_lossy(&output.stderr));
    buffer.push('\n');

    fs::write(&file_path, buffer)?;
    Ok(Some(file_path))
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
