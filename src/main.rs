mod params;
mod database;
mod output;
mod scanner;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let app_args = params::Params::parse();
    let connection = database::get_connection(&app_args)?; 
    let duplicates = scanner::duplicates(&app_args, &connection)?;

    output::print(duplicates, &app_args);
    Ok(())
}
