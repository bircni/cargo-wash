use core::num::NonZeroUsize;
use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use glob::{Pattern, PatternError};

use crate::size::Size;

/// Command line interface of `cargo-wash`
#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Show more output, use twice for even more
    #[arg(short, long, action = ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Only show warnings and errors
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,
}

/// Represents the available commands
#[derive(Subcommand)]
pub enum Command {
    /// Print statistics about all Rust projects in the directory
    Stats(StatsArgs),
    /// Clean the `target` folders of all Rust projects in the directory
    Clean(CleanArgs),
    #[command(flatten)]
    Cargo(CargoCommand),
}

/// Commands that are forwarded to `cargo` for every project
#[derive(Subcommand)]
pub enum CargoCommand {
    /// Execute `cargo build` on all Rust projects in the directory
    Build(ExecArgs),
    /// Execute `cargo check` on all Rust projects in the directory
    Check(ExecArgs),
    /// Execute `cargo doc` on all Rust projects in the directory
    Doc(ExecArgs),
    /// Execute `cargo run` on all Rust projects in the directory
    Run(ExecArgs),
    /// Execute `cargo test` on all Rust projects in the directory
    Test(ExecArgs),
    /// Execute `cargo bench` on all Rust projects in the directory
    Bench(ExecArgs),
    /// Execute `cargo update` on all Rust projects in the directory
    Update(ExecArgs),
}

impl CargoCommand {
    /// The cargo subcommand to run together with its options
    pub const fn parts(&self) -> (&'static str, &ExecArgs) {
        match self {
            Self::Build(args) => ("build", args),
            Self::Check(args) => ("check", args),
            Self::Doc(args) => ("doc", args),
            Self::Run(args) => ("run", args),
            Self::Test(args) => ("test", args),
            Self::Bench(args) => ("bench", args),
            Self::Update(args) => ("update", args),
        }
    }
}

/// Options of the `stats` command
#[derive(Args)]
pub struct StatsArgs {
    #[command(flatten)]
    pub scan: ScanArgs,

    #[command(flatten)]
    pub filter: FilterArgs,

    /// Print the projects as JSON with sizes in bytes
    #[arg(long)]
    pub json: bool,
}

/// Options of the `clean` command
#[derive(Args)]
pub struct CleanArgs {
    #[command(flatten)]
    pub scan: ScanArgs,

    #[command(flatten)]
    pub filter: FilterArgs,

    /// Show what would be cleaned without removing anything
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Clean without asking for confirmation
    #[arg(short, long)]
    pub yes: bool,

    #[command(flatten)]
    pub run: RunArgs,
}

/// Options of the commands forwarded to cargo
#[derive(Args)]
pub struct ExecArgs {
    #[command(flatten)]
    pub scan: ScanArgs,

    #[command(flatten)]
    pub run: RunArgs,
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

/// Options selecting and ordering projects by the size of their target folder
#[derive(Args)]
pub struct FilterArgs {
    /// Order of the projects
    #[arg(long, value_enum, default_value_t = SortBy::Name)]
    pub sort: SortBy,

    /// Only include projects whose target folder is at least this big, e.g. `500MB` or `1.5GB`
    #[arg(long, value_name = "SIZE")]
    pub min_size: Option<Size>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum SortBy {
    /// Alphabetically by name
    Name,
    /// Largest target folder first
    Size,
}

/// Options controlling how cargo is run in the projects
#[derive(Args)]
pub struct RunArgs {
    /// Number of projects to process at the same time
    ///
    /// Defaults to 1 for cargo commands, which use all cores on their own,
    /// and to the number of cores for `clean`.
    #[arg(short, long)]
    pub jobs: Option<NonZeroUsize>,

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

    use super::{Cli, Command, ScanArgs, SortBy};
    use crate::size::Size;

    fn long_help(mut command: clap::Command) -> String {
        command.render_long_help().to_string()
    }

    #[test]
    fn help_snapshots() {
        let app = Cli::command().name("cargo-wash").version("0.0.0");
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

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from([&["cargo-wash"], args].concat())
    }

    fn scan_args(args: &[&str]) -> Result<ScanArgs, clap::Error> {
        parse(&[&["stats"], args].concat()).map(|cli| match cli.command {
            Command::Stats(stats) => stats.scan,
            Command::Clean(_) | Command::Cargo(_) => unreachable!("parsed `stats`"),
        })
    }

    #[test]
    fn cargo_commands_map_to_their_subcommand() {
        for name in ["build", "check", "doc", "run", "test", "bench", "update"] {
            let Command::Cargo(command) = parse(&[name, "-j", "2", "--", "--release"])
                .unwrap()
                .command
            else {
                panic!("`{name}` was not parsed as a cargo command");
            };
            let (subcommand, args) = command.parts();
            assert_eq!(subcommand, name);
            assert_eq!(args.run.args, ["--release"]);
            assert_eq!(args.run.jobs.map(usize::from), Some(2));
        }
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

    #[test]
    fn clean_parses_safety_and_filter_flags() {
        let Command::Clean(clean) = parse(&["clean", "-n", "--min-size", "1GB", "--sort", "size"])
            .unwrap()
            .command
        else {
            panic!("`clean` was not parsed");
        };
        assert!(clean.dry_run && !clean.yes, "only --dry-run was passed");
        assert_eq!(clean.filter.min_size, Some(Size(1024 * 1024 * 1024)));
        assert_eq!(clean.filter.sort, SortBy::Size);
        assert!(
            parse(&["clean", "--min-size", "big"]).is_err(),
            "not a size"
        );
        assert!(
            parse(&["clean", "--jobs", "0"]).is_err(),
            "at least one job"
        );
    }

    #[test]
    fn verbosity_flags_are_global_and_exclusive() {
        assert_eq!(parse(&["stats", "-vv"]).unwrap().verbose, 2);
        assert!(
            parse(&["-q", "stats"]).unwrap().quiet,
            "-q before the command"
        );
        assert!(
            parse(&["stats", "-q", "-v"]).is_err(),
            "-q conflicts with -v"
        );
    }

    #[test]
    fn stats_does_not_accept_cargo_options() {
        assert!(parse(&["stats", "--log-dir", "logs"]).is_err(), "--log-dir");
        assert!(
            parse(&["stats", "--", "--release"]).is_err(),
            "cargo arguments"
        );
    }
}
