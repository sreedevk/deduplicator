use anyhow::Result;
use gxhash::gxhash128;
use memmap2::Mmap;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::SystemTime,
};

#[derive(Debug, Clone, PartialEq)]
pub enum FileState {
    Unprocessed,
    SwProcessed,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: Box<Path>,
    pub size: u64,
    pub modified: SystemTime,
    pub state: Arc<Mutex<FileState>>,
}

impl FileInfo {
    pub fn hash(&self, seed: i64) -> Result<u128> {
        let file = fs::File::open(&self.path)?;
        let mapper = unsafe { Mmap::map(&file)? };

        Ok(mapper
            .chunks(4096)
            .fold(0u128, |acc, chunk: &[u8]| acc ^ gxhash128(chunk, seed)))
    }

    pub fn initpage_hash(&self, seed: i64) -> Result<u128> {
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
            state: Arc::new(Mutex::new(FileState::Unprocessed)),
        })
    }

    pub fn sw_processed(&self) {
        let mut self_state = self.state.lock().unwrap();
        *self_state = FileState::SwProcessed;
    }

    pub fn is_sw_processed(&self) -> bool {
        let self_state = self.state.lock().unwrap();
        *self_state == FileState::SwProcessed
    }
}
