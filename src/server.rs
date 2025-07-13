use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex};

use crate::processor::Processor;
use crate::scanner::Scanner;
use anyhow::Result;
use dashmap::DashMap;
use indicatif::{MultiProgress, ProgressDrawTarget};
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
            threadpool: ThreadPool::new(4),
            app_args: Arc::new(opts),
            max_file_path_len: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn start(&self) -> Result<()> {
        let progbarbox = Arc::new(MultiProgress::new());

        if !self.app_args.progress {
            progbarbox.set_draw_target(ProgressDrawTarget::hidden());
        }

        let app_args_clone_for_sc = self.app_args.clone();
        let app_args_clone_for_pr = self.app_args.clone();
        let file_queue_clone_sc = self.filequeue.clone();
        let file_queue_clone_pr = self.filequeue.clone();
        let scanner_finished = Arc::new(AtomicBool::new(false));

        let sfin_sc_tr_cl = scanner_finished.clone();
        let sfin_pr_tr_cl = scanner_finished.clone();

        let store_dupl_sw_for_sw = self.sw_duplicate_set.clone();
        let store_dupl_sw_for_hw = self.sw_duplicate_set.clone();
        let store_dupl_hw = self.hw_duplicate_set.clone();
        let max_file_path_len_clone = self.max_file_path_len.clone();

        let progbarbox_sc_clone = progbarbox.clone();

        self.threadpool.execute(move || {
            Scanner::new(app_args_clone_for_sc)
                .unwrap()
                .scan(file_queue_clone_sc, progbarbox_sc_clone)
                .unwrap();

            sfin_sc_tr_cl.store(true, std::sync::atomic::Ordering::Relaxed);
        });

        let progbarbox_pr_clone = progbarbox.clone();

        self.threadpool.execute(move || {
            Processor::sizewise(
                app_args_clone_for_pr.clone(),
                sfin_pr_tr_cl,
                store_dupl_sw_for_sw,
                file_queue_clone_pr,
                max_file_path_len_clone,
                progbarbox_pr_clone.clone(),
            )
            .unwrap();

            Processor::hashwise(
                app_args_clone_for_pr,
                store_dupl_sw_for_hw,
                store_dupl_hw,
                progbarbox_pr_clone,
            )
            .unwrap();
        });

        progbarbox.clear()?;

        self.threadpool.join();

        Ok(())
    }
}
