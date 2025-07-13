use anyhow::Result;
use gxhash::GxHasher;
use memmap2::Mmap;
use std::{
    fs,
    hash::Hasher,
    io::Read,
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: Box<Path>,
    pub size: u64,
    pub modified: SystemTime,
}

impl FileInfo {
    pub fn hash(&self) -> Result<u128> {
        let file = fs::File::open(&self.path)?;
        let mapper = unsafe { Mmap::map(&file)? };
        let mut primhasher = GxHasher::default();

        mapper
            .chunks(1_000_000)
            .for_each(|chunk| primhasher.write(chunk));

        Ok(primhasher.finish_u128())
    }

    pub fn initial_page_hash(&self) -> Result<u128> {
        let file = fs::File::open(&self.path)?;
        let mapper = unsafe { Mmap::map(&file)? };
        let mut primhasher = GxHasher::default();
        primhasher.write(mapper.take(4096).into_inner());

        Ok(primhasher.finish_u128())
    }

    pub fn new(path: PathBuf) -> Result<Self> {
        let filemeta = std::fs::metadata(&path)?;
        Ok(Self {
            path: path.into_boxed_path(),
            size: filemeta.len(),
            modified: filemeta.modified()?,
        })
    }
}
