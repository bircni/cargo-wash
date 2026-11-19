#![expect(clippy::float_cmp, clippy::similar_names, reason = "Tests")]
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    str,
};

use anyhow::Context as _;
use clap::{ColorChoice, CommandFactory as _};
use insta::_macro_support;

use crate::{
    cli::{
        self, Commands,
        opts::{Options, OptionsTrait as _},
    },
    commands::{executor, total_size_of_projects},
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
    let current_dir = std::env::current_dir().unwrap();
    let project_root = current_dir
        .ancestors()
        .find(|p| p.join("Cargo.toml").exists())
        .unwrap();

    let res = utility::get_project(&project_root.to_path_buf(), &[])
        .unwrap()
        .unwrap();
    assert!(res.size > data::Size::to_size(0));
    assert!(res.name.contains("cargo-wash"));
    assert!(
        res.path
            .as_os_str()
            .to_str()
            .unwrap()
            .contains("cargo-wash")
    );
}

/// Create a minimal cargo project with a fake `target` folder of `target_bytes` bytes.
fn create_project(parent: &Path, name: &str, target_bytes: usize) -> PathBuf {
    let project = parent.join(name);
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(
        project.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
    )
    .unwrap();
    fs::write(project.join("src").join("main.rs"), "fn main() {}\n").unwrap();
    if target_bytes > 0 {
        fs::create_dir_all(project.join("target")).unwrap();
        fs::write(
            project.join("target").join("artifact"),
            vec![0; target_bytes],
        )
        .unwrap();
    }
    project
}

fn project_names(opts: &Options) -> Vec<String> {
    let mut names: Vec<String> = opts
        .check_args()
        .unwrap()
        .into_iter()
        .map(|project| project.name)
        .collect();
    names.sort();
    names
}

#[test]
fn test_resolve_root() {
    let cwd = std::env::current_dir().unwrap();
    assert_eq!(utility::resolve_root(Path::new(".")).unwrap(), cwd);
    assert_eq!(
        utility::resolve_root(Path::new("..")).unwrap(),
        cwd.parent().unwrap()
    );
    utility::resolve_root(Path::new("not_existing")).unwrap_err();
    utility::resolve_root(Path::new("Cargo.toml")).unwrap_err();
}

#[cfg(unix)]
#[test]
fn test_resolve_root_keeps_filesystem_root() {
    assert_eq!(
        utility::resolve_root(Path::new("/")).unwrap(),
        PathBuf::from("/"),
        "the filesystem root must not be replaced by the current directory"
    );
}

#[test]
fn test_strip_cargo_subcommand() {
    let strip = |args: &[&str]| -> Vec<String> {
        super::strip_cargo_subcommand(args.iter().map(Into::into))
            .into_iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    };
    assert_eq!(
        strip(&["cargo-wash", "wash", "stats"]),
        ["cargo-wash", "stats"]
    );
    assert_eq!(strip(&["cargo-wash", "stats"]), ["cargo-wash", "stats"]);
    assert_eq!(
        strip(&["cargo-wash", "stats", "--path", "wash"]),
        ["cargo-wash", "stats", "--path", "wash"],
        "only the subcommand inserted by cargo may be removed"
    );
    assert_eq!(
        strip(&["cargo-wash", "wash", "run", "--", "wash"]),
        ["cargo-wash", "run", "--", "wash"]
    );
}

#[test]
fn test_exclude_matches_folder_names_exactly() {
    let tmp_dir = tempfile::tempdir().unwrap();
    create_project(tmp_dir.path(), "alpha", 0);
    create_project(tmp_dir.path(), "beta", 0);
    let with_exclude = |exclude: &str| Options {
        path: tmp_dir.path().to_path_buf(),
        exclude: Some(exclude.to_owned()),
        ..Default::default()
    };

    // A substring of a project name (or of a parent directory) must not exclude anything.
    assert_eq!(project_names(&with_exclude("a")), ["alpha", "beta"]);
    assert_eq!(project_names(&with_exclude("alpha")), ["beta"]);
    assert_eq!(project_names(&with_exclude("alpha, beta")), [""; 0]);
}

#[test]
fn test_clean_removes_target_and_respects_exclude() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let cleaned = create_project(tmp_dir.path(), "cleaned", 2048);
    let kept = create_project(tmp_dir.path(), "kept", 2048);

    Commands::Clean(Options {
        path: tmp_dir.path().to_path_buf(),
        exclude: Some("kept".to_owned()),
        ..Default::default()
    })
    .run()
    .unwrap();

    assert!(!cleaned.join("target").exists());
    assert!(kept.join("target").exists());
}

#[test]
fn test_clean_fails_when_a_project_cannot_be_cleaned() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let broken = create_project(tmp_dir.path(), "broken", 0);
    fs::write(broken.join("Cargo.toml"), "not a manifest").unwrap();

    Commands::Clean(generate_test_opts(tmp_dir.path()))
        .run()
        .unwrap_err();
}

#[cfg(unix)]
#[test]
fn test_get_folder_size_does_not_follow_symlinks() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let outside = tmp_dir.path().join("outside");
    let inside = tmp_dir.path().join("inside");
    fs::create_dir_all(&outside).unwrap();
    fs::create_dir_all(&inside).unwrap();
    fs::write(outside.join("big"), vec![0; 4096]).unwrap();
    fs::write(inside.join("small"), vec![0; 10]).unwrap();
    std::os::unix::fs::symlink(&outside, inside.join("dir_link")).unwrap();
    std::os::unix::fs::symlink(&inside, inside.join("loop")).unwrap();

    let size = utility::get_folder_size(&inside);
    assert!(size < 4096, "symlinked content was counted: {size}");
    assert!(size >= 10, "regular files must be counted: {size}");
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
}

#[test]
fn test_get_folder_size() {
    assert!(utility::get_folder_size("src") > 0);
    assert_eq!(utility::get_folder_size("not_existing"), 0);
}

#[test]
fn execute_test_with_args_and_project_logs() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let example_project = tmp_dir.path().join("example_project");
    let target_dir_with_space = tmp_dir.path().join("target dir with space");
    let log_dir = tmp_dir.path().join("logs");

    fs::create_dir_all(&example_project).unwrap();
    let init_output = Command::new("cargo")
        .arg("init")
        .arg("--bin")
        .current_dir(&example_project)
        .output()
        .unwrap();
    assert!(init_output.status.success());

    let opts = Options {
        path: example_project,
        log_dir: Some(log_dir.clone()),
        args: vec![
            "--target-dir".to_owned(),
            target_dir_with_space.to_string_lossy().to_string(),
        ],
        ..Default::default()
    };

    Commands::Build(opts.clone()).run().unwrap();
    Commands::Clean(opts).run().unwrap();

    let build_log = log_dir.join("example_project-build.log");

    assert!(build_log.exists());

    let build_log_content = fs::read_to_string(build_log).unwrap();
    assert!(build_log_content.contains("command: cargo build"));
    assert!(build_log_content.contains("status:"));
    assert!(build_log_content.contains("target dir with space"));
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

    // Without a build target cargo cannot parse the manifest, so cleaning fails as well.
    let clean_cmd = Commands::Clean(generate_test_opts(&example_project));
    clean_cmd.run().unwrap_err();

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

#[test]
fn test_print_execution_time() {
    executor::print_execution_time(std::time::Duration::new(3665, 0)); // 1h 1m 5s
    executor::print_execution_time(std::time::Duration::new(125, 0)); // 2m 5s
    executor::print_execution_time(std::time::Duration::new(45, 0)); // 45s
}
