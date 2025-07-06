use anyhow::{anyhow, Result};
use std::fs;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
use std::sync::Arc;
use std::sync::Mutex;
use threadpool::ThreadPool;

pub struct Scanner {
    files: Arc<Mutex<Vec<Box<str>>>>,
    threadpool: Arc<ThreadPool>,
    proc_count: Arc<AtomicU32>,
    proc_queue: Arc<Mutex<Vec<Box<str>>>>,
}

impl Scanner {
    pub fn new(path: Box<str>) -> Result<Self> {
        Ok(Self {
            files: Arc::new(Mutex::new(vec![])),
            threadpool: Arc::new(ThreadPool::new(8)),
            proc_count: Arc::new(AtomicU32::new(1)),
            proc_queue: Arc::new(Mutex::new(vec![path])),
        })
    }

    pub fn index(&self) -> Result<()> {
        loop {
            let pc = self.proc_count.load(Relaxed);
            match pc {
                0 => break,
                _ => {
                    let npath = {
                        let mut q = self.proc_queue.lock().unwrap();
                        q.pop()
                    };

                    match npath {
                        None => continue,
                        Some(path) => {
                            self.proc_count.fetch_add(1, Relaxed);
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

                // TODO: Support Symlinks
                if entrymeta.is_file() {
                    files.push(path);
                } else if entrymeta.is_dir() {
                    dirs.push(path);
                }
            });

        Ok((files, dirs))
    }
}
