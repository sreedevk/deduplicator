use super::{FileQueue, Message};
use anyhow::Result;
use std::fs;
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug)]
pub struct Scanner {
    files: FileQueue,
    proc_queue: FileQueue,
    msg_rx: Receiver<Message>,
}

impl Scanner {
    pub fn new(fq: FileQueue, rx: Receiver<Message>) -> Self {
        Self {
            files: fq,
            proc_queue: Arc::new(Mutex::new(vec![])),
            msg_rx: rx,
        }
    }

    pub fn index(&self) -> Result<()> {
        loop {
            match self.msg_rx.try_recv() {
                Ok(Message::AddScanDirectory(path)) => {
                    let mut mfq = self.proc_queue.lock().unwrap();
                    mfq.push(path);
                }
                Err(mpsc::TryRecvError::Empty) | Ok(Message::None) => {}
                Ok(Message::Exit) | Err(_) => break,
            }

            let npath = {
                match self.proc_queue.try_lock() {
                    Ok(mut q) => q.pop(),
                    Err(_) => None,
                }
            };

            match npath {
                None => continue,
                Some(path) => {
                    std::fs::read_dir(path.as_ref())?
                        .filter_map(Result::ok)
                        .for_each(|entry: fs::DirEntry| {
                            let mdata =
                                fs::metadata(entry.path()).expect("unable to read file metadata.");

                            let mpath = entry
                                .path()
                                .into_os_string()
                                .into_string()
                                .expect("invalid path conversion failed.")
                                .into_boxed_str();

                            match mdata.is_dir() {
                                true => {
                                    let mut pq = self
                                        .proc_queue
                                        .lock()
                                        .expect("proc queue lock acq failed.");
                                    pq.push(mpath);
                                }
                                false => {
                                    let mut fq = self
                                        .files
                                        .lock()
                                        .expect("file queue lock acq failed.");
                                    fq.push(mpath)
                                }
                            }
                        });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::fs::File;
    use std::io::Write;
    use std::sync::mpsc::channel;
    use tempfile::TempDir;
    use std::thread;

    #[test]
    fn scanner_scans_files() -> Result<()> {
        let files = 
            [("hello.txt", "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum"), 
            ("hello_dup.txt", "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum")];

        let file_queue: FileQueue = Arc::new(Mutex::new(vec![]));
        let (tx, rx) = channel::<Message>();
        let root = TempDir::new()?;

        for (filename, content) in files.into_iter() {
            let mut tf = File::create(root.path().join(filename))?;
            tf.write_all(content.as_bytes())?;
        }

        let rpath = root.path().to_str().unwrap().to_string().into_boxed_str();
        let fq_clone = file_queue.clone();
        let scanner_thread = thread::spawn(move || {
            let scanner = Scanner::new(fq_clone, rx);
            scanner.index().unwrap();
        });


        tx.send(Message::AddScanDirectory(rpath))?;
        tx.send(Message::Exit)?;
        scanner_thread.join().unwrap();

        let fq_len = {
            let v = file_queue.lock().unwrap();
            v.len()
        };

        assert_eq!(files.len(), fq_len);

        Ok(())
    }
}
