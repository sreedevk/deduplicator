use anyhow::Result;
use dashmap::DashMap;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::iter::IntoParallelRefMutIterator;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, TryLockError, TryLockResult};
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;

use crate::fileinfo::FileInfo;
use crate::params::Params;

pub struct Processor {}

impl Processor {
    pub fn hashwise(
        app_args: Arc<Params>,
        sw_store: Arc<DashMap<u64, Vec<FileInfo>>>,
        hw_store: Arc<DashMap<u128, Vec<FileInfo>>>,
        progress_bar_box: Arc<MultiProgress>,
        max_file_size: Arc<AtomicU64>,
        seed: i64,
        sw_sorting_finished: Arc<AtomicBool>,
    ) -> Result<()> {
        let progress_bar = match app_args.progress {
            true => progress_bar_box.add(ProgressBar::new_spinner()),
            false => ProgressBar::hidden(),
        };

        let progress_style = ProgressStyle::with_template("[{elapsed_precise}] {pos:>7} {msg}")?;
        progress_bar.set_style(progress_style);
        progress_bar.enable_steady_tick(Duration::from_millis(50));
        progress_bar.set_message("files grouped by hash.");

        loop {
            let keys: Vec<u64> = sw_store
                .clone()
                .iter()
                .filter(|i| !i.value().iter().all(|x| x.is_sw_processed()))
                .filter(|i| i.value().len() > 1)
                .map(|i| *i.key())
                .collect();

            if keys.is_empty() {
                match sw_sorting_finished.load(std::sync::atomic::Ordering::Relaxed) {
                    true => {
                        progress_bar.finish_with_message("files grouped by hash.");
                        break Ok(());
                    }
                    false => continue,
                }
            } else {
                keys.into_par_iter().for_each(|key| {
                    let mut group: Vec<FileInfo> = sw_store.get(&key).unwrap().to_vec();
                    if group.len() > 1 {
                        group.par_iter_mut().for_each(|file| {
                            progress_bar.inc(1);
                            file.sw_processed();

                            let fhash = match app_args.strict {
                                true => file.hash(seed).expect("hashing file failed."),
                                false => file.initpages_hash(seed).expect("hashing file failed."),
                            };

                            Self::compare_and_update_max_path_len(
                                max_file_size.clone(),
                                file.path.to_string_lossy().graphemes(true).count() as u64,
                            );

                            hw_store
                                .entry(fhash)
                                .and_modify(|fileset| fileset.push(file.clone()))
                                .or_insert_with(|| vec![file.clone()]);
                        });
                    };
                });
            }
        }
    }

    pub fn compare_and_update_max_path_len(current: Arc<AtomicU64>, next: u64) {
        if current.load(Ordering::Relaxed) < next {
            current.store(next, Ordering::Release);
        }
    }

    pub fn sizewise(
        app_args: Arc<Params>,
        scanner_finished: Arc<AtomicBool>,
        store: Arc<DashMap<u64, Vec<FileInfo>>>,
        files: Arc<Mutex<Vec<FileInfo>>>,
        progress_bar_box: Arc<MultiProgress>,
    ) -> Result<()> {
        let progress_bar = match app_args.progress {
            true => progress_bar_box.add(ProgressBar::new_spinner()),
            false => ProgressBar::hidden(),
        };

        let progress_style = ProgressStyle::with_template("[{elapsed_precise}] {pos:>7} {msg}")?;
        progress_bar.set_style(progress_style);
        progress_bar.enable_steady_tick(Duration::from_millis(50));
        progress_bar.set_message("files grouped by size");

        loop {
            let fileopt: Option<FileInfo> = {
                match files.try_lock() {
                    Ok(mut flist) => flist.pop(),
                    TryLockResult::Err(TryLockError::WouldBlock) => None,
                    _ => None,
                }
            };

            match fileopt {
                Some(file) => {
                    progress_bar.inc(1);
                    store
                        .entry(file.size)
                        .and_modify(|fileset| fileset.push(file.clone()))
                        .or_insert_with(|| vec![file]);
                    continue;
                }
                None => match scanner_finished.load(std::sync::atomic::Ordering::Relaxed) {
                    true => {
                        progress_bar.finish_with_message("files grouped by size");
                        break Ok(());
                    }
                    false => continue,
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use dashmap::DashMap;
    use indicatif::MultiProgress;
    use rand::Rng;
    use std::fs::File;
    use std::io::Write;
    use std::sync::atomic::{AtomicBool, AtomicU64};
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    use crate::{fileinfo::FileInfo, params::Params};

    use super::Processor;

    fn generate_bytes(size: usize) -> Vec<u8> {
        let mut rng = rand::rng();
        (0..size).map(|_| rng.random::<u8>()).collect::<Vec<u8>>()
    }

    #[test]
    fn hashwise_sorting_two_files_with_identical_init_pages_only_strict_mode() -> Result<()> {
        let root = TempDir::new()?;
        let content = generate_bytes(16384);

        let mut content_x = content.clone();
        let mut content_y = content.clone();

        content_x.extend(generate_bytes(1720320));
        content_y.extend(generate_bytes(1720320));

        let files = [
            (root.path().join("fileone.bin"), content_x),
            (root.path().join("filetwo.bin"), content_y),
        ];

        for (fpath, content) in files.iter() {
            let mut f = File::create_new(fpath)?;
            f.write_all(content)?;
        }

        let dupstore = Arc::new(DashMap::new());
        let file_queue = Arc::new(Mutex::new(
            files
                .iter()
                .map(|f| FileInfo::new(f.0.clone()).unwrap())
                .collect::<Vec<FileInfo>>(),
        ));

        let hw_dupstore = Arc::new(DashMap::new());
        Processor::sizewise(
            Arc::new(Params::default()),
            Arc::new(AtomicBool::new(true)),
            dupstore.clone(),
            file_queue,
            Arc::new(MultiProgress::new()),
        )?;

        let args = Params {
            strict: true,
            ..Default::default()
        };

        Processor::hashwise(
            Arc::new(args),
            dupstore.clone(),
            hw_dupstore.clone(),
            Arc::new(MultiProgress::new()),
            Arc::new(AtomicU64::new(32)),
            300,
            Arc::new(AtomicBool::new(true)),
        )?;

        assert_eq!(hw_dupstore.len(), 2);

        Ok(())
    }

    #[test]
    fn hashwise_sorting_two_files_with_identical_init_pages_only_fast_mode() -> Result<()> {
        let root = TempDir::new()?;
        let content = generate_bytes(16384);

        let mut content_x = content.clone();
        let mut content_y = content.clone();

        content_x.extend(generate_bytes(1720320));
        content_y.extend(generate_bytes(1720320));

        let files = [
            (root.path().join("fileone.bin"), content_x),
            (root.path().join("filetwo.bin"), content_y),
        ];

        for (fpath, content) in files.iter() {
            let mut f = File::create_new(fpath)?;
            f.write_all(content)?;
        }

        let dupstore = Arc::new(DashMap::new());
        let file_queue = Arc::new(Mutex::new(
            files
                .iter()
                .map(|f| FileInfo::new(f.0.clone()).unwrap())
                .collect::<Vec<FileInfo>>(),
        ));

        let hw_dupstore = Arc::new(DashMap::new());
        Processor::sizewise(
            Arc::new(Params::default()),
            Arc::new(AtomicBool::new(true)),
            dupstore.clone(),
            file_queue,
            Arc::new(MultiProgress::new()),
        )?;

        Processor::hashwise(
            Arc::new(Params::default()),
            dupstore.clone(),
            hw_dupstore.clone(),
            Arc::new(MultiProgress::new()),
            Arc::new(AtomicU64::new(32)),
            300,
            Arc::new(AtomicBool::new(true)),
        )?;

        assert_eq!(hw_dupstore.len(), 1);

        Ok(())
    }

    #[test]
    fn hashwise_sorting_two_files_with_identical_data() -> Result<()> {
        let root = TempDir::new()?;
        let content = generate_bytes(282624);
        let files = [
            (root.path().join("fileone.bin"), content.clone()),
            (root.path().join("filetwo.bin"), content.clone()),
        ];

        for (fpath, content) in files.iter() {
            let mut f = File::create_new(fpath)?;
            f.write_all(content)?;
        }

        let dupstore = Arc::new(DashMap::new());
        let file_queue = Arc::new(Mutex::new(
            files
                .iter()
                .map(|f| FileInfo::new(f.0.clone()).unwrap())
                .collect::<Vec<FileInfo>>(),
        ));

        let hw_dupstore = Arc::new(DashMap::new());
        Processor::sizewise(
            Arc::new(Params::default()),
            Arc::new(AtomicBool::new(true)),
            dupstore.clone(),
            file_queue,
            Arc::new(MultiProgress::new()),
        )?;

        Processor::hashwise(
            Arc::new(Params::default()),
            dupstore.clone(),
            hw_dupstore.clone(),
            Arc::new(MultiProgress::new()),
            Arc::new(AtomicU64::new(32)),
            300,
            Arc::new(AtomicBool::new(true)),
        )?;

        assert_eq!(hw_dupstore.len(), 1);

        Ok(())
    }

    #[test]
    fn sizewise_sorting_two_files_of_different_sizes() -> Result<()> {
        let root = TempDir::new()?;
        let files = [
            (root.path().join("fileone.bin"), generate_bytes(282624)),
            (root.path().join("filetwo.bin"), generate_bytes(1720320)),
        ];

        for (fpath, content) in files.iter() {
            let mut f = File::create_new(fpath)?;
            f.write_all(content)?;
        }

        let file_queue = Arc::new(Mutex::new(
            files
                .iter()
                .map(|f| FileInfo::new(f.0.clone()).unwrap())
                .collect::<Vec<FileInfo>>(),
        ));

        let dupstore = Arc::new(DashMap::new());

        Processor::sizewise(
            Arc::new(Params::default()),
            Arc::new(AtomicBool::new(true)),
            dupstore.clone(),
            file_queue,
            Arc::new(MultiProgress::new()),
        )?;

        assert_eq!(dupstore.len(), 2);

        Ok(())
    }

    #[test]
    fn sizewise_sorting_two_files_of_same_size() -> Result<()> {
        let root = TempDir::new()?;
        let files = [
            (root.path().join("fileone.bin"), generate_bytes(282624)),
            (root.path().join("filetwo.bin"), generate_bytes(282624)),
        ];

        for (fpath, content) in files.iter() {
            let mut f = File::create_new(fpath)?;
            f.write_all(content)?;
        }

        let file_queue = Arc::new(Mutex::new(
            files
                .iter()
                .map(|f| FileInfo::new(f.0.clone()).unwrap())
                .collect::<Vec<FileInfo>>(),
        ));

        let dupstore = Arc::new(DashMap::new());

        Processor::sizewise(
            Arc::new(Params::default()),
            Arc::new(AtomicBool::new(true)),
            dupstore.clone(),
            file_queue,
            Arc::new(MultiProgress::new()),
        )?;

        assert_eq!(dupstore.len(), 1);

        Ok(())
    }
}
