#![expect(clippy::float_cmp, clippy::similar_names, reason = "Tests")]
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
use crate::data::Size;
use crate::data::SizeUnit;
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
        ..Default::default()
    };
    cli::Commands::Clean(opts3).show().unwrap();
}

#[test]
fn test_get_folder_size() {
    let size = logic::get_folder_size("src").unwrap();
    assert!(size > 0);
}

#[test]
fn test_run_clean_excluded() {
    let opts = Opts::default();
    let projects = opts.check_args().unwrap();
    let exclude = "cargo-wash, target".to_owned();
    let result = logic::run_clean(&projects, Some(&exclude));
    assert!(result.is_ok(), "Test failed: {}", result.unwrap_err());
    assert!(
        *result.as_ref().unwrap() == 0,
        "Tests failed: {}",
        result.unwrap_err()
    );
}

#[test]
fn clean_test() {
    // create example project
    let tmp_dir = tempfile::tempdir().unwrap();
    let example_project = tmp_dir.path().join("example_project");

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

        let opts = Opts {
            path: Some(example_project.clone()),
            ..Default::default()
        };

        let command = cli::Commands::Clean(opts);
        command.show().context("Could not run command")?;
        // utils::run_clean(&projects, false, Some(&exclude))?;
        Ok(())
    })();

    assert!(result.is_ok(), "Test failed: {}", result.unwrap_err());
}

#[test]
fn rebuild_test() {
    // create example project
    let tmp_dir = tempfile::tempdir().unwrap();
    let example_project = tmp_dir.path().join("example_project");

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

        let opts = Opts {
            path: Some(example_project.clone()),
            ..Default::default()
        };

        let command = cli::Commands::Rebuild(opts);
        command.show().context("Could not run command")?;
        Ok(())
    })();

    assert!(result.is_ok(), "Test failed: {}", result.unwrap_err());
}

#[test]
fn test_size_in_bytes() {
    let size_b = Size::new(500.0, SizeUnit::B);
    assert_eq!(size_b.size_in_bytes(), 500);

    let size_kb = Size::new(1.0, SizeUnit::KB);
    assert_eq!(size_kb.size_in_bytes(), 1024);

    let size_mb = Size::new(1.0, SizeUnit::MB);
    assert_eq!(size_mb.size_in_bytes(), 1024 * 1024);

    let size_gb = Size::new(1.0, SizeUnit::GB);
    assert_eq!(size_gb.size_in_bytes(), 1024 * 1024 * 1024);
}

#[test]
fn test_to_size() {
    let size = Size::to_size(500);
    assert_eq!(size.unit, SizeUnit::B);
    assert_eq!(size.value, 500.0);

    let size = Size::to_size(1024);
    assert_eq!(size.unit, SizeUnit::KB);
    assert_eq!(size.value, 1.0);

    let size = Size::to_size(1024 * 1024);
    assert_eq!(size.unit, SizeUnit::MB);
    assert_eq!(size.value, 1.0);

    let size = Size::to_size(1024 * 1024 * 1024);
    assert_eq!(size.unit, SizeUnit::GB);
    assert_eq!(size.value, 1.0);
}

#[test]
fn test_display() {
    let size = Size::new(1.2345, SizeUnit::MB);
    assert_eq!(format!("{}", size), "1.23 MB");

    let size = Size::new(2048.0, SizeUnit::B);
    assert_eq!(format!("{}", size), "2048.00 B");
}

#[test]
fn test_round_trip_conversion() {
    let original = Size::new(1.5, SizeUnit::MB);
    let bytes = original.size_in_bytes();
    let converted = Size::to_size(bytes);

    assert_eq!(converted.unit, SizeUnit::MB);
    assert!((converted.value - 1.5).abs() < 0.01);
}
