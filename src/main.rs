mod fileinfo;
mod processor;
mod scanner;
mod params;
mod formatter;
mod interative;

use anyhow::Result;
use params::Params;
use scanner::Scanner;
use processor::Processor;
use clap::Parser;
use formatter::Formatter;

fn main() -> Result<()> {
    let app_args = Params::parse();
    let scan_results = Scanner::build(&app_args)?.scan()?;
    let processor = Processor::new(scan_results);
    let results = processor.sizewise()?.hashwise()?;

    Formatter::print(results.files, &app_args)?;
    Ok(())
}
