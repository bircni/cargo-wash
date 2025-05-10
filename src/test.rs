#![allow(clippy::unwrap_used, reason = "Tests")]
use anyhow::Context;
use insta::_macro_support;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str;

use clap::ColorChoice;

use crate::cli;
use crate::cli::opts::Opts;
use crate::data;
use crate::logic;

#[test]
fn test_logger() {
    super::initialize_logger().unwrap();
}

/// From <https://github.com/EmbarkStudios/cargo-deny/blob/f6e40d8eff6a507977b20588c842c53bc0bfd427/src/cargo-deny/main.rs#L369>
/// Snapshot tests for the CLI commands
#[expect(clippy::panic, reason = "Snapshot failed")]
fn snapshot_test_cli_command(app: clap::Command, cmd_name: &str) {
    let mut app_cmd = app
        .color(ColorChoice::Never)
        .version("0.0.0")
        .long_version("0.0.0");

    let mut buffer = Vec::new();
    app_cmd.write_long_help(&mut buffer).unwrap();
    let help_text = str::from_utf8(&buffer).unwrap();

    let snapshot = _macro_support::SnapshotValue::FileText {
        name: Some(cmd_name.into()),
        content: help_text,
    };

    if _macro_support::assert_snapshot(
        snapshot,
        Path::new(env!("CARGO_MANIFEST_DIR")),
        "cli-cmd",
        module_path!(),
        file!(),
        line!(),
        "help_text",
    )
    .is_err()
    {
        panic!("Snapshot failed");
    }

    for cmd in app_cmd.get_subcommands() {
        if cmd.get_name() == "help" {
            continue;
        }

        snapshot_test_cli_command(cmd.clone(), &format!("{cmd_name}-{}", cmd.get_name()));
    }
}

#[test]
fn test_check_project() {
    let res = logic::check_project(&PathBuf::from("../cargo-wash"), None)
        .unwrap()
        .unwrap();
    assert!(res.size > data::Size::to_size(0));
    assert!(res.name == "cargo-wash");
    assert!(res.path == PathBuf::from("../cargo-wash"));
}

#[test]
fn test_clean_path() {
    logic::sanitize_path_input(Some(PathBuf::from("/"))).unwrap();
    logic::sanitize_path_input(Some(PathBuf::from("."))).unwrap();
    logic::sanitize_path_input(Some(PathBuf::from(".."))).unwrap();
    logic::sanitize_path_input(Some(PathBuf::from("test"))).unwrap();
}

#[test]
fn test_cli_snapshot() {
    use clap::CommandFactory as _;

    insta::with_settings!({
        snapshot_path => "../tests/snapshots",
    }, {
        snapshot_test_cli_command(
            super::Commands::command().name("cargo_wash"),
            "cargo_wash",
        );
    });
}

#[test]
fn test_commands() {
    let opts = Opts::default();
    let command_stats = cli::Commands::Stats(opts);
    command_stats.show().unwrap();
    let opts2 = Opts {
        path: Some(PathBuf::from("/not_existing")),
        ..Default::default()
    };
    assert!(cli::Commands::Stats(opts2.clone()).show().is_err());
    assert!(cli::Commands::Clean(opts2).show().is_err());
    let opts3 = Opts {
        path: Some(PathBuf::from(".")),
        dry_run: true,
        ..Default::default()
    };
    cli::Commands::Clean(opts3).show().unwrap();
}

#[test]
fn test_get_folder_size() {
    let size = logic::get_folder_size("src").unwrap();
    assert!(size > 0);
}

// #[ignore = "This test is not reliable"]
#[test]
fn test_run_clean() {
    let opts = Opts::default();
    let (projects, _) = opts.check_args().unwrap();
    let exclude = "cargo-wash, target".to_owned();
    let result = logic::run_clean(&projects, false, Some(&exclude));
    assert!(result.is_ok(), "Test failed: {}", result.unwrap_err());
    assert!(
        *result.as_ref().unwrap() == 0,
        "Tests failed: {}",
        result.unwrap_err()
    );
}

#[test]
fn full_test() {
    // create example project
    let example_project = PathBuf::from("example_project");
    if example_project.exists() {
        fs::remove_dir_all(&example_project).unwrap();
    }

    let result: anyhow::Result<()> = (|| {
        fs::create_dir_all(&example_project)?;
        Command::new("cargo")
            .arg("init")
            .current_dir(&example_project)
            .output()?;

        Command::new("cargo")
            .arg("build")
            .current_dir(&example_project)
            .output()?;

        let mut opts = Opts::default();

        let exclude = "cargo-wash, target".to_owned();
        opts.exclude = Some(exclude);
        let command = cli::Commands::Clean(opts);
        command.show().context("Could not run command")?;
        // utils::run_clean(&projects, false, Some(&exclude))?;

        fs::remove_dir_all(&example_project)?;
        Ok(())
    })();

    assert!(result.is_ok(), "Test failed: {}", result.unwrap_err());
}
