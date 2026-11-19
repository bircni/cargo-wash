//! End-to-end tests running the `cargo-wash` binary against temporary projects.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

/// Create a minimal cargo project with a `target` folder of `target_bytes` bytes.
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

fn wash<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    Command::new(env!("CARGO_BIN_EXE_cargo-wash"))
        .args(args)
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Everything the command printed, log messages are split between both streams
fn printed(output: &Output) -> String {
    stdout(output) + &stderr(output)
}

#[track_caller]
fn assert_success(output: &Output) {
    assert!(output.status.success(), "{}", printed(output));
}

#[track_caller]
fn assert_failure(output: &Output) {
    assert_eq!(output.status.code(), Some(1), "{}", printed(output));
}

fn path_arg(path: &Path) -> &str {
    path.to_str().unwrap()
}

#[test]
fn stats_lists_projects_with_their_size() {
    let tmp_dir = tempfile::tempdir().unwrap();
    create_project(tmp_dir.path(), "big", 3 * 1024);
    create_project(tmp_dir.path(), "empty", 0);

    let output = wash(["stats", "--path", path_arg(tmp_dir.path())]);
    assert_success(&output);
    let table = stdout(&output);
    assert!(table.contains("big"), "{table}");
    assert!(table.contains("empty"), "{table}");
    assert!(table.contains("3.00 KB"), "{table}");
    assert!(table.contains("2 projects"), "{table}");
}

#[test]
fn works_as_a_cargo_subcommand() {
    let tmp_dir = tempfile::tempdir().unwrap();
    create_project(tmp_dir.path(), "wash", 0);

    let output = wash([
        "wash",
        "stats",
        "--path",
        path_arg(&tmp_dir.path().join("wash")),
    ]);
    assert_success(&output);
    assert!(
        stdout(&output).contains("1 projects"),
        "{}",
        stdout(&output)
    );
}

#[test]
fn fails_for_a_missing_path() {
    for command in ["stats", "clean", "build"] {
        let output = wash([command, "--path", "/not_existing"]);
        assert_failure(&output);
        assert!(
            stderr(&output).contains("/not_existing"),
            "{}",
            stderr(&output)
        );
    }
}

#[test]
fn clean_removes_target_and_respects_exclude() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let cleaned = create_project(tmp_dir.path(), "cleaned", 2048);
    let kept = create_project(tmp_dir.path(), "kept", 2048);

    let output = wash([
        "clean",
        "--path",
        path_arg(tmp_dir.path()),
        "--exclude",
        "kept",
    ]);
    assert_success(&output);
    assert!(!cleaned.join("target").exists());
    assert!(kept.join("target").exists());
    assert!(
        printed(&output).contains("Cleaned 2.00 KB (1 projects)"),
        "{}",
        printed(&output)
    );
}

#[test]
fn clean_fails_when_a_project_cannot_be_cleaned() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let broken = create_project(tmp_dir.path(), "broken", 0);
    fs::write(broken.join("Cargo.toml"), "not a manifest").unwrap();
    let fine = create_project(tmp_dir.path(), "fine", 1024);

    let output = wash(["clean", "--path", path_arg(tmp_dir.path())]);
    assert_failure(&output);
    assert!(
        !fine.join("target").exists(),
        "other projects must still be cleaned"
    );
    assert!(
        stderr(&output).contains("failed to clean: broken"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn runs_cargo_commands_in_every_project() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let project = create_project(tmp_dir.path(), "example", 0);

    for command in ["check", "build", "test", "doc", "run", "bench", "update"] {
        let output = wash([command, "--path", path_arg(tmp_dir.path())]);
        assert_success(&output);
    }
    assert!(project.join("target").join("debug").exists());
    assert!(project.join("Cargo.lock").exists());
}

#[test]
fn forwards_arguments_and_writes_project_logs() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let project = create_project(tmp_dir.path(), "example", 0);
    let target_dir = tmp_dir.path().join("target dir with space");
    let log_dir = tmp_dir.path().join("logs");

    for command in ["build", "clean"] {
        let output = wash([
            command,
            "--path",
            path_arg(&project),
            "--log-dir",
            path_arg(&log_dir),
            "--",
            "--target-dir",
            path_arg(&target_dir),
        ]);
        assert_success(&output);
    }

    assert!(!project.join("target").exists());
    let build_log = fs::read_to_string(log_dir.join("example-build.log")).unwrap();
    assert!(build_log.contains("command: cargo build"), "{build_log}");
    assert!(build_log.contains("status:"), "{build_log}");
    assert!(build_log.contains("target dir with space"), "{build_log}");
    assert!(log_dir.join("example-clean.log").exists());
}

#[test]
fn reports_failing_projects_with_their_output() {
    let tmp_dir = tempfile::tempdir().unwrap();
    create_project(tmp_dir.path(), "working", 0);
    let broken = create_project(tmp_dir.path(), "broken", 0);
    fs::write(
        broken.join("src").join("main.rs"),
        "fn main() { not_defined() }\n",
    )
    .unwrap();

    // `cargo doc` is not part of this list as rustdoc does not check function bodies.
    for command in ["check", "build", "test", "run", "bench"] {
        let output = wash([command, "--path", path_arg(tmp_dir.path())]);
        assert_failure(&output);
        let stderr = stderr(&output);
        assert!(
            stderr.contains("Some projects (1) failed: broken"),
            "{stderr}"
        );
        assert!(
            stderr.contains("not_defined"),
            "compiler output is missing: {stderr}"
        );
    }
}

#[test]
fn parallel_mode_runs_all_projects() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let first = create_project(tmp_dir.path(), "first", 0);
    let second = create_project(tmp_dir.path(), "second", 0);

    let output = wash(["check", "--parallel", "--path", path_arg(tmp_dir.path())]);
    assert_success(&output);
    assert!(first.join("target").exists());
    assert!(second.join("target").exists());
}
