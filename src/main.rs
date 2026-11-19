//! `cargo-wash` cleans up and runs cargo commands across many Rust projects.

use std::{env, ffi::OsString, process::ExitCode};

use anyhow::Context as _;
use clap::Parser as _;
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

use crate::{
    cli::{Cli, Command},
    report::ProgressLogger,
};

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
    let cli = Cli::parse_from(strip_cargo_subcommand(env::args_os()));
    initialize_logger(log_level(cli.verbose, cli.quiet))?;
    match &cli.command {
        Command::Stats(args) => commands::stats::run(args),
        Command::Clean(args) => commands::clean::run(args),
        Command::Cargo(cargo) => {
            let (command, args) = cargo.parts();
            commands::exec::run(command, args)
        }
    }
}

const fn log_level(verbose: u8, quiet: bool) -> LevelFilter {
    match (quiet, verbose) {
        (true, _) => LevelFilter::Warn,
        (false, 0) => LevelFilter::Info,
        (false, 1) => LevelFilter::Debug,
        (false, _) => LevelFilter::Trace,
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

/// Log to stderr, which keeps stdout free for the output of the commands.
fn initialize_logger(level: LevelFilter) -> anyhow::Result<()> {
    let config = ConfigBuilder::new()
        .add_filter_allow("cargo_wash".to_owned())
        .build();
    let logger = TermLogger::new(level, config, TerminalMode::Stderr, ColorChoice::Auto);
    log::set_boxed_logger(Box::new(ProgressLogger(logger)))
        .context("Failed to initialize logger")?;
    log::set_max_level(level);
    Ok(())
}

#[cfg(test)]
mod tests {
    use log::LevelFilter;

    use super::{log_level, strip_cargo_subcommand};

    #[test]
    fn verbosity_selects_the_log_level() {
        assert_eq!(log_level(0, false), LevelFilter::Info);
        assert_eq!(log_level(1, false), LevelFilter::Debug);
        assert_eq!(log_level(5, false), LevelFilter::Trace);
        assert_eq!(log_level(0, true), LevelFilter::Warn);
    }

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
