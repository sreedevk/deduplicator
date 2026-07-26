mod cache;
mod fileinfo;
mod formatter;
mod interactive;
mod params;
mod pipeline;
mod processor;
mod resolver;
mod scanner;

use self::{formatter::Formatter, interactive::Interactive};
use anyhow::Result;
use clap::Parser;
use params::Params;

fn main() -> Result<()> {
    let params = Params::parse();
    let report = pipeline::run(&params)?;

    match params.keep {
        Some(strategy) => resolver::run(&report, strategy, params.force, &params)?,
        None => match params.interactive {
            true => Interactive::init(&report.groups, &params)?,
            false => Formatter::print(&report, &params),
        },
    }

    Ok(())
}
