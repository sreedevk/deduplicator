mod event_handler;
mod events;
mod ui;

use crate::database;
use crate::output;
use crate::params::Params;
use crate::scanner;
use anyhow::{anyhow, Result};
use std::time::Duration;
use crossterm::{event, execute, terminal};
use event_handler::EventHandler;
use std::io;
use std::thread;
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Widget},
    Terminal,
};
use ui::Ui;

pub struct App;

impl App {
    pub fn init(app_args: &Params) -> Result<()> {
        let connection = database::get_connection(&app_args)?;
        let duplicates = scanner::duplicates(&app_args, &connection)?;
        let mut term = Self::init_terminal()?;

        Self::init_render_loop(&mut term)?;
        Self::cleanup(&mut term)?;

        output::print(duplicates, &app_args); /* TODO: APP TUI INIT FUNCTION */
        Ok(())
    }

    fn cleanup(term: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        terminal::disable_raw_mode()?;
        execute!(
            term.backend_mut(),
            terminal::LeaveAlternateScreen,
            event::DisableMouseCapture
        )?;

        term.show_cursor()?;
        Ok(())
    }

    fn render_cycle(term: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        match EventHandler::init()? {
            events::Event::Noop => Ui::render_frame(term),
            events::Event::Exit => Err(anyhow!("Exit")),
        }
    }

    fn init_render_loop(term: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            match Self::render_cycle(term) {
                Ok(_) => continue,
                Err(_) => break,
            }
        }

        Ok(())
    }

    fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            event::EnableMouseCapture
        )?;
        let backend = CrosstermBackend::new(stdout);
        Ok(Terminal::new(backend)?)
    }
}
