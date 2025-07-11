use anyhow::Result;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::Arc;

use super::file::FileMeta;
use super::store::{Index, Store};
use super::{FileQueue, Message};

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
                Ok(Message::Exit) => break,
                Err(TryRecvError::Empty) => {},
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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::sync::mpsc;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn processor_groups_files_with_same_size() -> Result<()> {
        let file_queue = Arc::new(Mutex::new(vec![]));
        let (tx, rx) = mpsc::channel::<Message>();
        let root = TempDir::new()?;
        let store = Arc::new(Store::new());

        let fq_c = file_queue.clone();
        let store_c = store.clone();
        let proc_thread = thread::spawn(move || {
            let processor = Processor::new(fq_c, store_c, rx);
            processor.process().expect("processor failed");
        });

        let files = 
            [("hello.txt", "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum"), 
            ("hello_dup.txt", "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum")];

        for (filename, content) in files.into_iter() {
            let fpath = root.path().join(filename);
            let mut tf = File::create(fpath.clone())?;
            tf.write_all(content.as_bytes())?;

            let mut mfq = file_queue.lock().unwrap();
            mfq.push(fpath.into_os_string().into_string().unwrap().into_boxed_str());
        }

        for _ in 0..10 {
            if store.entries().is_empty() {
                thread::sleep(std::time::Duration::from_millis(100));
            } 
        }

        tx.send(Message::Exit).expect("unable to send msg to processor");
        proc_thread.join().expect("failed to join on thread!");

        assert!(store.entries().len() == 1);

        Ok(())
    }
}
