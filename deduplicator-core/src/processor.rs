use anyhow::Result;
use dashmap::DashMap;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

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

        let duplicates_table: DashMap<String, Vec<FileInfo>> = DashMap::new();
        self.files
            .clone()
            .into_par_iter()
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

        let duplicates_table: DashMap<u64, Vec<FileInfo>> = DashMap::new();
        self.files.clone().into_par_iter().for_each(|file| {
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
