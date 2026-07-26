use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;

use crate::fileinfo::FileInfo;
use crate::params::Params;
use crate::processor;
use crate::scanner::Scanner;

pub struct DuplicateGroup {
    pub hash: u128,
    pub files: Vec<FileInfo>,
}

pub struct DedupReport {
    pub groups: Vec<DuplicateGroup>,
    pub max_path_len: usize,
}

pub(crate) fn spinner(enabled: bool, message: &'static str) -> ProgressBar {
    let bar = if enabled {
        ProgressBar::new_spinner()
    } else {
        ProgressBar::hidden()
    };

    let style = ProgressStyle::with_template("[{elapsed_precise}] {pos:>7} {msg}")
        .expect("valid progress template");

    bar.set_style(style);
    bar.enable_steady_tick(Duration::from_millis(50));
    bar.set_message(message);
    bar
}

pub fn run(params: &Params) -> Result<DedupReport> {
    let seed: i64 = rand::rng().random();

    let files = Scanner::new(params)?.scan()?;
    let size_groups = processor::group_by_size(files, params.progress);

    let candidates: Vec<FileInfo> = size_groups
        .into_iter()
        .filter(|group| group.len() > 1)
        .flatten()
        .collect();

    let max_path_len = candidates
        .iter()
        .map(|file| file.path.to_string_lossy().graphemes(true).count())
        .max()
        .unwrap_or(0);

    let groups = processor::group_by_hash(candidates, params.strict, seed, params.progress);

    Ok(DedupReport {
        groups,
        max_path_len,
    })
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::params::Params;
    use anyhow::Result;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn run_finds_a_single_group_of_identical_files() -> Result<()> {
        let root = TempDir::new()?;

        let duplicate = vec![7u8; 200_000];
        for name in ["a.bin", "b.bin"] {
            let mut file = File::create_new(root.path().join(name))?;
            file.write_all(&duplicate)?;
        }

        let mut unique = File::create_new(root.path().join("c.bin"))?;
        unique.write_all(&vec![9u8; 100_000])?;

        let params = Params {
            dir: Some(root.path().into()),
            ..Default::default()
        };

        let report = run(&params)?;

        assert_eq!(report.groups.len(), 1);
        assert_eq!(report.groups[0].files.len(), 2);
        Ok(())
    }
}
