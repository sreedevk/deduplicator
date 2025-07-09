use super::{FileQueue, Message};
use anyhow::Result;
use std::fs;
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::sync::Mutex;
use vfs::FileSystem;

pub struct Scanner<T: FileSystem> {
    files: FileQueue,
    proc_queue: FileQueue,
    msg_rx: Receiver<Message>,
    root: Arc<T>,
}

impl<T: FileSystem> Scanner<T> {
    pub fn new(fq: FileQueue, rx: Receiver<Message>, fsys: Arc<T>) -> Self {
        Self {
            files: fq,
            proc_queue: Arc::new(Mutex::new(vec![])),
            msg_rx: rx,
            root: fsys,
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
                    self.root.read_dir(path.as_ref())?.for_each(|entry| {
                        let mdata = fs::metadata(&entry).expect("unable to read file metadata.");
                        let mpath = entry.into_boxed_str();
                        match mdata.is_dir() {
                            true => {
                                let mut pq =
                                    self.proc_queue.lock().expect("proc queue lock acq failed.");
                                pq.push(mpath);
                            }
                            false => {
                                let mut fq =
                                    self.files.lock().expect("file queue lock acq failed.");
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
    use std::sync::mpsc::channel;
    use std::thread;
    use vfs::MemoryFS;

    #[test]
    fn scanner_scans_files_on_vfs() -> Result<()> {
        let files = [
            ("hello.txt", "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum"), 

            ("hello_dup.txt", "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum")];

        let file_queue: FileQueue = Arc::new(Mutex::new(vec![]));
        let filesystem: Arc<MemoryFS> = Arc::new(MemoryFS::new());
        let fs_root = String::from("/root");
        let (tx, rx) = channel::<Message>();

        filesystem.create_dir(&fs_root)?;

        let fs_root_box = fs_root.into_boxed_str();
        for (filename, content) in files.into_iter() {
            filesystem
                .create_file(&format!("{}/{}", fs_root_box.as_ref(), filename))?
                .write_all(content.as_bytes())?;
        }

        let scanner = Scanner::new(file_queue.clone(), rx, filesystem);
        tx.send(Message::AddScanDirectory(fs_root_box))?;

        scanner.index()?;

        tx.send(Message::Exit)?;

        let fq_len = {
            let v = file_queue.lock().unwrap();
            v.len()
        };

        // assert_eq!(files.len(), fq_len);

        Ok(())
    }
}
