mod database;
mod scanner;
mod cli;

use clap::Parser;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    database::setup()?;
    let duplicates = scanner::duplicates(cli::App::parse())?;
    dbg!(duplicates);

    Ok(())
}
