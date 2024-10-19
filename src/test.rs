use std::path::PathBuf;

use anyhow::Context;
use clap::ColorChoice;
use clap::Command;

use crate::cli;
use crate::data;
use crate::utils;

fn snapshot_test_cli_command(app: Command, cmd_name: &str) -> anyhow::Result<()> {
    let mut app = app
        .color(ColorChoice::Never)
        .version("0.0.0")
        .long_version("0.0.0");

    let mut buffer = Vec::new();
    app.write_long_help(&mut buffer)?;
    let help_text = std::str::from_utf8(&buffer)?;

    if insta::_macro_support::assert_snapshot(
        cmd_name.into(),
        help_text,
        env!("CARGO_MANIFEST_DIR"),
        "cli-cmd",
        module_path!(),
        file!(),
        line!(),
        "help_text",
    )
    .is_err()
    {
        anyhow::bail!("Snapshot test failed for command: {}", cmd_name);
    }

    for app in app.get_subcommands() {
        if app.get_name() == "help" {
            continue;
        }

        snapshot_test_cli_command(app.clone(), &format!("{cmd_name}-{}", app.get_name()))?;
    }
    Ok(())
}

#[allow(clippy::expect_used)]
#[test]
fn cli_snapshot() {
    use clap::CommandFactory;

    insta::with_settings!({
        snapshot_path => "../tests/snapshots",
    }, {
        snapshot_test_cli_command(
            super::Commands::command().name("cargo_wash"),
            "cargo_wash",
        ).expect("Failed to run snapshot test");
    });
}

#[test]
fn test_clean_path() {
    assert!(utils::clean_path(Some(PathBuf::from("/"))).is_ok());
    assert!(utils::clean_path(Some(PathBuf::from("."))).is_ok());
    assert!(utils::clean_path(Some(PathBuf::from(".."))).is_ok());
    assert!(utils::clean_path(Some(PathBuf::from("test"))).is_ok());
}

#[test]
fn test_get_folder_size() -> anyhow::Result<()> {
    let size = utils::get_folder_size("src")?;
    assert!(size > 0);
    Ok(())
}

#[test]
fn test_get_project() {
    assert_eq!(utils::get_language("."), cli::Language::Rust);
    assert_eq!(utils::get_language("src"), cli::Language::Other);
}

#[test]
fn test_check_project() -> anyhow::Result<()> {
    let opts = cli::Opts::default();
    let res = opts
        .check_project(&PathBuf::from("../cargo-wash"), Some(cli::Language::Rust))?
        .context("Project was `None`")?;
    assert!(res.size > data::Size::to_size(0));
    assert!(res.language == cli::Language::Rust);
    assert!(res.name == "cargo-wash");
    assert!(res.path == PathBuf::from("../cargo-wash"));
    Ok(())
}

#[cfg(not(target_os = "windows"))] //Windows does not allow deleting the current executable
#[test]
fn test_run_clean() -> anyhow::Result<()> {
    let opts = cli::Opts::default();
    let (projects, _) = opts.check_args()?;
    utils::run_clean(&projects, false)?;
    Ok(())
}
