use anyhow::Result;
use memmap2::Mmap;
use std::fs;
use std::hash::Hasher;
use std::{fs::Metadata, path::PathBuf};

/// FileInfo is a struct that is used to represent a file on disk.
/// it contains the following information:
///     path - the absolute path to the file 
///     filemeta - file metadata of the file (std::fs::MetaData)
///     hash - Option of hash generated using the fxhash library
///     size - size of file obtained from the MetaData of the file
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub filemeta: Metadata,
    pub hash: Option<String>,
    pub size: u64,
}

impl FileInfo {
    /// returns a copy of self with the `hash` field replaced with Some(String) containing the hash
    /// of the contents of the file
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

    /// accepts the path of the file & returns a new FileInfo struct instance that contains
    /// metadata, size & path of the file.
    /// The hash is set to None at this stage as it will only be calculated when its absolutely
    /// required.
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
