use clap::ColorChoice;
use clap::Command;

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
