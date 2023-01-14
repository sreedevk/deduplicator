mod app;
mod database;
mod output;
mod params;
mod scanner;
mod file_manager;

use anyhow::Result;
use app::App;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    App::init(&params::Params::parse())
}
