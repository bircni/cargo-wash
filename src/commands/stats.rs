use serde::Serialize;

use crate::{cli::StatsArgs, commands::measured_projects, project::Project, report, size::Size};

/// The machine readable output of the command
#[derive(Serialize)]
struct Stats<'a> {
    projects: &'a [Project],
    total_size: Size,
}

#[expect(
    clippy::print_stdout,
    reason = "the statistics are the output of the command"
)]
pub fn run(args: &StatsArgs) -> anyhow::Result<()> {
    let projects = measured_projects(&args.scan, &args.filter)?;
    if args.json {
        let stats = Stats {
            projects: &projects,
            total_size: report::total_size(&projects),
        };
        println!("{}", serde_json::to_string_pretty(&stats)?);
    } else {
        println!("{}", report::table(&projects));
    }
    Ok(())
}
