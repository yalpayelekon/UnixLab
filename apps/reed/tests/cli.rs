//! Black-box tests for Reed's command-line contract.

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
        let path = std::env::temp_dir().join(format!("reed-test-{}-{id}.txt", std::process::id()));
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
    Command::new(env!("CARGO_BIN_EXE_reed"))
        .args(arguments)
        .output()
        .expect("reed process should run")
}

fn run_with_stdin(arguments: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_reed"))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("reed process should spawn");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(stdin.as_bytes())
        .expect("write stdin");
    child
        .wait_with_output()
        .expect("reed process should finish")
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

#[test]
fn substitutes_and_returns_zero() {
    let file = TestFile::text("cat\ncot\n");
    let output = run(&["s/c.t/dog/", file.path().to_str().expect("utf-8 path")]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_text(&output), "dog\ndog\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn global_substitute() {
    let file = TestFile::text("a a a\n");
    let output = run(&["s/a/b/g", file.path().to_str().expect("utf-8 path")]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_text(&output), "b b b\n");
}

#[test]
fn quiet_mode_prints_with_flag() {
    let file = TestFile::text("keep\nchange\n");
    let output = run(&[
        "-n",
        "s/change/done/p",
        file.path().to_str().expect("utf-8 path"),
    ]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_text(&output), "done\n");
}

#[test]
fn deletes_by_regex_address() {
    let file = TestFile::text("one\nskip\ntwo\n");
    let output = run(&["/skip/d", file.path().to_str().expect("utf-8 path")]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_text(&output), "one\ntwo\n");
}

#[test]
fn deletes_line_range() {
    let file = TestFile::text("a\nb\nc\nd\n");
    let output = run(&["2,3d", file.path().to_str().expect("utf-8 path")]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_text(&output), "a\nd\n");
}

#[test]
fn last_line_address() {
    let file = TestFile::text("one\ntwo\nthree\n");
    let output = run(&[r"$s/$/!/", file.path().to_str().expect("utf-8 path")]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_text(&output), "one\ntwo\nthree!\n");
}

#[test]
fn multiple_e_scripts() {
    let file = TestFile::text("ab\n");
    let output = run(&[
        "-e",
        "s/a/A/",
        "-e",
        "s/b/B/",
        file.path().to_str().expect("utf-8 path"),
    ]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_text(&output), "AB\n");
}

#[test]
fn reads_stdin_when_no_files() {
    let output = run_with_stdin(&["s/foo/bar/"], "foo\n");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_text(&output), "bar\n");
}

#[test]
fn processes_multiple_files_with_continuing_line_numbers() {
    let first = TestFile::text("a\nb\n");
    let second = TestFile::text("c\nd\n");
    let output = run(&[
        "2,3d",
        first.path().to_str().expect("utf-8 path"),
        second.path().to_str().expect("utf-8 path"),
    ]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout_text(&output), "a\nd\n");
}

#[test]
fn rejects_invalid_script() {
    let file = TestFile::text("a\n");
    let output = run(&["z", file.path().to_str().expect("utf-8 path")]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout_text(&output).is_empty());
    assert!(stderr_text(&output).starts_with("reed: invalid script:"));
}

#[test]
fn rejects_missing_file() {
    let output = run(&["s/a/b/", "missing-reed-input-file.txt"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout_text(&output).is_empty());
    assert!(stderr_text(&output).contains("cannot open"));
}

#[test]
fn rejects_invalid_utf8() {
    let file = TestFile::bytes(&[0xff, b'\n']);
    let output = run(&["s/a/b/", file.path().to_str().expect("utf-8 path")]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout_text(&output).is_empty());
    assert!(stderr_text(&output).contains("UTF-8"));
}

#[test]
fn usage_when_script_missing() {
    let output = run(&[]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr_text(&output).contains("usage: reed"));
}
