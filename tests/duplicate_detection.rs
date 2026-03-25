mod helpers;
use helpers::FixtureDir;
use predicates::prelude::*;

fn generate_random_bytes(size: usize) -> Vec<u8> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let call_id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut state: u64 = call_id.wrapping_mul(0x517cc1b727220a95) ^ (size as u64);
    let remainder = size % 8;
    let mut buf = vec![0u8; size];
    for chunk in buf[..size - remainder].chunks_exact_mut(8) {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        chunk.copy_from_slice(&state.to_ne_bytes());
    }
    if remainder > 0 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        buf[size - remainder..].copy_from_slice(&state.to_ne_bytes()[..remainder]);
    }
    buf
}

trait FixtureDirExt {
    fn write_empty_file(&self, name: &str) -> std::path::PathBuf;
    fn write_null_file(&self, name: &str, size: usize) -> std::path::PathBuf;
}

impl FixtureDirExt for FixtureDir {
    fn write_empty_file(&self, name: &str) -> std::path::PathBuf {
        self.write_file(name, &[])
    }

    fn write_null_file(&self, name: &str, size: usize) -> std::path::PathBuf {
        self.write_file(name, &vec![0u8; size])
    }
}

#[test]
fn finds_two_identical_files() {
    let fixture = FixtureDir::new();
    fixture.write_file("a.txt", b"duplicate content here");
    fixture.write_file("b.txt", b"duplicate content here");

    fixture
        .cmd()
        .assert()
        .success()
        .stdout(
            predicate::str::contains("a.txt")
                .and(predicate::str::contains("b.txt")),
        );
}

#[test]
fn finds_multiple_duplicate_groups() {
    let fixture = FixtureDir::new();
    fixture.write_file("a1.txt", b"group one content!");
    fixture.write_file("a2.txt", b"group one content!");
    fixture.write_file("b1.txt", b"group two content!");
    fixture.write_file("b2.txt", b"group two content!");
    fixture.write_file("unique.txt", b"i am unique file");

    let output = fixture.cmd().output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Both groups are present; unique file is not listed in any group
    assert!(stdout.contains("a1.txt") && stdout.contains("a2.txt"));
    assert!(stdout.contains("b1.txt") && stdout.contains("b2.txt"));

    // The tree character appears (groups are rendered)
    assert!(stdout.contains("\u{251c}\u{2500}") || stdout.contains("\u{2514}\u{2500}"));
}

#[test]
fn does_not_group_same_size_different_content() {
    let fixture = FixtureDir::new();
    // Same size (22 bytes each) but different content
    fixture.write_file("a.txt", b"same length content AA");
    fixture.write_file("b.txt", b"same length content BB");

    fixture
        .cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicates found"));
}

#[test]
fn strict_mode_differentiates_files_with_identical_first_16kb() {
    let fixture = FixtureDir::new();
    let shared_prefix = generate_random_bytes(16384);

    let mut content_a = shared_prefix.clone();
    content_a.extend_from_slice(&generate_random_bytes(32768));

    let mut content_b = shared_prefix;
    content_b.extend_from_slice(&generate_random_bytes(32768));

    fixture.write_file("a.bin", &content_a);
    fixture.write_file("b.bin", &content_b);

    // Strict mode: full hash — should NOT report as duplicates
    fixture
        .cmd()
        .arg("--strict")
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicates found"));
}

#[test]
fn fast_mode_groups_files_with_identical_first_16kb() {
    let fixture = FixtureDir::new();
    let shared_prefix = generate_random_bytes(16384);

    // Both files share first 16KB but differ after — must also be same size
    // for the size-wise pass to group them
    let suffix_len = 32768;
    let mut content_a = shared_prefix.clone();
    content_a.extend_from_slice(&generate_random_bytes(suffix_len));

    let mut content_b = shared_prefix;
    content_b.extend_from_slice(&generate_random_bytes(suffix_len));

    fixture.write_file("a.bin", &content_a);
    fixture.write_file("b.bin", &content_b);

    // Fast mode (default): only hashes first 16KB — should report as duplicates
    let output = fixture.cmd().output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("a.bin") && stdout.contains("b.bin"),
        "Fast mode should group files with identical first 16KB. Got:\n{stdout}"
    );
}

#[test]
fn empty_file_and_null_byte_file_are_not_grouped() {
    let fixture = FixtureDir::new();
    fixture.write_empty_file("empty.bin");
    fixture.write_null_file("nulls.bin", 4096);

    // --min-size 0b is required: the default (1b) filters out the empty file,
    // which would make this test pass for the wrong reason.
    fixture
        .cmd()
        .args(["--strict", "--min-size", "0b"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicates found"));
}

#[test]
fn multiple_identical_empty_files_are_grouped() {
    let fixture = FixtureDir::new();
    fixture.write_empty_file("a.bin");
    fixture.write_empty_file("b.bin");

    // --min-size 0b is required: the default (1b) filters out 0-byte files.
    let output = fixture
        .cmd()
        .args(["--min-size", "0b"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("a.bin") && stdout.contains("b.bin"),
        "Multiple empty files should be grouped as duplicates. Got:\n{stdout}"
    );
}

#[test]
fn finds_duplicates_in_subdirectories() {
    let fixture = FixtureDir::new();
    fixture.write_file("top/a.txt", b"nested duplicate content");
    fixture.write_file("deep/nested/b.txt", b"nested duplicate content");

    let output = fixture.cmd().output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("a.txt") && stdout.contains("b.txt"),
        "Should find duplicates across subdirectories. Got:\n{stdout}"
    );
}
