#![expect(clippy::float_cmp, clippy::similar_names, reason = "Tests")]
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    str,
};

use anyhow::Context;
use clap::{ColorChoice, CommandFactory as _};
use insta::_macro_support;

use crate::{
    cli::{
        self, Commands,
        opts::{Options, OptionsTrait as _},
    },
    commands::{clean, total_size_of_projects},
    data::{self, Project, Size, SizeUnit},
    extensions::PathBufExt as _,
    utility,
};

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
    let res = utility::get_project(&PathBuf::from("../cargo-wash"), None)
        .unwrap()
        .unwrap();
    assert!(res.size > data::Size::to_size(0));
    assert!(res.name == "cargo-wash");
    assert!(res.path == PathBuf::from("../cargo-wash").as_path());
}

#[test]
fn test_clean_path() {
    utility::sanitize_path_input(&PathBuf::from("/")).unwrap();
    utility::sanitize_path_input(&PathBuf::from(".")).unwrap();
    utility::sanitize_path_input(&PathBuf::from("..")).unwrap();
    utility::sanitize_path_input(&PathBuf::from("test")).unwrap();
}

#[test]
fn test_cli_snapshot() {
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
    let opts2 = Options {
        path: PathBuf::from("/not_existing"),
        ..Default::default()
    };
    assert!(cli::Commands::Stats(opts2.clone()).run().is_err());
    assert!(cli::Commands::Clean(opts2).run().is_err());
    let opts3 = Options {
        path: PathBuf::from("."),
        ..Default::default()
    };
    cli::Commands::Clean(opts3).run().unwrap();
}

#[test]
fn test_get_folder_size() {
    let size = utility::get_folder_size("src").unwrap();
    assert!(size > 0);
}

#[test]
fn test_run_clean_excluded() {
    let opts = Options {
        path: ".".into(),
        ..Default::default()
    };
    let projects = opts.check_args().unwrap();
    let exclude = "cargo-wash, target".to_owned();
    let result = clean::run(&projects, Some(&exclude));
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

    fs::create_dir_all(&example_project).unwrap();
    Command::new("cargo")
        .arg("init")
        .current_dir(&example_project)
        .output()
        .unwrap();

    Command::new("cargo")
        .arg("build")
        .current_dir(&example_project)
        .output()
        .unwrap();

    let opts = Options {
        path: example_project,
        ..Default::default()
    };

    let command = cli::Commands::Clean(opts);
    command.run().context("Could not run command").unwrap();
    // utils::run_clean(&projects, false, Some(&exclude))?;
}

fn generate_test_opts(dir: &Path) -> Options {
    Options {
        path: dir.to_path_buf(),
        ..Default::default()
    }
}

#[test]
fn execute_test() {
    // create example project
    let tmp_dir = tempfile::tempdir().unwrap();
    let example_project = tmp_dir.path().join("example_project");

    fs::create_dir_all(&example_project).unwrap();
    Command::new("cargo")
        .arg("init")
        .current_dir(&example_project)
        .output()
        .unwrap();

    Command::new("cargo")
        .arg("build")
        .current_dir(&example_project)
        .output()
        .unwrap();

    let clean_cmd = Commands::Clean(generate_test_opts(&example_project));
    clean_cmd.run().unwrap();

    let build_cmd = Commands::Build(generate_test_opts(&example_project));
    build_cmd.run().unwrap();

    let test_cmd = Commands::Test(generate_test_opts(&example_project));
    test_cmd.run().unwrap();

    let doc_cmd = Commands::Doc(generate_test_opts(&example_project));
    doc_cmd.run().unwrap();

    let run_cmd = Commands::Run(generate_test_opts(&example_project));
    run_cmd.run().unwrap();

    let test_cmd = Commands::Update(generate_test_opts(&example_project));
    test_cmd.run().unwrap();

    let bench_cmd = Commands::Bench(generate_test_opts(&example_project));
    bench_cmd.run().unwrap();

    let update_cmd = Commands::Update(generate_test_opts(&example_project));
    update_cmd.run().unwrap();

    let stats_cmd = Commands::Stats(generate_test_opts(&example_project));
    stats_cmd.run().unwrap();
}

#[test]
fn execute_test_failures() {
    // create example project
    let tmp_dir = tempfile::tempdir().unwrap();
    let example_project = tmp_dir.path().join("example_project");

    fs::create_dir_all(&example_project).unwrap();
    Command::new("cargo")
        .arg("init")
        .current_dir(&example_project)
        .output()
        .unwrap();

    Command::new("cargo")
        .arg("build")
        .current_dir(&example_project)
        .output()
        .unwrap();

    std::fs::remove_file(example_project.join("src").join("main.rs")).unwrap();

    let clean_cmd = Commands::Clean(generate_test_opts(&example_project));
    clean_cmd.run().unwrap();

    let build_cmd = Commands::Build(generate_test_opts(&example_project));
    build_cmd.run().unwrap_err();

    let test_cmd = Commands::Test(generate_test_opts(&example_project));
    test_cmd.run().unwrap_err();

    let doc_cmd = Commands::Doc(generate_test_opts(&example_project));
    doc_cmd.run().unwrap_err();

    let run_cmd = Commands::Run(generate_test_opts(&example_project));
    run_cmd.run().unwrap_err();

    let test_cmd = Commands::Update(generate_test_opts(&example_project));
    test_cmd.run().unwrap_err();

    let bench_cmd = Commands::Bench(generate_test_opts(&example_project));
    bench_cmd.run().unwrap_err();

    let update_cmd = Commands::Update(generate_test_opts(&example_project));
    update_cmd.run().unwrap_err();

    let stats_cmd = Commands::Stats(generate_test_opts(&example_project));
    stats_cmd.run().unwrap();
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
    assert_eq!(format!("{size}"), "1.23 MB");

    let size = Size::new(2048.0, SizeUnit::B);
    assert_eq!(format!("{size}"), "2048.00 B");
}

#[test]
fn test_round_trip_conversion() {
    let original = Size::new(1.5, SizeUnit::MB);
    let bytes = original.size_in_bytes();
    let converted = Size::to_size(bytes);

    assert_eq!(converted.unit, SizeUnit::MB);
    assert!((converted.value - 1.5).abs() < 0.01);
}

#[test]
fn test_pathbufext_get_name() {
    // Test mit Datei
    let file_path = PathBuf::from("foo.txt");
    let name = file_path.get_name().unwrap();
    assert_eq!(name, "foo.txt");

    // Test mit Verzeichnis
    let dir_path = PathBuf::from("bar");
    let name = dir_path.get_name().unwrap();
    assert_eq!(name, "bar");

    // Test mit verschachteltem Pfad
    let nested_path = PathBuf::from("foo/bar/baz");
    let name = nested_path.get_name().unwrap();
    assert_eq!(name, "baz");

    // Test mit leerem PathBuf (sollte Fehler liefern)
    let empty_path = PathBuf::new();
    let result = empty_path.get_name();
    result.unwrap_err();
}

#[test]
fn test_total_size_of_projects() {
    let p1 = Project::new("foo", PathBuf::from("/tmp/foo"), 1024);
    let p2 = Project::new("bar", PathBuf::from("/tmp/bar"), 2048);
    let projects = vec![p1, p2];
    let total = total_size_of_projects(&projects);
    assert_eq!(total, 1024 + 2048);
}
