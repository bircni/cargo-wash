use crate::{cli::Options, report, scan};

#[expect(
    clippy::print_stdout,
    reason = "the table is the output of the command"
)]
pub fn run(options: &Options) -> anyhow::Result<()> {
    let mut projects = scan::discover(options)?;
    scan::measure(&mut projects);
    println!("{}", report::table(&projects));
    Ok(())
}
