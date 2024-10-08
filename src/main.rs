use anyhow::Context;
use clap::Parser;
use cli::Commands;
use log::LevelFilter;
use simplelog::{ColorChoice, TerminalMode};

mod cli;
mod data;
mod extensions;
mod utils;

#[cfg(test)]
mod test;

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
        simplelog::Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .context("Failed to initialize logger")?;

    let args = Commands::parse_from(std::env::args().filter(|a| a != "wash"));

    args.show()
}
