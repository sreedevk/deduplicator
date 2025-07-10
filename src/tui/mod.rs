use std::sync::mpsc::Sender;
use std::time::Duration;

use anyhow::{Context, Result};
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::{DefaultTerminal, Frame};

use crate::server::{Message, Server};
use std::sync::Arc;

pub struct Tui {
    app_tx: Sender<Message>,
    server: Arc<Server>,
}

impl Tui {
    pub fn new(app_tx: Sender<Message>, server: Arc<Server>) -> Self {
        Self { app_tx, server }
    }

    pub fn start(&mut self) -> Result<()> {
        let terminal = ratatui::init();
        self.run(terminal).expect("ui loop failed.");
        ratatui::restore();

        Ok(())
    }

    fn poll_events() -> Result<Message> {
        match event::poll(Duration::from_millis(100)).context("event polling failed.")? {
            true => match event::read().context("event read failed.")? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => Ok(Message::Exit),
                    _ => Ok(Message::None),
                },
                _ => Ok(Message::None),
            },
            false => Ok(Message::None),
        }
    }

    fn handle_events(&self) -> Result<Message> {
        match Self::poll_events() {
            Ok(Message::Exit) => {
                self.app_tx
                    .send(Message::Exit)
                    .expect("app event send failed.");
                Ok(Message::Exit)
            }
            _ => Ok(Message::None),
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let listitems = self
            .server
            .dupstore
            .entries()
            .iter()
            .flatten()
            .map(|f| ListItem::new(format!("{}", f.path)))
            .collect::<Vec<ListItem>>();

        let list = List::new(listitems);
        frame.render_widget(list.block(Block::new().borders(Borders::ALL)), frame.area());
    }

    fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;
            match self.handle_events() {
                Ok(Message::Exit) => break,
                _ => {}
            }
        }

        Ok(())
    }
}
