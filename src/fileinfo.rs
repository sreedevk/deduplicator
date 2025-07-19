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
        if self.size == 0 {
            return Ok(0u128);
        };

        let file = fs::File::open(&self.path)?;
        let mapper = unsafe { Mmap::map(&file)? };
        let content_hash = mapper
            .chunks(4096)
            .fold(0u128, |acc, chunk: &[u8]| acc ^ gxhash128(chunk, seed));

        // NOTE: avoids collision bw an empty file & a file full of null bytes.
        Ok(content_hash ^ gxhash128(&self.size.to_ne_bytes(), seed))
    }

    pub fn initpages_hash(&self, seed: i64) -> Result<u128> {
        let mut file = fs::File::open(&self.path)?;
        let mut buffer = [0; 16384];
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

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;
    use anyhow::Result;

    fn generate_null_bytes(size: usize) -> Vec<u8> {
        (0..size).map(|_| 0).collect::<Vec<u8>>()
    }

    #[test]
    fn hash_differentiates_between_a_file_of_null_bytes_vs_an_empty_file() -> Result<()> {
        let root = TempDir::new()?;
        let empty_file_name = root.path().join("empty_file.bin");

        File::create_new(&empty_file_name)?;

        let file_with_null_bytes_name = root.path().join("file_with_null_bytes.bin");
        let mut file_with_null_bytes = File::create_new(&file_with_null_bytes_name)?;

        file_with_null_bytes.write_all(&generate_null_bytes(1000 * 4096))?;

        let empty_file_info = FileInfo::new(empty_file_name)?;
        let file_with_empty_bytes_info = FileInfo::new(file_with_null_bytes_name)?;

        let seed: i64 = 246910456374;

        assert_ne!(empty_file_info.hash(seed)?, file_with_empty_bytes_info.hash(seed)?);

        Ok(())
    }
}
