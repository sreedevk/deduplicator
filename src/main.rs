mod app;
mod file_manager;
mod output;
mod params;
mod scanner;
mod filters;

use anyhow::Result;
use app::App;
use clap::Parser;

fn main() -> Result<()> {
    App::init(&params::Params::parse())
}
