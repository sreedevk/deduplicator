use anyhow::Result;
use dashmap::DashMap;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle, ProgressFinish};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{time::Duration, borrow::Cow};

use crate::fileinfo::FileInfo;

#[derive(Debug, Clone)]
pub enum State {
    Initial,
    SizeWise,
    HashWise,
}

#[derive(Debug, Clone)]
pub struct Processor {
    pub files: Vec<FileInfo>,
    pub state: State,
}

impl Processor {
    pub fn new(files: Vec<FileInfo>) -> Self {
        Self {
            files,
            state: State::Initial,
        }
    }

    pub fn hashwise(&self) -> Result<Self> {
        if self.files.is_empty() {
            return Ok(self.clone());
        }

        let progress_style = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")?;
        let progress_bar = ProgressBar::new(self.files.len() as u64);
        progress_bar.set_style(progress_style);
        progress_bar.enable_steady_tick(Duration::from_millis(50));
        progress_bar.set_message("indexing file hashes");

        let duplicates_table: DashMap<String, Vec<FileInfo>> = DashMap::new();
        self.files
            .clone()
            .into_par_iter()
            .progress_with(progress_bar)
            .with_finish(ProgressFinish::WithMessage(Cow::from("indexed files hashes")))
            .map(|file| file.hash())
            .filter_map(Result::ok)
            .for_each(|file| {
                duplicates_table
                    .entry(file.hash.clone().unwrap_or_default())
                    .and_modify(|fileset| fileset.push(file.clone()))
                    .or_insert_with(|| vec![file]);
            });

        let files = duplicates_table
            .into_read_only()
            .values()
            .cloned()
            .filter(|subfiles| subfiles.len() > 1)
            .flatten()
            .collect::<Vec<FileInfo>>();

        Ok(Self {
            files,
            state: State::HashWise,
        })
    }

    pub fn sizewise(&self) -> Result<Self> {
        if self.files.is_empty() {
            return Ok(self.clone());
        }

        let progress_style = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")?;
        let progress_bar = ProgressBar::new(self.files.len() as u64);
        progress_bar.set_style(progress_style);
        progress_bar.enable_steady_tick(Duration::from_millis(50));
        progress_bar.set_message("indexing file sizes");

        let duplicates_table: DashMap<u64, Vec<FileInfo>> = DashMap::new();
        self.files
            .clone()
            .into_par_iter()
            .progress_with(progress_bar)
            .with_finish(ProgressFinish::WithMessage(Cow::from("indexed files sizes")))
            .for_each(|file| {
                duplicates_table
                    .entry(file.size)
                    .and_modify(|fileset| fileset.push(file.clone()))
                    .or_insert_with(|| vec![file]);
            });

        let files = duplicates_table
            .into_read_only()
            .values()
            .cloned()
            .filter(|subfiles| subfiles.len() > 1)
            .flatten()
            .collect::<Vec<FileInfo>>();

        Ok(Self {
            files,
            state: State::SizeWise,
        })
    }
}
