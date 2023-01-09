#![allow(unused)] // TODO: remove this once TUI is implemented
mod app;
mod database;
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
