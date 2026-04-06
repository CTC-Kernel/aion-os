//! CLI integration tests.

use std::process::Command;

fn rewind_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rewind"))
}

#[test]
fn cli_help_exits_zero() {
    let output = rewind_bin().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Information is Sacred"));
}

#[test]
fn cli_example_exits_zero() {
    let output = rewind_bin().arg("example").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Garbage-free: true"));
}

#[test]
fn cli_run_demo_file() {
    let output = rewind_bin()
        .args(["run", "examples/demo.rev"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("FORWARD"));
    assert!(stdout.contains("BACKWARD"));
    assert!(stdout.contains("Garbage-free: true"));
}

#[test]
fn cli_analyze_demo_file() {
    let output = rewind_bin()
        .args(["analyze", "examples/demo.rev"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("5 ops"));
    assert!(stdout.contains("3 registers"));
}

#[test]
fn cli_run_nonexistent_file() {
    let output = rewind_bin()
        .args(["run", "nonexistent.rev"])
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn cli_unknown_command() {
    let output = rewind_bin().arg("foobar").output().unwrap();
    assert!(!output.status.success());
}
