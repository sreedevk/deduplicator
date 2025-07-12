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
    let mut processor = Processor::new(scan_results);

    processor.sizewise()?;
    processor.hashwise()?;

    let results = processor.hashwise_results;

    match app_args.interactive {
        false => Formatter::print(results, processor.max_path_len, &app_args)?,
        true => interactive::init(results, &app_args)?,
    }

    Ok(())
}
