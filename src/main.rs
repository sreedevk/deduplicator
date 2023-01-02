mod params;
mod database;
mod output;
mod scanner;
mod app;

use anyhow::Result;
use clap::Parser;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    App::init(&params::Params::parse())
}
