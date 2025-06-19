use anyhow::Context as _;
use clap::Parser as _;
use cli::Commands;
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TerminalMode};
use std::{env, process::exit};
use update_available::Source;

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
    update_available::print_check(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        Source::CratesIo,
    );
    let args = Commands::parse_from(env::args().filter(|a| a != "wash"));
    args.run()
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
