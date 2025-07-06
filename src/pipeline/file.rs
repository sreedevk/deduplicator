use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::sync::Arc;
use uuid::Uuid;

const PARTIAL_SIZE: u64 = 4096;

pub struct FileMeta {
    id: Uuid,
    path: Box<str>,
    size: u64,
    modtime: i64,
    partial: Arc<[u8]>,
    full_hash: Arc<[u8]>,
}

impl FileMeta {
    fn create_partial(path: &str) -> Result<Arc<[u8]>> {
        let mut partial_take = File::open(path)?.take(PARTIAL_SIZE);
        let mut partial_buffer = Vec::with_capacity(PARTIAL_SIZE as usize);
        partial_take.read_to_end(&mut partial_buffer)?;

        Ok(Arc::from(partial_buffer))
    }

    pub fn new(path: Box<str>) -> Result<Self> {
        let pstr = path.to_string();
        let filemeta = std::fs::metadata(&pstr)?;
        let partial = Self::create_partial(&pstr)?;

        Ok(Self {
            id: Uuid::new_v4(),
            size: filemeta.size(),
            modtime: filemeta.mtime(),
            full_hash: Arc::new([]),
            path,
            partial,
        })
    }
}
