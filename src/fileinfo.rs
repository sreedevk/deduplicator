use anyhow::Result;
use memmap2::Mmap;
use std::fs;
use std::hash::Hasher;
use std::{fs::Metadata, path::PathBuf};

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub filemeta: Metadata,
    pub hash: Option<String>,
    pub size: u64,
}

impl FileInfo {
    pub fn hash(&self) -> Result<Self> {
        let file = fs::File::open(self.path.clone())?;
        let mapper = unsafe { Mmap::map(&file)? };
        let mut primhasher = fxhash::FxHasher::default();

        mapper
            .chunks(1_000_000)
            .for_each(|chunk| primhasher.write(chunk));

        Ok(Self {
            hash: Some(primhasher.finish().to_string()),
            ..self.clone()
        })
    }

    pub fn new(path: PathBuf) -> Result<Self> {
        let filemeta = std::fs::metadata(path.clone())?;
        Ok(Self {
            path,
            filemeta: filemeta.clone(),
            hash: None,
            size: filemeta.len(),
        })
    }
}
