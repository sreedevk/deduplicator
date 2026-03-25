mod helpers;
use helpers::FixtureDir;
use predicates::prelude::*;

#[test]
fn help_flag_prints_usage() {
    let mut cmd = assert_cmd::Command::cargo_bin("deduplicator").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn version_flag_prints_version() {
    let mut cmd = assert_cmd::Command::cargo_bin("deduplicator").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("deduplicator"));
}

#[test]
fn empty_directory_reports_no_duplicates() {
    let fixture = FixtureDir::new();
    fixture
        .cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicates found"));
}

#[test]
fn single_file_reports_no_duplicates() {
    let fixture = FixtureDir::new();
    fixture.write_file("only.txt", b"hello world");
    fixture
        .cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicates found"));
}

#[test]
fn unique_files_report_no_duplicates() {
    let fixture = FixtureDir::new();
    fixture.write_file("a.txt", b"content one");
    fixture.write_file("b.txt", b"content two");
    fixture.write_file("c.txt", b"content three");
    fixture
        .cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicates found"));
}
