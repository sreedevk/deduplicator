mod fileinfo;
mod formatter;
mod interactive;
mod params;
mod processor;
mod scanner;
mod server;

use anyhow::Result;
use clap::Parser;
use formatter::Formatter;
use params::Params;
use processor::Processor;
use scanner::Scanner;

use self::fileinfo::FileInfo;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

fn main() -> Result<()> {
    let files: Arc<Mutex<Vec<FileInfo>>> = Arc::new(Mutex::new(Vec::new()));
    let app_args = Params::parse();

    // TODO: RACE CONDITION BETWEEN SCANNER AND PROCESSOR
    let file_arc_clone = files.clone();
    let app_args_clone = app_args.clone();

    let scanner_thread = thread::spawn(move || {
        Scanner::build(&app_args_clone)
            .unwrap()
            .scan(file_arc_clone)
            .unwrap();
    });

    let file_arc_clone = files.clone();

    let processor_thread = thread::spawn(move || {
        let mut processor = Processor::new(file_arc_clone);
        processor.sizewise().unwrap();
        processor.hashwise().unwrap();

        (processor.hashwise_results, processor.max_path_len)
    });

    scanner_thread.join().unwrap();
    let results = processor_thread.join().unwrap();

    match app_args.interactive {
        false => Formatter::print(results.0, results.1, &app_args)?,
        true => interactive::init(results.0, &app_args)?,
    }

    Ok(())
}
