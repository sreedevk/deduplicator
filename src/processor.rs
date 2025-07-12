use anyhow::Result;
use dashmap::DashMap;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, TryLockError, TryLockResult};

use crate::fileinfo::FileInfo;

pub struct Processor {}

impl Processor {
    pub fn hashwise(
        sw_store: Arc<DashMap<u64, Vec<FileInfo>>>,
        hw_store: Arc<DashMap<String, Vec<FileInfo>>>,
    ) -> Result<()> {
        let keys: Vec<u64> = sw_store.clone().iter().map(|i| *i.key()).collect();

        keys.into_iter().for_each(|key| {
            let group: Vec<FileInfo> = sw_store.get(&key).unwrap().to_vec();
            if group.len() > 1 {
                group.into_par_iter().for_each(|file| {
                    hw_store
                        .entry(file.hash.clone().unwrap_or_default())
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

    // TODO: reduce the amount of time files remain locked for
    pub fn sizewise(
        scanner_finished: Arc<AtomicBool>,
        store: Arc<DashMap<u64, Vec<FileInfo>>>,
        files: Arc<Mutex<Vec<FileInfo>>>,
        max_file_size: Arc<AtomicU64>,
    ) -> Result<()> {
        loop {
            match files.try_lock() {
                Ok(mut flist) => match flist.pop() {
                    Some(file) => {
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
                        true => break Ok(()),
                        false => continue,
                    },
                },
                TryLockResult::Err(TryLockError::WouldBlock) => continue,
                _ => break Ok(()),
            }
        }
    }
}
