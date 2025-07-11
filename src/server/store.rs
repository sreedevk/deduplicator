use super::file::FileMeta;
use dashmap::DashMap;
use std::sync::Arc;

#[allow(unused)]
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Index {
    Size(u64),
    Partial(Arc<[u8]>),
    Full(Box<str>),
}

#[derive(Debug)]
pub struct Store {
    internal: Arc<DashMap<Index, Vec<Arc<FileMeta>>>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            internal: Arc::new(DashMap::new()),
        }
    }

    pub fn entries(&self) -> Arc<DashMap<Index, Vec<Arc<FileMeta>>>> {
        self.internal.clone()
    }

    pub fn add(&self, index: Index, file: Arc<FileMeta>) {
        self.internal
            .entry(index)
            .and_modify(|fg| fg.push(file.clone()))
            .or_insert(vec![file]);
    }
}
