use anyhow::Result;
use gxhash::GxHasher;
use memmap2::Mmap;
use std::fs;
use std::hash::Hasher;
use std::io::Read;
use std::{fs::Metadata, path::PathBuf};

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub filemeta: Metadata,
}

impl FileInfo {
    pub fn hash(&self) -> Result<String> {
        let file = fs::File::open(&self.path)?;
        let mapper = unsafe { Mmap::map(&file)? };
        let mut primhasher = GxHasher::default();

        mapper
            .chunks(1_000_000)
            .for_each(|chunk| primhasher.write(chunk));

        Ok(primhasher.finish().to_string())
    }

    pub fn initial_page_hash(&self) -> Result<String> {
        let file = fs::File::open(&self.path)?;
        let mapper = unsafe { Mmap::map(&file)? };
        let mut primhasher = GxHasher::default();
        primhasher.write(mapper.take(4096).into_inner());

        Ok(primhasher.finish().to_string())
    }

    pub fn new(path: PathBuf) -> Result<Self> {
        let filemeta = std::fs::metadata(&path)?;
        Ok(Self {
            path,
            filemeta: filemeta.clone(),
            size: filemeta.len(),
        })
    }
}
