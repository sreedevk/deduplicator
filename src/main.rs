mod fileinfo;
mod formatter;
mod interactive;
mod params;
mod pipeline;
mod processor;
mod scanner;

use self::{formatter::Formatter, interactive::Interactive};
use anyhow::Result;
use clap::Parser;
use params::Params;

fn main() -> Result<()> {
    let params = Params::parse();
    let report = pipeline::run(&params)?;

    match params.interactive {
        false => Formatter::print(&report, &params),
        true => Interactive::init(&report.groups, &params)?,
    }

    Ok(())
}
