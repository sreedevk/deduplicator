mod cli;
mod database;
mod output;
mod scanner;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let connection = sqlite::open("/tmp/deduplicator.db").and_then(|conn| {
        database::setup(&conn).ok();
        Ok(conn)
    })?;

    let duplicates = scanner::duplicates(cli::App::parse(), &connection)?;
    output::print(duplicates);

    Ok(())
}
