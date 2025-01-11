#![allow(
    clippy::blanket_clippy_restriction_lints,
    reason = "I want it thaaat way"
)]
use std::{env, process::exit};

use anyhow::Context as _;
use clap::Parser as _;
use cli::Commands;
use log::LevelFilter;
use simplelog::{ColorChoice, TerminalMode};

mod cli;
mod data;
mod extensions;
#[cfg(test)]
mod test;
mod utils;

fn main() {
    match real_main() {
        Ok(()) => {}
        Err(e) => {
            log::error!("{:#}", e);
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
