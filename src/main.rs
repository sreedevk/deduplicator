mod fileinfo;
mod formatter;
mod interactive;
mod params;
mod processor;
mod scanner;

use anyhow::Result;
use clap::Parser;
use formatter::Formatter;
use params::Params;
use processor::Processor;
use scanner::Scanner;

fn main() -> Result<()> {
    let app_args = Params::parse();
    let scan_results = Scanner::build(&app_args)?.scan()?;
    let processor = Processor::new(scan_results);
    let results = processor.sizewise()?.hashwise()?;

    match app_args.interactive {
        false => { Formatter::print(results.files, &app_args)?; }
        true => { interactive::init(results.files, &app_args)?; }
    }

    Ok(())
}
