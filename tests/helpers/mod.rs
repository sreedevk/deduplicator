use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;

/// A temporary directory with controlled file fixtures.
/// Dropped automatically after the test.
pub struct FixtureDir {
    pub dir: TempDir,
}

impl FixtureDir {
    pub fn new() -> Self {
        Self {
            dir: TempDir::new().expect("failed to create temp dir"),
        }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Write a file with exact content bytes.
    pub fn write_file(&self, name: &str, content: &[u8]) -> PathBuf {
        let path = self.dir.path().join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create parent dirs");
        }
        let mut f = File::create(&path).expect("failed to create fixture file");
        f.write_all(content).expect("failed to write fixture file");
        path
    }

    /// Build an assert_cmd::Command targeting the deduplicator binary,
    /// with this fixture's directory as the positional argument.
    pub fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("deduplicator").expect("binary not found");
        cmd.arg(self.path());
        cmd
    }
}
