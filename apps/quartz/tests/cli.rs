//! Black-box tests for Quartz's command-line contract.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FILE_ID: AtomicU64 = AtomicU64::new(0);

struct TestFile {
    path: PathBuf,
}

impl TestFile {
    fn text(contents: &str) -> Self {
        Self::bytes(contents.as_bytes())
    }

    fn bytes(contents: &[u8]) -> Self {
        let id = NEXT_FILE_ID.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("quartz-test-{}-{id}.json", std::process::id()));
        fs::write(&path, contents).expect("test file should be written");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_quartz"))
        .args(arguments)
        .output()
        .expect("quartz process should run")
}

fn run_with_stdin(arguments: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_quartz"))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("quartz process should spawn");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(stdin.as_bytes())
        .expect("write stdin");
    child
        .wait_with_output()
        .expect("quartz process should finish")
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

#[test]
fn identity_pretty_prints_file() {
    let file = TestFile::text(r#"{"a":1}"#);
    let output = run(&[".", file.path().to_str().expect("utf-8 path")]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_text(&output), "{\n  \"a\": 1\n}\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn default_filter_on_file() {
    let file = TestFile::text(r#"{"a":1}"#);
    let output = run(&[file.path().to_str().expect("utf-8 path")]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_text(&output), "{\n  \"a\": 1\n}\n");
}

#[test]
fn nested_path_and_array_index() {
    let file = TestFile::text(r#"{"user":{"name":"ada","tags":["x","y"]}}"#);
    let path = file.path().to_str().expect("utf-8 path");

    let name = run(&[".user.name", path]);
    assert_eq!(name.status.code(), Some(0));
    assert_eq!(stdout_text(&name), "\"ada\"\n");

    let tag = run(&[".user.tags[1]", path]);
    assert_eq!(tag.status.code(), Some(0));
    assert_eq!(stdout_text(&tag), "\"y\"\n");
}

#[test]
fn missing_path_prints_null() {
    let file = TestFile::text(r#"{"a":1}"#);
    let output = run(&[".missing", file.path().to_str().expect("utf-8 path")]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_text(&output), "null\n");
}

#[test]
fn compact_flag() {
    let file = TestFile::text(r#"{"a":1,"b":2}"#);
    let output = run(&["-c", ".", file.path().to_str().expect("utf-8 path")]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_text(&output), "{\"a\":1,\"b\":2}\n");
}

#[test]
fn reads_stdin() {
    let output = run_with_stdin(&[".value"], r#"{"value":42}"#);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_text(&output), "42\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn processes_multiple_files_in_order() {
    let first = TestFile::text(r#"{"n":1}"#);
    let second = TestFile::text(r#"{"n":2}"#);
    let output = run(&[
        "-c",
        ".n",
        first.path().to_str().expect("utf-8 path"),
        second.path().to_str().expect("utf-8 path"),
    ]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_text(&output), "1\n2\n");
}

#[test]
fn invalid_json_fails() {
    let file = TestFile::text("{");
    let output = run(&[file.path().to_str().expect("utf-8 path")]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(stderr_text(&output).starts_with("quartz: cannot parse"));
}

#[test]
fn invalid_filter_fails() {
    let file = TestFile::text("1");
    let output = run(&[".foo.", file.path().to_str().expect("utf-8 path")]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(stderr_text(&output).contains("invalid filter"));
}

#[test]
fn missing_file_fails() {
    let output = run(&[".", "definitely-missing-quartz-input.json"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(stderr_text(&output).contains("cannot open"));
}

#[test]
fn unknown_option_is_usage() {
    let output = run(&["-x"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr_text(&output).contains("usage: quartz"));
}
