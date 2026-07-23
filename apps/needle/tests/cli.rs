//! Black-box tests for Needle's command-line contract.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
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
            std::env::temp_dir().join(format!("needle-test-{}-{id}.txt", std::process::id()));
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
    Command::new(env!("CARGO_BIN_EXE_needle"))
        .args(arguments)
        .output()
        .expect("needle process should run")
}

fn run_with_file(pattern: &str, file: &TestFile) -> Output {
    Command::new(env!("CARGO_BIN_EXE_needle"))
        .arg(pattern)
        .arg(file.path())
        .output()
        .expect("needle process should run")
}

#[test]
fn prints_matching_lines_and_returns_zero() {
    let file = TestFile::text("ordinary\nerror: first\nwarning\nerror: second\n");

    let output = run_with_file(r"^error:", &file);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        "error: first\nerror: second\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn supports_unicode_patterns_and_text() {
    let file = TestFile::text("hello\nİstanbul\nκόσμος\n");

    let output = run_with_file(r"İstanbul|κόσμος", &file);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
        "İstanbul\nκόσμος\n"
    );
}

#[test]
fn returns_one_when_nothing_matches() {
    let file = TestFile::text("first\nsecond\n");

    let output = run_with_file("absent", &file);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn returns_two_for_invalid_pattern() {
    let file = TestFile::text("content\n");

    let output = run_with_file("(", &file);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .expect("stderr should be UTF-8")
            .contains("needle: invalid pattern:")
    );
}

#[test]
fn returns_two_for_missing_file() {
    let missing = std::env::temp_dir().join(format!(
        "needle-missing-{}.txt",
        NEXT_FILE_ID.fetch_add(1, Ordering::Relaxed)
    ));

    let output = Command::new(env!("CARGO_BIN_EXE_needle"))
        .arg("pattern")
        .arg(&missing)
        .output()
        .expect("needle process should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .expect("stderr should be UTF-8")
            .contains("needle: cannot open")
    );
}

#[test]
fn returns_two_for_invalid_utf8() {
    let file = TestFile::bytes(&[b'v', b'a', b'l', b'i', b'd', b'\n', 0xff, b'\n']);

    let output = run_with_file("valid", &file);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, b"valid\n");
    assert!(
        String::from_utf8(output.stderr)
            .expect("stderr should be UTF-8")
            .contains("as UTF-8 text")
    );
}

#[test]
fn returns_two_for_invalid_usage() {
    let output = run(&[]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "needle: usage: needle <pattern> <file>\n"
    );
}

#[test]
fn redirected_output_contains_no_color_codes() {
    let file = TestFile::text("find the needle\n");

    let output = run_with_file("needle", &file);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"find the needle\n");
    assert!(!output.stdout.contains(&0x1b));
}
