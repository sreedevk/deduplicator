use anyhow::Result;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use super::file::FileMeta;
use super::Store;
use super::{FileQueue, Message};

const BATCH_SIZE: usize = 10;

pub struct Processor {
    files: FileQueue,
    duplicates: Arc<Store>,
    msg_rx: Receiver<Message>,
}

impl Processor {
    pub fn new(files: FileQueue, duplicates: Arc<Store>, msg_rx: Receiver<Message>) -> Self {
        Self {
            files,
            duplicates,
            msg_rx,
        }
    }

    pub fn process(&self) -> Result<()> {
        loop {
            match self.msg_rx.try_recv() {
                Ok(Message::Exit) | Err(_) => break,
                _ => {}
            }
        }

        Ok(())
    }
}
