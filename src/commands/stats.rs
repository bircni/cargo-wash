use comfy_table::{Table, presets::UTF8_FULL_CONDENSED};

use crate::{
    commands::total_size_of_projects,
    data::{Project, Size},
};

#[expect(clippy::print_stdout, reason = "No other way to show the stats")]
pub fn show(projects: &[Project]) {
    let mut sorted_projects: Vec<Project> = projects.to_vec();
    sorted_projects.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["Project", "Size", "Path"]);

    for project in sorted_projects {
        table.add_row(vec![
            &project.name,
            &project.size.to_string(),
            &project.path.to_string_lossy().to_string(),
        ]);
    }

    table.add_row(vec![
        "Total",
        &Size::to_size(total_size_of_projects(projects)).to_string(),
        &format!("{} projects", projects.len()),
    ]);
    println!("{table}");
}
