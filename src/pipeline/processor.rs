use anyhow::Result;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use super::file::FileMeta;
use super::store::{Index, Store};
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

            let next_file = {
                let mut cfiles = self.files.lock().unwrap();
                cfiles.pop()
            };

            match next_file {
                None => continue,
                Some(file_res) => match FileMeta::new(file_res) {
                    Err(_) => continue,
                    Ok(fm) => {
                        // self.duplicates.add(Index::Partial(fm_arc.partial), fm_arc.clone());
                        let fm_arc = Arc::new(fm);
                        self.duplicates
                            .add(Index::Size(fm_arc.size), fm_arc.clone());
                    }
                },
            }
        }

        Ok(())
    }
}
