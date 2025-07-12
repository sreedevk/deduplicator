use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex};

use crate::processor::Processor;
use crate::scanner::Scanner;
use anyhow::Result;
use dashmap::DashMap;
use threadpool::ThreadPool;

use crate::fileinfo::FileInfo;
use crate::params::Params;

pub struct Server {
    filequeue: Arc<Mutex<Vec<FileInfo>>>,
    sw_duplicate_set: Arc<DashMap<u64, Vec<FileInfo>>>,
    pub hw_duplicate_set: Arc<DashMap<String, Vec<FileInfo>>>,
    threadpool: ThreadPool,
    app_args: Arc<Params>,
    pub max_file_path_len: Arc<AtomicU64>,
}

impl Server {
    pub fn new(opts: Params) -> Self {
        Self {
            filequeue: Arc::new(Mutex::new(Vec::new())),
            sw_duplicate_set: Arc::new(DashMap::new()),
            hw_duplicate_set: Arc::new(DashMap::new()),
            threadpool: ThreadPool::new(8),
            app_args: Arc::new(opts),
            max_file_path_len: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn start(&self) -> Result<()> {
        let app_args_clone = self.app_args.clone();
        let file_queue_clone_sc = self.filequeue.clone();
        let file_queue_clone_pr = self.filequeue.clone();
        let scanner_finished = Arc::new(AtomicBool::new(false));

        let sfin_sc_tr_cl = scanner_finished.clone();
        let sfin_pr_tr_cl = scanner_finished.clone();

        let store_dupl_sw_for_sw = self.sw_duplicate_set.clone();
        let store_dupl_sw_for_hw = self.sw_duplicate_set.clone();
        let store_dupl_hw = self.hw_duplicate_set.clone();
        let max_file_path_len_clone = self.max_file_path_len.clone();

        self.threadpool.execute(move || {
            Scanner::build(app_args_clone)
                .unwrap()
                .scan(file_queue_clone_sc)
                .unwrap();

            sfin_sc_tr_cl.store(true, std::sync::atomic::Ordering::Relaxed);
        });

        self.threadpool.execute(move || {
            Processor::sizewise(
                sfin_pr_tr_cl,
                store_dupl_sw_for_sw,
                file_queue_clone_pr,
                max_file_path_len_clone,
            )
            .unwrap();

            Processor::hashwise(store_dupl_sw_for_hw, store_dupl_hw).unwrap();
        });

        self.threadpool.join();

        Ok(())
    }
}
