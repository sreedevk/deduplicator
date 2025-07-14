use anyhow::Result;
use gxhash::gxhash128;
use memmap2::Mmap;
use std::{
    fs,
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
    pub fn hash(&self, seed: i64) -> Result<u128> {
        let file = fs::File::open(&self.path)?;
        let mapper = unsafe { Mmap::map(&file)? };

        Ok(mapper
            .chunks(4096)
            .fold(0u128, |acc, chunk: &[u8]| acc ^ gxhash128(chunk, seed)))
    }

    pub fn initial_page_hash(&self, seed: i64) -> Result<u128> {
        let mut file = fs::File::open(&self.path)?;
        let mut buffer = [0; 4096];
        let bytes_read = file.read(&mut buffer)?;

        Ok(gxhash128(&buffer[..bytes_read], seed))
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
