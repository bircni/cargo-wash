//! `cargo-wash` cleans up and runs cargo commands across many Rust projects.

use std::{env, ffi::OsString, process::ExitCode};

use anyhow::Context as _;
use clap::Parser as _;
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

use crate::cli::Command;

mod cli;
mod commands;
mod project;
mod report;
mod runner;
mod scan;
mod size;

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            log::error!("{error:#}");
            ExitCode::FAILURE
        }
    }
}

fn real_main() -> anyhow::Result<()> {
    initialize_logger()?;
    match Command::parse_from(strip_cargo_subcommand(env::args_os())) {
        Command::Stats(options) => commands::stats::run(&options),
        Command::Clean(options) => commands::clean::run(&options),
        Command::Cargo(cargo) => {
            let (command, options) = cargo.parts();
            commands::exec::run(command, options)
        }
    }
}

/// Drop the `wash` argument cargo inserts when invoked as `cargo wash`.
///
/// Only the first argument is considered, so values such as `--path wash` are kept.
fn strip_cargo_subcommand(args: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    let mut args: Vec<OsString> = args.into_iter().collect();
    if args.get(1).is_some_and(|arg| arg == "wash") {
        args.remove(1);
    }
    args
}

fn initialize_logger() -> anyhow::Result<()> {
    let config = ConfigBuilder::new()
        .add_filter_allow("cargo_wash".to_owned())
        .build();
    TermLogger::init(
        if cfg!(debug_assertions) {
            LevelFilter::max()
        } else {
            LevelFilter::Info
        },
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .context("Failed to initialize logger")
}

#[cfg(test)]
mod tests {
    use super::strip_cargo_subcommand;

    fn strip(args: &[&str]) -> Vec<String> {
        strip_cargo_subcommand(args.iter().map(Into::into))
            .into_iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn strips_only_the_subcommand_inserted_by_cargo() {
        assert_eq!(
            strip(&["cargo-wash", "wash", "stats"]),
            ["cargo-wash", "stats"]
        );
        assert_eq!(strip(&["cargo-wash", "stats"]), ["cargo-wash", "stats"]);
        assert_eq!(
            strip(&["cargo-wash", "stats", "--path", "wash"]),
            ["cargo-wash", "stats", "--path", "wash"]
        );
        assert_eq!(
            strip(&["cargo-wash", "wash", "run", "--", "wash"]),
            ["cargo-wash", "run", "--", "wash"]
        );
    }
}
