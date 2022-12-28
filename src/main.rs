mod database;
mod scanner;
mod cli;
mod output;

use clap::Parser;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let connection = sqlite::open(":memory:")?;
    database::setup(&connection)?;

    let duplicates = scanner::duplicates(cli::App::parse(), &connection)?;
    output::print(duplicates);

    Ok(())
}
