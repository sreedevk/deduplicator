mod app;
mod params;
mod formatter;
mod interative;

use anyhow::Result;
use app::App;
use clap::Parser;

fn main() -> Result<()> {
    App::init(&params::Params::parse())
}
