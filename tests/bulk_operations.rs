mod helpers;
use helpers::FixtureDir;
use predicates::prelude::*;
use std::time::{Duration, SystemTime};

trait FixtureDirExt {
    fn write_file_with_mtime(
        &self,
        name: &str,
        content: &[u8],
        mtime: SystemTime,
    ) -> std::path::PathBuf;
}

impl FixtureDirExt for FixtureDir {
    fn write_file_with_mtime(
        &self,
        name: &str,
        content: &[u8],
        mtime: SystemTime,
    ) -> std::path::PathBuf {
        let path = self.write_file(name, content);
        let ft = filetime::FileTime::from_system_time(mtime);
        filetime::set_file_mtime(&path, ft).expect("failed to set mtime");
        path
    }
}

// ===== Conflict tests =====

#[test]
fn keep_and_interactive_conflict() {
    let mut cmd = assert_cmd::Command::cargo_bin("deduplicator").unwrap();
    cmd.args(["--keep", "newest", "--interactive", "/tmp"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn delete_without_keep_errors() {
    let mut cmd = assert_cmd::Command::cargo_bin("deduplicator").unwrap();
    cmd.args(["--delete", "/tmp"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ===== Dry-run tests =====

#[test]
fn keep_newest_dry_run_shows_annotations() {
    let fixture = FixtureDir::new();
    let old_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    let new_time = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000);

    fixture.write_file_with_mtime("old.txt", b"duplicate content!", old_time);
    fixture.write_file_with_mtime("new.txt", b"duplicate content!", new_time);

    let output = fixture.cmd().args(["--keep", "newest"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("[KEEP]"), "Should show [KEEP] annotation. Got:\n{stdout}");
    assert!(stdout.contains("[DELETE]"), "Should show [DELETE] annotation. Got:\n{stdout}");
    assert!(stdout.contains("new.txt") && stdout.contains("old.txt"));
}

#[test]
fn keep_oldest_dry_run_shows_annotations() {
    let fixture = FixtureDir::new();
    let old_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    let new_time = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000);

    fixture.write_file_with_mtime("old.txt", b"duplicate content!", old_time);
    fixture.write_file_with_mtime("new.txt", b"duplicate content!", new_time);

    let output = fixture.cmd().args(["--keep", "oldest"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("[KEEP]"));
    assert!(stdout.contains("[DELETE]"));
}

#[test]
fn keep_newest_dry_run_does_not_delete_files() {
    let fixture = FixtureDir::new();
    let old_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    let new_time = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000);

    let old_path = fixture.write_file_with_mtime("old.txt", b"duplicate content!", old_time);
    let new_path = fixture.write_file_with_mtime("new.txt", b"duplicate content!", new_time);

    fixture.cmd().args(["--keep", "newest"]).assert().success();

    assert!(old_path.exists(), "old.txt should still exist after dry-run");
    assert!(new_path.exists(), "new.txt should still exist after dry-run");
}

#[test]
fn dry_run_with_multiple_groups() {
    let fixture = FixtureDir::new();
    let old_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    let new_time = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000);

    fixture.write_file_with_mtime("g1_old.txt", b"group one content!", old_time);
    fixture.write_file_with_mtime("g1_new.txt", b"group one content!", new_time);
    fixture.write_file_with_mtime("g2_old.txt", b"group two content!", old_time);
    fixture.write_file_with_mtime("g2_new.txt", b"group two content!", new_time);

    let output = fixture.cmd().args(["--keep", "newest"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("g1_old.txt") && stdout.contains("g1_new.txt"));
    assert!(stdout.contains("g2_old.txt") && stdout.contains("g2_new.txt"));
    assert!(stdout.matches("[KEEP]").count() >= 2, "Each group needs a [KEEP]");
    assert!(stdout.matches("[DELETE]").count() >= 2, "Each group needs a [DELETE]");
}

#[test]
fn keep_with_single_file_no_output() {
    let fixture = FixtureDir::new();
    fixture.write_file("only.txt", b"unique content");

    fixture
        .cmd()
        .args(["--keep", "newest"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicates found"));
}

#[test]
fn keep_newest_with_three_duplicates() {
    let fixture = FixtureDir::new();
    let t1 = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    let t2 = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000);
    let t3 = SystemTime::UNIX_EPOCH + Duration::from_secs(3_000_000);

    fixture.write_file_with_mtime("oldest.txt", b"triple duplicate!", t1);
    fixture.write_file_with_mtime("middle.txt", b"triple duplicate!", t2);
    fixture.write_file_with_mtime("newest.txt", b"triple duplicate!", t3);

    let output = fixture.cmd().args(["--keep", "newest"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(stdout.matches("[KEEP]").count(), 1, "Only one file should be kept");
    assert_eq!(stdout.matches("[DELETE]").count(), 2, "Two files should be marked for deletion");
}

// ===== Delete tests =====

#[test]
fn keep_newest_delete_removes_older_files() {
    let fixture = FixtureDir::new();
    let old_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    let new_time = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000);

    let old_path = fixture.write_file_with_mtime("old.txt", b"duplicate content!", old_time);
    let new_path = fixture.write_file_with_mtime("new.txt", b"duplicate content!", new_time);

    fixture
        .cmd()
        .args(["--keep", "newest", "--delete"])
        .assert()
        .success();

    assert!(!old_path.exists(), "old.txt should be deleted");
    assert!(new_path.exists(), "new.txt should be kept");
}

#[test]
fn keep_oldest_delete_removes_newer_files() {
    let fixture = FixtureDir::new();
    let old_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    let new_time = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000);

    let old_path = fixture.write_file_with_mtime("old.txt", b"duplicate content!", old_time);
    let new_path = fixture.write_file_with_mtime("new.txt", b"duplicate content!", new_time);

    fixture
        .cmd()
        .args(["--keep", "oldest", "--delete"])
        .assert()
        .success();

    assert!(old_path.exists(), "old.txt should be kept");
    assert!(!new_path.exists(), "new.txt should be deleted");
}

#[test]
fn delete_with_multiple_groups() {
    let fixture = FixtureDir::new();
    let old_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    let new_time = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000);

    let g1_old = fixture.write_file_with_mtime("g1_old.txt", b"group one content!", old_time);
    let g1_new = fixture.write_file_with_mtime("g1_new.txt", b"group one content!", new_time);
    let g2_old = fixture.write_file_with_mtime("g2_old.txt", b"group two content!", old_time);
    let g2_new = fixture.write_file_with_mtime("g2_new.txt", b"group two content!", new_time);

    fixture
        .cmd()
        .args(["--keep", "newest", "--delete"])
        .assert()
        .success();

    assert!(!g1_old.exists() && g1_new.exists(), "Group 1: old deleted, new kept");
    assert!(!g2_old.exists() && g2_new.exists(), "Group 2: old deleted, new kept");
}

#[test]
fn delete_prints_kept_and_deleted_paths() {
    let fixture = FixtureDir::new();
    let old_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    let new_time = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000);

    fixture.write_file_with_mtime("old.txt", b"duplicate content!", old_time);
    fixture.write_file_with_mtime("new.txt", b"duplicate content!", new_time);

    let output = fixture
        .cmd()
        .args(["--keep", "newest", "--delete"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("KEPT:"), "Should print KEPT line. Got:\n{stdout}");
    assert!(stdout.contains("DELETED:"), "Should print DELETED line. Got:\n{stdout}");
}

// ===== Tie-breaking tests =====

#[test]
fn keep_with_same_mtime_keeps_both() {
    let fixture = FixtureDir::new();
    let same_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);

    fixture.write_file_with_mtime("a.txt", b"duplicate content!", same_time);
    fixture.write_file_with_mtime("b.txt", b"duplicate content!", same_time);

    let output = fixture.cmd().args(["--keep", "newest"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(!stdout.contains("[DELETE]"),
        "No files should be marked for deletion when mtimes tie. Got:\n{stdout}");
    assert!(stdout.contains("[KEEP]"),
        "Should have at least one [KEEP]. Got:\n{stdout}");
    assert!(stdout.contains("[KEEP - same mtime]"),
        "Tied file should show [KEEP - same mtime]. Got:\n{stdout}");
}

#[test]
fn keep_with_partial_mtime_tie() {
    let fixture = FixtureDir::new();
    let old_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    let new_time = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000);

    fixture.write_file_with_mtime("old.txt", b"triple duplicate!", old_time);
    fixture.write_file_with_mtime("new_a.txt", b"triple duplicate!", new_time);
    fixture.write_file_with_mtime("new_b.txt", b"triple duplicate!", new_time);

    let output = fixture.cmd().args(["--keep", "newest"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(stdout.matches("[DELETE]").count(), 1,
        "Only the old file should be marked DELETE. Got:\n{stdout}");
    assert!(stdout.contains("[KEEP]"));
    assert!(stdout.contains("[KEEP - same mtime]"));
}

#[test]
fn keep_all_same_mtime_deletes_nothing() {
    let fixture = FixtureDir::new();
    let same_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);

    fixture.write_file_with_mtime("a.txt", b"all same time!", same_time);
    fixture.write_file_with_mtime("b.txt", b"all same time!", same_time);
    fixture.write_file_with_mtime("c.txt", b"all same time!", same_time);

    let output = fixture.cmd().args(["--keep", "newest"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(!stdout.contains("[DELETE]"),
        "Nothing should be deleted when all mtimes are the same. Got:\n{stdout}");
    assert_eq!(stdout.matches("[KEEP - same mtime]").count(), 2,
        "Two files should show [KEEP - same mtime]. Got:\n{stdout}");
}

#[test]
fn keep_partial_tie_delete_mode() {
    let fixture = FixtureDir::new();
    let old_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    let new_time = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000);

    let old_path = fixture.write_file_with_mtime("old.txt", b"triple duplicate!", old_time);
    let new_a_path = fixture.write_file_with_mtime("new_a.txt", b"triple duplicate!", new_time);
    let new_b_path = fixture.write_file_with_mtime("new_b.txt", b"triple duplicate!", new_time);

    let output = fixture
        .cmd()
        .args(["--keep", "newest", "--delete"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(!old_path.exists(), "old.txt should be deleted");
    assert!(new_a_path.exists(), "new_a.txt should be kept (tie)");
    assert!(new_b_path.exists(), "new_b.txt should be kept (tie)");
    assert!(stdout.contains("same mtime, skipped"),
        "Tied file should show skip reason. Got:\n{stdout}");
}
