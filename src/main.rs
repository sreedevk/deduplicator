mod app;
mod database;
mod file_manager;
mod output;
mod params;
mod scanner;

use anyhow::Result;
use app::App;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    App::init(&params::Params::parse())
}
