use anyhow::Result;
use dashmap::DashMap;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressFinish, ProgressStyle};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{borrow::Cow, time::Duration};

use crate::fileinfo::FileInfo;

#[derive(Debug, Clone)]
pub struct Processor {
    pub files: Vec<FileInfo>,
    pub hashwise_results: DashMap<String, Vec<FileInfo>>,
    pub sizewise_results: DashMap<u64, Vec<FileInfo>>,
    pub max_path_len: usize,
}

impl Processor {
    pub fn new(files: Vec<FileInfo>) -> Self {
        Self {
            files,
            hashwise_results: DashMap::new(),
            sizewise_results: DashMap::new(),
            max_path_len: 0,
        }
    }

    pub fn hashwise(&mut self) -> Result<()> {
        if self.sizewise_results.is_empty() {
            return Ok(());
        }

        let progress_style = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )?;
        let progress_bar = ProgressBar::new(self.files.len() as u64);
        progress_bar.set_style(progress_style);
        progress_bar.enable_steady_tick(Duration::from_millis(50));
        progress_bar.set_message("indexing file hashes");

        let filelist = self
            .sizewise_results
            .clone()
            .into_read_only()
            .values()
            .filter(|&subfiles| subfiles.len() > 1)
            .flatten()
            .cloned()
            .collect::<Vec<FileInfo>>();

        self.max_path_len = filelist
            .iter()
            .map(|x| x.path.clone().into_os_string().len())
            .max()
            .unwrap_or_default();

        filelist
            .into_par_iter()
            .progress_with(progress_bar)
            .with_finish(ProgressFinish::WithMessage(Cow::from(
                "indexed files hashes",
            )))
            .map(|file| file.hash())
            .filter_map(Result::ok)
            .for_each(move |file| {
                self.hashwise_results
                    .entry(file.hash.clone().unwrap_or_default())
                    .and_modify(|fileset| fileset.push(file.clone()))
                    .or_insert_with(|| vec![file]);
            });

        Ok(())
    }

    pub fn sizewise(&mut self) -> Result<()> {
        if self.files.is_empty() {
            return Ok(());
        }

        let progress_style = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )?;
        let progress_bar = ProgressBar::new(self.files.len() as u64);
        progress_bar.set_style(progress_style);
        progress_bar.enable_steady_tick(Duration::from_millis(50));
        progress_bar.set_message("indexing file sizes");

        self.files
            .clone()
            .into_par_iter()
            .progress_with(progress_bar)
            .with_finish(ProgressFinish::WithMessage(Cow::from(
                "indexed files sizes",
            )))
            .for_each(|file| {
                self.sizewise_results
                    .entry(file.size)
                    .and_modify(|fileset| fileset.push(file.clone()))
                    .or_insert_with(|| vec![file]);
            });

        Ok(())
    }
}
