use anyhow::Result;

#[test]
fn anzoth_version_reports_release_version() -> Result<()> {
    let mut cmd = assert_cmd::Command::new(codex_utils_cargo_bin::cargo_bin("anzoth")?);

    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("Anzoth CLI 1.0.5"));

    Ok(())
}
