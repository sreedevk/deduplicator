use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::sync::Arc;
use uuid::Uuid;

const PARTIAL_SIZE: u64 = 4096;

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct FileMeta {
    pub id: Uuid,
    pub path: Box<str>,
    pub size: u64,
    pub modtime: i64,
    pub partial: Arc<[u8]>,
    pub full_hash: Arc<[u8]>,
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
