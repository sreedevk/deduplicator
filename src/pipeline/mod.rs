mod file;
mod scanner;
use std::sync::Arc;
use std::sync::Mutex;

use self::file::FileMeta;

const CONCURRENCY: usize = 4;

pub struct Server { }

impl Server {
    // pub fn new() -> Result<Self> {
    //     Ok(Self {
    //         index_queue: Arc::new(Mutex::new(vec![])),
    //         index_tpool: ThreadPool::new(CONCURRENCY),
    //         process_queue: Arc::new(Mutex::new(vec![])),
    //     })
    // }

    // fn index(&self) -> Result<()> {
    //     let mut iq = self.index_queue.lock().unwrap();
    //     let mut pq = self.process_queue.lock().unwrap();
    //
    //     match iq.pop() {
    //         None => Ok(()),
    //         Some(QueueElem::File(path)) => {
    //             pq.push(FileMeta::new(path)?);
    //
    //             Ok(())
    //         }
    //         Some(QueueElem::Directory(path)) => {
    //             self.index_tpool.execute(move || {
    //                 Scanner::new(&path);
    //             });
    //
    //             Ok(())
    //         }
    //     }
    // }
}
