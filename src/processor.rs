use std::collections::HashMap;

use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::cache::{mtime_nanos, Cache};
use crate::fileinfo::FileInfo;
use crate::pipeline::{spinner, DuplicateGroup};

pub fn group_by_size(files: Vec<FileInfo>, progress: bool) -> Vec<Vec<FileInfo>> {
    let bar = spinner(progress, "files grouped by size");

    let mut buckets: HashMap<u64, Vec<FileInfo>> = HashMap::new();
    for file in files {
        bar.inc(1);
        buckets.entry(file.size).or_default().push(file);
    }

    bar.finish_with_message("files grouped by size");
    buckets.into_values().collect()
}

pub fn hash_candidates(
    candidates: Vec<FileInfo>,
    strict: bool,
    seed: i64,
    progress: bool,
    cache: &Cache,
) -> Vec<(u128, FileInfo)> {
    let bar = spinner(progress, "files grouped by hash");

    let hashed: Vec<(u128, FileInfo)> = candidates
        .into_par_iter()
        .map(|file| {
            bar.inc(1);
            let mtime = mtime_nanos(file.modified);
            let hash = match cache.lookup(&file.path, file.size, mtime, strict) {
                Some(cached) => cached,
                None => match strict {
                    true => file.hash(seed).expect("hashing file failed."),
                    false => file.initpages_hash(seed).expect("hashing file failed."),
                },
            };
            (hash, file)
        })
        .collect();

    bar.finish_with_message("files grouped by hash.");
    hashed
}

pub fn group_hashed(hashed: Vec<(u128, FileInfo)>) -> Vec<DuplicateGroup> {
    let mut buckets: HashMap<u128, Vec<FileInfo>> = HashMap::new();
    for (hash, file) in hashed {
        buckets.entry(hash).or_default().push(file);
    }

    buckets
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(hash, files)| DuplicateGroup { hash, files })
        .collect()
}

#[cfg(test)]
mod staged_tests {
    use anyhow::Result;
    use rand::Rng;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    use crate::fileinfo::FileInfo;

    fn generate_bytes(size: usize) -> Vec<u8> {
        let mut rng = rand::rng();
        (0..size).map(|_| rng.random::<u8>()).collect::<Vec<u8>>()
    }

    fn write_files(root: &TempDir, specs: Vec<(&str, Vec<u8>)>) -> Result<Vec<FileInfo>> {
        specs
            .into_iter()
            .map(|(name, content)| {
                let path = root.path().join(name);
                let mut file = File::create_new(&path)?;
                file.write_all(&content)?;
                FileInfo::new(path)
            })
            .collect()
    }

    #[test]
    fn group_by_size_separates_files_of_different_sizes() -> Result<()> {
        let root = TempDir::new()?;
        let files = write_files(
            &root,
            vec![
                ("fileone.bin", generate_bytes(282624)),
                ("filetwo.bin", generate_bytes(1720320)),
            ],
        )?;

        let groups = super::group_by_size(files, false);
        assert_eq!(groups.len(), 2);
        Ok(())
    }

    #[test]
    fn group_by_size_buckets_same_size_files_together() -> Result<()> {
        let root = TempDir::new()?;
        let files = write_files(
            &root,
            vec![
                ("fileone.bin", generate_bytes(282624)),
                ("filetwo.bin", generate_bytes(282624)),
            ],
        )?;

        let groups = super::group_by_size(files, false);
        assert_eq!(groups.len(), 1);
        Ok(())
    }

    #[test]
    fn group_by_hash_fast_mode_matches_identical_init_pages() -> Result<()> {
        let root = TempDir::new()?;
        let shared = generate_bytes(16384);

        let mut content_x = shared.clone();
        let mut content_y = shared.clone();
        content_x.extend(generate_bytes(1720320));
        content_y.extend(generate_bytes(1720320));

        let files = write_files(
            &root,
            vec![("fileone.bin", content_x), ("filetwo.bin", content_y)],
        )?;

        let groups = super::group_hashed(super::hash_candidates(files, false, 300, false, &crate::cache::Cache::disabled()));
        assert_eq!(groups.len(), 1);
        Ok(())
    }

    #[test]
    fn group_by_hash_strict_mode_rejects_different_tails() -> Result<()> {
        let root = TempDir::new()?;
        let shared = generate_bytes(16384);

        let mut content_x = shared.clone();
        let mut content_y = shared.clone();
        content_x.extend(generate_bytes(1720320));
        content_y.extend(generate_bytes(1720320));

        let files = write_files(
            &root,
            vec![("fileone.bin", content_x), ("filetwo.bin", content_y)],
        )?;

        let groups = super::group_hashed(super::hash_candidates(files, true, 300, false, &crate::cache::Cache::disabled()));
        assert_eq!(groups.len(), 0);
        Ok(())
    }

    #[test]
    fn group_by_hash_matches_identical_files() -> Result<()> {
        let root = TempDir::new()?;
        let content = generate_bytes(282624);

        let files = write_files(
            &root,
            vec![
                ("fileone.bin", content.clone()),
                ("filetwo.bin", content.clone()),
            ],
        )?;

        let groups = super::group_hashed(super::hash_candidates(files, false, 300, false, &crate::cache::Cache::disabled()));
        assert_eq!(groups.len(), 1);
        Ok(())
    }

    #[test]
    fn hash_candidates_uses_cached_hash_when_valid() {
        let root = TempDir::new().unwrap();
        let path = root.path().join("f.bin");
        let mut f = File::create_new(&path).unwrap();
        f.write_all(b"real content for cache hit test").unwrap();
        let info = FileInfo::new(path.clone()).unwrap();
        let mtime = crate::cache::mtime_nanos(info.modified);

        let mut cache = crate::cache::Cache::disabled();
        cache.record(&path, info.size, mtime, false, 0xDEAD_BEEF, 0);

        let hashed = super::hash_candidates(vec![info], false, 300, false, &cache);
        assert_eq!(hashed[0].0, 0xDEAD_BEEF);
    }

    #[test]
    fn hash_candidates_recomputes_when_mtime_differs() {
        let root = TempDir::new().unwrap();
        let path = root.path().join("f.bin");
        let mut f = File::create_new(&path).unwrap();
        f.write_all(b"real content for cache miss test").unwrap();
        let info = FileInfo::new(path.clone()).unwrap();
        let mtime = crate::cache::mtime_nanos(info.modified);
        let real = info.initpages_hash(300).unwrap();

        let mut cache = crate::cache::Cache::disabled();
        cache.record(&path, info.size, mtime + 1, false, 0xDEAD_BEEF, 0);

        let hashed = super::hash_candidates(vec![info], false, 300, false, &cache);
        assert_eq!(hashed[0].0, real);
        assert_ne!(hashed[0].0, 0xDEAD_BEEF);
    }
}
