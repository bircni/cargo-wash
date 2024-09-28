use anyhow::Context;
use args::Args;
use clap::Parser;
use data::Project;
use log::{debug, warn, LevelFilter};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use simplelog::{ColorChoice, ConfigBuilder, TerminalMode};
use std::fs;

mod args;
mod data;

fn main() {
    match real_main() {
        Ok(()) => {}
        Err(e) => {
            log::error!("{:#}", e);
            std::process::exit(1);
        }
    }
}

fn real_main() -> anyhow::Result<()> {
    simplelog::TermLogger::init(
        #[cfg(debug_assertions)]
        LevelFilter::max(),
        #[cfg(not(debug_assertions))]
        LevelFilter::Info,
        ConfigBuilder::new()
            // suppress all logs from dependencies
            .add_filter_allow_str("cargo_cleaner")
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .context("Failed to initialize logger")?;

    let args = Args::parse();
    debug!("Provided path: {:?}", args.path);
    debug!("Dry run: {}", args.dry_run);

    let mut projects: Vec<Project> = vec![];

    if args.path.is_dir() {
        match fs::read_dir(&args.path) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let p = check_for_cargo_and_target(&path);
                        if let Some(p) = p {
                            let size = get_folder_size(path.join("target"))?;
                            projects.push(Project::new(p, size)?);
                        }
                    }
                }
            }
            Err(e) => warn!("Error reading directory: {}", e),
        }
    } else {
        anyhow::bail!("The provided path is not a directory.");
    }

    args.command.show(&projects, args.dry_run)?;

    println!("Done!");

    Ok(())
}

fn total_size_of_projects(projects: &[Project]) -> u64 {
    projects
        .par_iter()
        .map(|project| project.size.size_in_bytes())
        .sum()
}

/// Recursively calculate the size of a folder
fn get_folder_size<P: AsRef<std::path::Path>>(dir: P) -> anyhow::Result<u64> {
    let mut total_size = 0;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::metadata(&path)?;

        if metadata.is_dir() {
            total_size += get_folder_size(&path)?;
        } else {
            total_size += metadata.len();
        }
    }

    Ok(total_size)
}

/// Check if a given directory contains both a Cargo.toml file and a target folder
fn check_for_cargo_and_target<P: AsRef<std::path::Path>>(dir: P) -> Option<P> {
    let path = dir.as_ref();
    let has_cargo_toml = path.join("Cargo.toml").is_file();
    let has_target_dir = path.join("target").is_dir();

    if has_cargo_toml && has_target_dir {
        return Some(dir);
    }

    None
}
