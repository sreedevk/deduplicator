use super::file::FileMeta;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Index {
    Size(Box<str>),
    Partial(Box<str>),
    Full(Box<str>),
}

pub struct Store {
    internal: Arc<Mutex<HashMap<Index, Vec<Arc<FileMeta>>>>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            internal: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add(&self, index: Index, file: Arc<FileMeta>) -> Result<()> {
        let mut imut = self.internal.lock().unwrap();
        imut.entry(index)
            .and_modify(|fg| fg.push(file.clone()))
            .or_insert(vec![file]);

        Ok(())
    }
}
