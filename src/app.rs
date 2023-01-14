use crate::database;
use crate::output;
use crate::params::Params;
use crate::scanner;
use anyhow::Result;

pub struct App;

impl App {
    pub fn init(app_args: &Params) -> Result<()> {
        let connection = database::get_connection(app_args)?;
        let duplicates = scanner::duplicates(app_args, &connection)?;
        match app_args.interactive {
            true => output::interactive(duplicates, app_args),
            false => output::print(duplicates, app_args),
        }

        Ok(())
    }
}
