use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("askai").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("AI-powered terminal automation"))
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("askai").unwrap();
    cmd.arg("--version")
        .assert()
        .success();
}

#[test]
fn test_missing_prompt() {
    let mut cmd = Command::cargo_bin("askai").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required arguments were not provided"));
}

#[test]
fn test_dry_run_mode() {
    // Gemini CLI가 설치되어 있어야 하므로 skip 조건 추가
    if std::process::Command::new("which")
        .arg("gemini")
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
    {
        println!("Skipping test: Gemini CLI not installed");
        return;
    }

    let mut cmd = Command::cargo_bin("askai").unwrap();
    cmd.arg("현재 시간 출력")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("생성된 명령어:"))
        .stdout(predicate::str::contains("명령어만 출력합니다"));
}

#[test]
fn test_debug_flag() {
    // Gemini CLI가 설치되어 있어야 하므로 skip 조건 추가
    if std::process::Command::new("which")
        .arg("gemini")
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
    {
        println!("Skipping test: Gemini CLI not installed");
        return;
    }

    let mut cmd = Command::cargo_bin("askai").unwrap();
    cmd.arg("hello")
        .arg("--debug")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("DEBUG:"));
}
