use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use unicode_segmentation::UnicodeSegmentation;

use crate::cache::{mtime_nanos, Cache, CACHE_TTL_DAYS};
use crate::fileinfo::FileInfo;
use crate::params::Params;
use crate::processor;
use crate::scanner::Scanner;

const HASH_SEED: i64 = 0x00DE_D0CA_C4E5_EED1;

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

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
    let seed = HASH_SEED;

    let cache_path = match params.no_cache {
        true => None,
        false => params.cache_file.clone().or_else(Cache::default_path),
    };

    let mut cache = match (params.no_cache, &cache_path) {
        (false, Some(path)) => Cache::load(path),
        _ => Cache::disabled(),
    };

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

    let hashed = processor::hash_candidates(candidates, params.strict, seed, params.progress, &cache);

    let now = now_secs();
    for (hash, file) in &hashed {
        let mtime = mtime_nanos(file.modified);
        cache.record(&file.path, file.size, mtime, params.strict, *hash, now);
    }

    let groups = processor::group_hashed(hashed);

    if let Some(path) = &cache_path {
        cache.save(path, CACHE_TTL_DAYS * 86_400, now);
    }

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

#[cfg(test)]
mod cache_tests {
    use super::run;
    use crate::params::Params;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn make_tree(root: &TempDir) {
        for name in ["a.bin", "b.bin"] {
            let mut f = File::create_new(root.path().join(name)).unwrap();
            f.write_all(&vec![7u8; 200_000]).unwrap();
        }
        let mut u = File::create_new(root.path().join("c.bin")).unwrap();
        u.write_all(&vec![9u8; 100_000]).unwrap();
    }

    #[test]
    fn run_twice_with_cache_is_identical_and_writes_the_file() {
        let root = TempDir::new().unwrap();
        make_tree(&root);
        let cache_file = root.path().join("dd.cache");
        let params = Params {
            dir: Some(root.path().into()),
            cache_file: Some(cache_file.clone()),
            ..Default::default()
        };

        let first = run(&params).unwrap();
        assert!(cache_file.exists());

        let second = run(&params).unwrap();
        assert_eq!(first.groups.len(), second.groups.len());
        assert_eq!(first.groups.len(), 1);
        assert_eq!(second.groups[0].files.len(), 2);
    }

    #[test]
    fn no_cache_does_not_write_a_cache_file() {
        let root = TempDir::new().unwrap();
        make_tree(&root);
        let cache_file = root.path().join("dd.cache");
        let params = Params {
            dir: Some(root.path().into()),
            no_cache: true,
            cache_file: Some(cache_file.clone()),
            ..Default::default()
        };

        run(&params).unwrap();
        assert!(!cache_file.exists());
    }
}
