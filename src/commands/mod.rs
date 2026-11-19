use core::cmp::Reverse;

use crate::{
    cli::{FilterArgs, ScanArgs, SortBy},
    project::Project,
    scan,
};

pub mod clean;
pub mod exec;
pub mod stats;

/// Find the projects and measure their target folders, then filter and sort
/// them as requested.
fn measured_projects(scan: &ScanArgs, filter: &FilterArgs) -> anyhow::Result<Vec<Project>> {
    let mut projects = scan::discover(scan)?;
    scan::measure(&mut projects);
    Ok(apply_filter(projects, filter))
}

fn apply_filter(mut projects: Vec<Project>, filter: &FilterArgs) -> Vec<Project> {
    if let Some(min_size) = filter.min_size {
        projects.retain(|project| project.size >= min_size);
    }
    match filter.sort {
        // Discovery already returns the projects sorted by name.
        SortBy::Name => {}
        SortBy::Size => projects.sort_by_key(|project| Reverse(project.size)),
    }
    projects
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::apply_filter;
    use crate::{
        cli::{FilterArgs, SortBy},
        project::Project,
        size::Size,
    };

    fn filtered(sort: SortBy, min_size: Option<u64>) -> Vec<String> {
        let projects = [("a", 10), ("b", 3000), ("c", 200)]
            .into_iter()
            .map(|(name, bytes)| Project {
                size: Size(bytes),
                ..Project::new(Path::new(name), None)
            })
            .collect();
        let filter = FilterArgs {
            sort,
            min_size: min_size.map(Size),
        };
        apply_filter(projects, &filter)
            .into_iter()
            .map(|project| project.name)
            .collect()
    }

    #[test]
    fn sorts_and_filters_by_size() {
        assert_eq!(filtered(SortBy::Name, None), ["a", "b", "c"]);
        assert_eq!(filtered(SortBy::Size, None), ["b", "c", "a"]);
        assert_eq!(filtered(SortBy::Name, Some(200)), ["b", "c"]);
        assert_eq!(filtered(SortBy::Size, Some(5000)), [""; 0]);
    }
}
