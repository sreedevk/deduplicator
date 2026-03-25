mod helpers;
use helpers::FixtureDir;

// --- Type include filter (-t) ---

#[test]
fn include_type_filter_only_scans_specified_extensions() {
    let fixture = FixtureDir::new();
    fixture.write_file("a.txt", b"duplicate");
    fixture.write_file("b.txt", b"duplicate");
    fixture.write_file("c.csv", b"duplicate");
    fixture.write_file("d.csv", b"duplicate");

    // Only scan .txt files
    let output = fixture.cmd().arg("-t").arg("txt").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("a.txt") && stdout.contains("b.txt"));
    assert!(!stdout.contains("c.csv") && !stdout.contains("d.csv"));
}

// --- Type exclude filter (-T) ---

#[test]
fn exclude_type_filter_skips_specified_extensions() {
    let fixture = FixtureDir::new();
    fixture.write_file("a.txt", b"duplicate");
    fixture.write_file("b.txt", b"duplicate");
    fixture.write_file("c.csv", b"duplicate");
    fixture.write_file("d.csv", b"duplicate");

    // Exclude .csv files
    let output = fixture.cmd().arg("-T").arg("csv").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("a.txt") && stdout.contains("b.txt"));
    assert!(!stdout.contains("c.csv") && !stdout.contains("d.csv"));
}

// --- Combined include + exclude ---

#[test]
fn combined_include_and_exclude_filters() {
    let fixture = FixtureDir::new();
    fixture.write_file("a.js", b"duplicate!");
    fixture.write_file("b.js", b"duplicate!");
    fixture.write_file("c.csv", b"duplicate!");
    fixture.write_file("d.csv", b"duplicate!");
    fixture.write_file("e.rs", b"duplicate!");
    fixture.write_file("f.rs", b"duplicate!");

    // Include js,csv,rs but exclude csv
    let output = fixture
        .cmd()
        .args(["-t", "js,csv,rs", "-T", "csv"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("a.js") && stdout.contains("b.js"));
    assert!(stdout.contains("e.rs") && stdout.contains("f.rs"));
    assert!(!stdout.contains("c.csv") && !stdout.contains("d.csv"));
}

// --- Min-size filter (-m) ---

#[test]
fn min_size_filter_skips_small_files() {
    let fixture = FixtureDir::new();
    // Small duplicates (9 bytes)
    fixture.write_file("small_a.txt", b"tiny data");
    fixture.write_file("small_b.txt", b"tiny data");
    // Large duplicates (10KB)
    let large = vec![42u8; 10240];
    fixture.write_file("big_a.bin", &large);
    fixture.write_file("big_b.bin", &large);

    let output = fixture
        .cmd()
        .args(["--min-size", "1K"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("big_a.bin") && stdout.contains("big_b.bin"));
    assert!(!stdout.contains("small_a.txt") && !stdout.contains("small_b.txt"));
}

// --- Max-depth filter (-D) ---

#[test]
fn max_depth_limits_to_root_level_files() {
    let fixture = FixtureDir::new();
    fixture.write_file("root_a.txt", b"dup content here");
    fixture.write_file("root_b.txt", b"dup content here");
    fixture.write_file("sub/deep_a.txt", b"dup content here");

    // globwalk depth: 0 = root dir entry itself, 1 = files in root.
    // --max-depth 1 means "root-level files only, no subdirectories".
    let output = fixture
        .cmd()
        .args(["--max-depth", "1"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Root-level duplicates should be found
    assert!(stdout.contains("root_a.txt") && stdout.contains("root_b.txt"));
    // Subdirectory file should not appear
    assert!(!stdout.contains("deep_a.txt"));
}

// --- Min-depth filter (-d) ---

#[test]
fn min_depth_excludes_shallow_files() {
    let fixture = FixtureDir::new();
    fixture.write_file("root.txt", b"dup content here");
    fixture.write_file("sub/deep_a.txt", b"dup content here");
    fixture.write_file("sub/deep_b.txt", b"dup content here");

    // globwalk depth: 0 = root dir, 1 = files in root, 2 = files in subdirs.
    // --min-depth 2 excludes root-level files.
    let output = fixture
        .cmd()
        .args(["--min-depth", "2"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("deep_a.txt") && stdout.contains("deep_b.txt"));
    // Root-level file must NOT appear
    assert!(!stdout.contains("root.txt"),
        "root.txt should be excluded by --min-depth 2. Got:\n{stdout}");
}

// --- Progress flag (-p) ---

#[test]
fn progress_flag_does_not_break_output() {
    let fixture = FixtureDir::new();
    fixture.write_file("a.txt", b"duplicate content");
    fixture.write_file("b.txt", b"duplicate content");

    // Just verify it doesn't crash and still finds duplicates
    let output = fixture.cmd().arg("--progress").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("a.txt") && stdout.contains("b.txt"));
}
