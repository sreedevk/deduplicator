use anyhow::Result;
use dashmap::DashMap;
use indicatif::{
    MultiProgress, ParallelProgressIterator, ProgressBar, ProgressFinish, ProgressStyle,
};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, TryLockError, TryLockResult};
use std::time::Duration;

use crate::fileinfo::FileInfo;
use crate::params::Params;

pub struct Processor {}

impl Processor {
    pub fn hashwise(
        app_args: Arc<Params>,
        sw_store: Arc<DashMap<u64, Vec<FileInfo>>>,
        hw_store: Arc<DashMap<String, Vec<FileInfo>>>,
        progress_bar_box: Arc<MultiProgress>,
    ) -> Result<()> {
        let progress_bar = match app_args.progress {
            true => progress_bar_box.add(ProgressBar::new_spinner()),
            false => ProgressBar::hidden(),
        };

        let keys: Vec<u64> = sw_store.clone().iter().map(|i| *i.key()).collect();
        let progress_style = ProgressStyle::with_template("[{elapsed_precise}] {pos:>7} {msg}")?;
        progress_bar.set_style(progress_style);
        progress_bar.enable_steady_tick(Duration::from_millis(50));
        progress_bar.set_message("files grouped by hash.");

        keys.into_par_iter()
            .progress_with(progress_bar)
            .with_finish(ProgressFinish::WithMessage(Cow::from(
                "files grouped by hash.",
            )))
            .for_each(|key| {
                let group: Vec<FileInfo> = sw_store.get(&key).unwrap().to_vec();
                if group.len() > 1 {
                    group.into_par_iter().for_each(|file| {
                        let fhash = if app_args.strict {
                            file.hash().expect("hashing file failed.")
                        } else {
                            file.initial_page_hash().expect("hashing file failed.")
                        };

                        hw_store
                            .entry(fhash)
                            .and_modify(|fileset| fileset.push(file.clone()))
                            .or_insert_with(|| vec![file]);
                    });
                }
            });

        Ok(())
    }

    pub fn compare_and_update_max_path_len(current: Arc<AtomicU64>, next: u64) -> Result<()> {
        if current.load(Ordering::Relaxed) < next {
            current.store(next, Ordering::Release);
        }

        Ok(())
    }

    pub fn sizewise(
        app_args: Arc<Params>,
        scanner_finished: Arc<AtomicBool>,
        store: Arc<DashMap<u64, Vec<FileInfo>>>,
        files: Arc<Mutex<Vec<FileInfo>>>,
        max_file_size: Arc<AtomicU64>,
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
                    Self::compare_and_update_max_path_len(
                        max_file_size.clone(),
                        file.path.to_string_lossy().len() as u64,
                    )?;
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
    fn sizewise_sorting_normal() -> Result<()> {
        let root = TempDir::new()?;
        let files = [
            (root.path().join("fileone.bin"), generate_bytes(80)),
            (root.path().join("filetwo.bin"), generate_bytes(120)),
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
            Arc::new(AtomicU64::new(0)),
            Arc::new(MultiProgress::new()),
        )?;

        assert_eq!(dupstore.len(), 2);

        Ok(())
    }
}
