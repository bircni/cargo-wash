use anyhow::Context as _;
use clap::Parser as _;
use cli::Commands;
use log::LevelFilter;
use simplelog::{ColorChoice, TerminalMode};
use std::{env, process::exit};

mod cli;
mod data;
mod extensions;
mod logic;
#[cfg(test)]
mod test;

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
    let args = Commands::parse_from(env::args().filter(|a| a != "wash"));
    args.show()
}

fn initialize_logger() -> anyhow::Result<()> {
    simplelog::TermLogger::init(
        #[cfg(debug_assertions)]
        LevelFilter::max(),
        #[cfg(not(debug_assertions))]
        LevelFilter::Info,
        simplelog::Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .context("Failed to initialize logger")
}
