use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use glob::{Pattern, PatternError};

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

/// Options selecting the projects a command applies to
#[derive(Args, Clone)]
pub struct ScanArgs {
    /// Directory to search for Rust projects, can be given multiple times
    #[arg(short, long = "path", value_name = "PATH", default_value = ".")]
    pub paths: Vec<PathBuf>,

    /// How many directory levels below each path are searched
    ///
    /// `0` only checks the paths themselves. The search never continues inside
    /// a project, a hidden folder, `target` or `node_modules`.
    #[arg(short, long, default_value_t = 1)]
    pub depth: usize,

    /// Skip folders whose name or path relative to the search path matches
    /// the glob pattern, can be given multiple times or comma-separated
    #[arg(short, long, value_name = "PATTERN", value_delimiter = ',', value_parser = parse_pattern)]
    pub exclude: Vec<Pattern>,
}

fn parse_pattern(pattern: &str) -> Result<Pattern, PatternError> {
    Pattern::new(pattern.trim())
}

/// Represents general command line options.
#[derive(Args, Clone)]
pub struct Options {
    #[command(flatten)]
    pub scan: ScanArgs,

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
    use std::path::Path;

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

    fn scan_args(args: &[&str]) -> Result<super::ScanArgs, clap::Error> {
        let args = ["cargo-wash", "stats"].iter().chain(args);
        Command::try_parse_from(args).map(|command| match command {
            Command::Stats(options) | Command::Clean(options) => options.scan,
            Command::Cargo(cargo) => cargo.parts().1.scan.clone(),
        })
    }

    #[test]
    fn scan_defaults_to_the_current_directory_and_its_children() {
        let scan = scan_args(&[]).unwrap();
        assert_eq!(scan.paths, [Path::new(".")]);
        assert_eq!(scan.depth, 1);
        assert_eq!(scan.exclude, []);
    }

    #[test]
    fn scan_accepts_repeated_and_comma_separated_values() {
        let scan = scan_args(&["-p", "a", "-p", "b", "-d", "3", "-e", "x, y*", "-e", "z"]).unwrap();
        assert_eq!(scan.paths, [Path::new("a"), Path::new("b")]);
        assert_eq!(scan.depth, 3);
        let patterns: Vec<&str> = scan.exclude.iter().map(glob::Pattern::as_str).collect();
        assert_eq!(patterns, ["x", "y*", "z"]);
    }

    #[test]
    fn scan_rejects_invalid_patterns() {
        assert!(
            scan_args(&["--exclude", "[a"]).is_err(),
            "unclosed character class"
        );
    }
}
