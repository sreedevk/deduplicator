mod fileinfo;
mod formatter;
mod interactive;
mod params;
mod processor;
mod scanner;
mod server;

use self::{formatter::Formatter, interactive::Interactive, server::Server};
use anyhow::Result;
use clap::Parser;
use params::Params;
use std::sync::atomic::Ordering;

fn main() -> Result<()> {
    let app_args = Params::parse();
    let server = Server::new(app_args.clone());

    server.start()?;

    match app_args.interactive {
        false => {
            Formatter::print(
                server.hw_duplicate_set,
                server.max_file_path_len.load(Ordering::Acquire),
                &app_args,
            );
        }
        true => {
            Interactive::init(server.hw_duplicate_set, &app_args)?;
        }
    };

    Ok(())
}
