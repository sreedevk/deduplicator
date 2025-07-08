pub mod file;
mod flags;
mod processor;
mod scanner;
mod store;

use anyhow::Result;
use std::sync::mpsc;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Mutex;
use threadpool::ThreadPool;

use self::processor::Processor;
use self::scanner::Scanner;
use self::store::Store;

pub type FileQueue = Arc<Mutex<Vec<Box<str>>>>;

pub enum Message {
    AddScanDirectory(Box<str>),
    Exit,
    None,
}

pub struct Server {
    pub fq: FileQueue,
    pub dupstore: Arc<Store>,
    pub tpool: ThreadPool,
}

impl Server {
    pub fn new() -> Result<Self> {
        Ok(Self {
            fq: Arc::new(Mutex::new(vec![])),
            dupstore: Arc::new(Store::new()),
            tpool: ThreadPool::new(4),
        })
    }

    pub fn start(&self, rx: mpsc::Receiver<Message>) -> Result<()> {
        let processor_fq = self.fq.clone();
        let processor_store = self.dupstore.clone();
        let (processor_tx, processor_rx) = channel::<Message>();
        let (server_tx, server_rx) = channel::<Message>();

        self.tpool.execute(move || {
            Processor::new(processor_fq, processor_store, processor_rx)
                .process()
                .expect("processer execution interrupted.");
        });

        let scanner_fq = self.fq.clone();
        let (scanner_tx, scanner_rx) = channel::<Message>();
        self.tpool.execute(move || {
            Scanner::new(scanner_fq, scanner_rx)
                .index()
                .expect("scanner indexing interrupted.");
        });

        self.tpool.execute(move || loop {
            match rx.recv() {
                Ok(Message::AddScanDirectory(path)) => {
                    scanner_tx
                        .send(Message::AddScanDirectory(path))
                        .expect("scanner tx message passing failed.");
                }
                Ok(Message::None) => {}
                Ok(Message::Exit) | Err(_) => {
                    scanner_tx.send(Message::Exit).unwrap_or_default();
                    processor_tx.send(Message::Exit).unwrap_or_default();
                    break;
                }
            };
        });

        self.tpool.join();

        Ok(())
    }
}
