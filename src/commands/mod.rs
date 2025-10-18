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
