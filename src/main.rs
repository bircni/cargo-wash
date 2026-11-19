use anyhow::Context as _;
use clap::Parser as _;
use cli::Commands;
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TerminalMode};
use std::{env, ffi::OsString, process::exit};

mod cli;
mod commands;
mod data;
mod extensions;
#[cfg(test)]
mod test;
mod utility;

fn main() {
    match real_main() {
        Ok(()) => {}
        Err(e) => {
            log::error!("{e:#}");
            exit(1);
        }
    }
}

fn real_main() -> anyhow::Result<()> {
    initialize_logger()?;
    let args = Commands::parse_from(strip_cargo_subcommand(env::args_os()));
    args.run()
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
    simplelog::TermLogger::init(
        #[cfg(debug_assertions)]
        LevelFilter::max(),
        #[cfg(not(debug_assertions))]
        LevelFilter::Info,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .context("Failed to initialize logger")
}
