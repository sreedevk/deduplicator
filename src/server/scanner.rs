use super::{FileQueue, Message};
use anyhow::Result;
use std::fs;
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::sync::Mutex;
use threadpool::ThreadPool;

pub struct Scanner {
    files: FileQueue,
    threadpool: Arc<ThreadPool>,
    proc_queue: FileQueue,
    msg_rx: Receiver<Message>,
}

impl Scanner {
    pub fn new(fq: FileQueue, rx: Receiver<Message>) -> Self {
        Self {
            files: fq,
            threadpool: Arc::new(ThreadPool::new(8)),
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
                let mut q = self.proc_queue.lock().unwrap();
                q.pop()
            };

            match npath {
                None => continue,
                Some(path) => {
                    let copy_of_queue = self.proc_queue.clone();
                    let copy_of_files = self.files.clone();

                    self.threadpool.execute(move || {
                        if let Ok((files, dirs)) = Self::scan(path) {
                            let mut q = copy_of_queue.lock().unwrap();
                            let mut f = copy_of_files.lock().unwrap();
                            q.extend(files);
                            f.extend(dirs);
                        }
                    });
                }
            }
        }

        Ok(())
    }

    fn scan(scan_path: Box<str>) -> Result<(Vec<Box<str>>, Vec<Box<str>>)> {
        let mut files = vec![];
        let mut dirs = vec![];

        fs::read_dir(scan_path.as_ref())?
            .filter_map(Result::ok)
            .for_each(|entry: fs::DirEntry| {
                let entrymeta: fs::Metadata = entry.metadata().unwrap();
                let path = entry
                    .path()
                    .into_os_string()
                    .into_string()
                    .unwrap()
                    .into_boxed_str();

                if entrymeta.is_file() {
                    files.push(path);
                } else if entrymeta.is_dir() {
                    dirs.push(path);
                }
            });

        Ok((files, dirs))
    }
}
