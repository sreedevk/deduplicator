mod cache;
mod fileinfo;
mod formatter;
mod interactive;
mod params;
mod pipeline;
mod processor;
mod resolver;
mod scanner;
mod tui;

use self::{formatter::Formatter, interactive::Interactive};
use anyhow::Result;
use clap::Parser;
use params::Params;

fn main() -> Result<()> {
    let params = Params::parse();
    let report = pipeline::run(&params)?;

    match (params.tui, params.keep, params.interactive) {
        (true, _, _) => tui::run(report, &params)?,
        (false, Some(strategy), _) => resolver::run(&report, strategy, params.force, &params)?,
        (false, None, true) => Interactive::init(&report.groups, &params)?,
        (false, None, false) => Formatter::print(&report, &params),
    }

    Ok(())
}
