pub mod opts;

use clap::Parser;
use opts::Options;

use crate::{
    cli::opts::OptionsTrait as _,
    commands::{clean, executor, stats},
};

/// Represents the available commands
#[derive(Parser)]
#[command(author, version, about)]
pub enum Commands {
    /// Print statistics about all Rust projects in the directory
    Stats(Options),
    /// Execute `cargo build` on all Rust projects in the directory
    Build(Options),
    /// Execute `cargo check` on all Rust projects in the directory
    Check(Options),
    /// Execute `cargo doc` on all Rust projects in the directory
    Doc(Options),
    /// Clean the `target` folders of all Rust projects in the directory
    Clean(Options),
    /// Execute `cargo run` on all Rust projects in the directory
    Run(Options),
    /// Execute `cargo test` on all Rust projects in the directory
    Test(Options),
    /// Execute `cargo bench` on all Rust projects in the directory
    Bench(Options),
    /// Execute `cargo update` on all Rust projects in the directory
    Update(Options),
}

impl Commands {
    pub const fn to_cargo_command(&self) -> &str {
        match self {
            Self::Build(_) => "build",
            Self::Check(_) => "check",
            Self::Doc(_) => "doc",
            Self::Run(_) => "run",
            Self::Test(_) => "test",
            Self::Bench(_) => "bench",
            Self::Update(_) => "update",
            Self::Stats(_) | Self::Clean(_) => "none",
        }
    }
}

impl Commands {
    const fn opts(&self) -> &Options {
        match self {
            Self::Stats(opts)
            | Self::Build(opts)
            | Self::Check(opts)
            | Self::Doc(opts)
            | Self::Clean(opts)
            | Self::Run(opts)
            | Self::Test(opts)
            | Self::Bench(opts)
            | Self::Update(opts) => opts,
        }
    }

    /// Run the command based on the provided options
    pub fn run(&self) -> anyhow::Result<()> {
        let projects = self.opts().check_args()?;
        match self {
            Self::Stats(_) => {
                stats::show(&projects);
            }
            Self::Clean(opts) => {
                clean::run(&projects, opts.exclude.as_ref())?;
            }
            Self::Build(opts)
            | Self::Run(opts)
            | Self::Test(opts)
            | Self::Bench(opts)
            | Self::Update(opts)
            | Self::Doc(opts)
            | Self::Check(opts) => {
                executor::run(&projects, opts, self.to_cargo_command())?;
            }
        }
        Ok(())
    }
}
