//! Black-box tests for Resolver's command-line contract.

use std::process::{Command, Output};

fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_resolver"))
        .args(arguments)
        .output()
        .expect("resolver process should run")
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

#[test]
fn resolves_localhost() {
    let output = run(&["localhost"]);

    assert_eq!(output.status.code(), Some(0));
    let stdout = stdout_text(&output);
    assert!(
        stdout
            .lines()
            .any(|line| line == "127.0.0.1" || line == "::1"),
        "expected loopback address in {stdout:?}"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn resolves_ipv4_literal() {
    let output = run(&["127.0.0.1"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_text(&output), "127.0.0.1\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn invalid_name_fails() {
    let output = run(&["something.invalid"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(stderr_text(&output).starts_with("resolver: "));
}

#[test]
fn missing_name_is_usage() {
    let output = run(&[]);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr_text(&output).contains("usage: resolver"));
}

#[test]
fn extra_argument_is_usage() {
    let output = run(&["localhost", "extra"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr_text(&output).contains("usage: resolver"));
}
