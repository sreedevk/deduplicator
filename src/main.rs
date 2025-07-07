mod app;
mod fileinfo;
mod formatter;
mod interactive;
mod params;
mod processor;
mod scanner;

/* version 2.0 modules*/
mod cli;
mod server;
mod tui;

use anyhow::Result;

use self::app::App;
use clap::Parser;
use params::Params;
use std::sync::Arc;
// use formatter::Formatter;
// use processor::Processor;
// use scanner::Scanner;

fn main() -> Result<()> {
    let app_args = Params::parse();
    // let scan_results = Scanner::build(&app_args)?.scan()?;
    // let processor = Processor::new(scan_results);
    // let results = processor.sizewise()?.hashwise()?;

    // match app_args.interactive {
    //     false => Formatter::print(results.files, &app_args)?,
    //     true => interactive::init(results.files, &app_args)?,
    // }

    App::new(Arc::new(app_args))
        .start()
        .expect("app init failed.");

    Ok(())
}
