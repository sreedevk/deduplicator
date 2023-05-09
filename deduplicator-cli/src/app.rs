use crate::params::Params;
use anyhow::Result;
use deduplicator_core::processor::Processor;
use deduplicator_core::{fileinfo::FileInfo, scanner::Scanner};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct App;

impl App {
    pub fn init(app_args: &Params) -> Result<()> {
        let (scan_tx, scan_rx) = mpsc::channel::<usize>();
        let app_args_cloned: Params = app_args.clone().to_owned();
        let scanner_t = thread::spawn(move || -> Result<Vec<FileInfo>> {
            let scan_result = Self::build_scanner(&app_args_cloned)?.scan(Some(scan_tx.clone()));
            scan_tx.send(0)?;

            scan_result
        });

        let scanner_progress_t = thread::spawn(move || -> Result<()> {
            let progress = Self::create_progress_bar()?;
            loop {
                dbg!("testing");
                match scan_rx.recv() {
                    Ok(code) => match code {
                        0 => {
                            progress.finish();
                            break;
                        }
                        _ => {
                            progress.inc(1u64);
                        }
                    },
                    Err(_) => {
                        progress.finish();
                        break;
                    }
                }
            }

            Ok(())
        });

        let scan_results  = scanner_t.join().unwrap()?;
        let _scan_prog_tr = scanner_progress_t.join().unwrap()?;

        let processor_t = thread::spawn(move || -> Result<Vec<FileInfo>> {
            let processor = Processor::new(scan_results);
            Ok(processor.sizewise()?.hashwise()?.files)
        });

        let duplicates = processor_t.join().unwrap()?;

        dbg!(duplicates);

        Ok(())
    }

    pub fn create_progress_bar() -> Result<ProgressBar> {
        let progress = ProgressBar::new_spinner();
        let progress_style =
            ProgressStyle::with_template("{spinner:.green} [mapping paths] {pos} paths")?;
        progress.set_style(progress_style);
        progress.enable_steady_tick(Duration::from_millis(50));

        Ok(progress)
    }

    pub fn build_scanner(app_args: &Params) -> Result<Scanner> {
        let scan_directory = app_args.get_directory()?;
        Ok(Scanner::new())
            .map(|scanner| scanner.directory(scan_directory))
            .map(|scanner| match app_args.get_min_size() {
                Some(min_size) => scanner.min_size(min_size),
                None => scanner,
            })
            .map(|scanner| match app_args.min_depth {
                Some(min_depth) => scanner.min_depth(min_depth),
                None => scanner,
            })
            .map(|scanner| match app_args.max_depth {
                Some(max_depth) => scanner.max_depth(max_depth),
                None => scanner,
            })
    }
}
