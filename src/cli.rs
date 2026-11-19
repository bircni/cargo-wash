use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Represents the available commands
#[derive(Parser)]
#[command(author, version, about)]
pub enum Command {
    /// Print statistics about all Rust projects in the directory
    Stats(Options),
    /// Clean the `target` folders of all Rust projects in the directory
    Clean(Options),
    #[command(flatten)]
    Cargo(CargoCommand),
}

/// Commands that are forwarded to `cargo` for every project
#[derive(Subcommand)]
pub enum CargoCommand {
    /// Execute `cargo build` on all Rust projects in the directory
    Build(Options),
    /// Execute `cargo check` on all Rust projects in the directory
    Check(Options),
    /// Execute `cargo doc` on all Rust projects in the directory
    Doc(Options),
    /// Execute `cargo run` on all Rust projects in the directory
    Run(Options),
    /// Execute `cargo test` on all Rust projects in the directory
    Test(Options),
    /// Execute `cargo bench` on all Rust projects in the directory
    Bench(Options),
    /// Execute `cargo update` on all Rust projects in the directory
    Update(Options),
}

impl CargoCommand {
    /// The cargo subcommand to run together with its options
    pub const fn parts(&self) -> (&'static str, &Options) {
        match self {
            Self::Build(options) => ("build", options),
            Self::Check(options) => ("check", options),
            Self::Doc(options) => ("doc", options),
            Self::Run(options) => ("run", options),
            Self::Test(options) => ("test", options),
            Self::Bench(options) => ("bench", options),
            Self::Update(options) => ("update", options),
        }
    }
}

/// Represents general command line options.
#[derive(Args, Default, Clone)]
pub struct Options {
    /// Path to the directory from which to start the search for Rust projects
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,

    /// Exclude folders by name (comma-separated) from all commands
    #[arg(long, short)]
    pub exclude: Option<String>,

    /// Enable parallel processing of projects
    /// ATTENTION: This may lead to high CPU usage!
    #[arg(long, default_value_t = false, verbatim_doc_comment)]
    pub parallel: bool,

    /// Directory to write per-project command logs
    #[arg(long)]
    pub log_dir: Option<PathBuf>,

    /// Additional arguments to pass to the cargo command
    #[arg(last = true)]
    pub args: Vec<String>,
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory as _, Parser as _};

    use super::{CargoCommand, Command};

    fn long_help(mut command: clap::Command) -> String {
        command.render_long_help().to_string()
    }

    #[test]
    fn help_snapshots() {
        let app = Command::command().name("cargo-wash").version("0.0.0");
        insta::with_settings!({ snapshot_path => "../tests/snapshots" }, {
            insta::assert_snapshot!("help", long_help(app.clone()));
            for subcommand in app.get_subcommands().filter(|cmd| cmd.get_name() != "help") {
                insta::assert_snapshot!(
                    format!("help-{}", subcommand.get_name()),
                    long_help(subcommand.clone())
                );
            }
        });
    }

    #[test]
    fn cargo_commands_map_to_their_subcommand() {
        for name in ["build", "check", "doc", "run", "test", "bench", "update"] {
            let Command::Cargo(command) =
                Command::parse_from(["cargo-wash", name, "--", "--release"])
            else {
                panic!("`{name}` was not parsed as a cargo command");
            };
            let (subcommand, options) = CargoCommand::parts(&command);
            assert_eq!(subcommand, name);
            assert_eq!(options.args, ["--release"]);
        }
    }
}
