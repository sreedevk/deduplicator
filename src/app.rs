#![allow(unused)]

use std::{io, thread, time::Duration};
use anyhow::{anyhow, Result};
use crossterm::{event, execute, terminal};
use crate::database;
use crate::output;
use crate::params::Params;
use crate::scanner;

pub struct App;

impl App {
    pub fn init(app_args: &Params) -> Result<()> {
        let connection = database::get_connection(app_args)?;
        let duplicates = scanner::duplicates(app_args, &connection)?;

        match app_args.interactive {
            true => output::interactive(duplicates, app_args),
            false => output::print(duplicates, app_args)
        }

        Ok(())
    }
}
