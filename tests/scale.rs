mod helpers;
use helpers::FixtureDir;

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
    fn write_random_file(&self, name: &str, size: usize) -> std::path::PathBuf;
}

impl FixtureDirExt for FixtureDir {
    fn write_random_file(&self, name: &str, size: usize) -> std::path::PathBuf {
        let content = generate_random_bytes(size);
        self.write_file(name, &content)
    }
}

/// Ported from `rake benchmark:few_large_files`.
/// Creates the same file mix: same-size pairs, different-size pairs,
/// identical-content pairs, different-content-same-size pairs.
/// Verifies the binary completes and reports the correct duplicate groups.
#[test]
fn few_large_files_correctness() {
    let fixture = FixtureDir::new();
    let block = 4096usize;

    // Two files, same size, random content (not duplicates)
    // NOTE: original rake used 100_000 blocks (~390MB each, ~2.3GB total).
    // Reduced to 10_000 blocks (~39MB each) for CI speed/disk.
    fixture.write_random_file("fwss_0.bin", block * 10_000);
    fixture.write_random_file("fwss_1.bin", block * 10_000);

    // Two files, different sizes (not duplicates)
    fixture.write_random_file("fwds_0.bin", block * 5_000);
    fixture.write_random_file("fwds_1.bin", block * 8_000);

    // Two files, same content (DUPLICATES)
    let identical_content = vec![0u8; block * 10_000];
    fixture.write_file("fwscas_0.bin", &identical_content);
    fixture.write_file("fwscas_1.bin", &identical_content);

    // Two files, different content, same size (not duplicates)
    fixture.write_random_file("fwdcbss_0.bin", block * 10_000);
    fixture.write_random_file("fwdcbss_1.bin", block * 10_000);

    let output = fixture.cmd().arg("--strict").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "binary exited with error");

    // The identical-content pair must be reported
    assert!(
        stdout.contains("fwscas_0.bin") && stdout.contains("fwscas_1.bin"),
        "Should find the identical-content pair as duplicates. Got:\n{stdout}"
    );
}

/// Ported from `rake benchmark:many_small_files`.
/// Creates 4x100 files (reduced from 4x1000 for CI speed).
/// Verifies the binary handles high file counts and correctly
/// identifies the identical-content group.
#[test]
fn many_small_files_correctness() {
    let fixture = FixtureDir::new();
    let block = 4096usize;

    // 100 files, same size, random content
    for i in 0..100 {
        fixture.write_random_file(&format!("fwss_{i}.bin"), block * 1000);
    }

    // 100 files, different sizes, random content
    for i in 0..100 {
        let size = block * ((i % 100) + 1);
        fixture.write_random_file(&format!("fwds_{i}.bin"), size);
    }

    // 100 files, identical content (ALL duplicates of each other)
    let identical = vec![0u8; block * 1000];
    for i in 0..100 {
        fixture.write_file(&format!("fwscas_{i}.bin"), &identical);
    }

    // 100 files, different content, same size
    for i in 0..100 {
        fixture.write_random_file(&format!("fwdcbss_{i}.bin"), block * 1000);
    }

    let output = fixture.cmd().arg("--strict").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "binary exited with error");

    // All 100 identical-content files should appear in a single group
    assert!(
        stdout.contains("fwscas_0.bin") && stdout.contains("fwscas_99.bin"),
        "Should find the identical-content group. Got:\n{stdout}"
    );
}

/// Mixed scenario: duplicates scattered across subdirectories with
/// various sizes and types.
#[test]
fn mixed_duplicates_across_subdirectories() {
    let fixture = FixtureDir::new();

    let photo_content = generate_random_bytes(50_000);
    fixture.write_file("photos/vacation/img001.jpg", &photo_content);
    fixture.write_file("photos/backup/img001_copy.jpg", &photo_content);
    fixture.write_file("photos/backup/img001_copy2.jpg", &photo_content);

    let doc_content = generate_random_bytes(10_000);
    fixture.write_file("docs/report.pdf", &doc_content);
    fixture.write_file("docs/old/report_backup.pdf", &doc_content);

    // Unique files that should not appear
    fixture.write_random_file("docs/notes.txt", 5_000);
    fixture.write_random_file("photos/unique.png", 60_000);

    let output = fixture.cmd().arg("--strict").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("img001.jpg") && stdout.contains("img001_copy.jpg"));
    assert!(stdout.contains("report.pdf") && stdout.contains("report_backup.pdf"));
    assert!(!stdout.contains("notes.txt"));
    assert!(!stdout.contains("unique.png"));
}
