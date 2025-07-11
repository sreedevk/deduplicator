use std::sync::mpsc::Sender;
use std::time::Duration;

use anyhow::{Context, Result};
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
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
            .flat_map(|val| {
                let mut group = vec![ListItem::new(format!("{:?}", val.key()))];
                val.value().iter().for_each(|f| {
                    group.push(ListItem::new(format!("|-{}", f.path)));
                });
                group
            })
            .collect::<Vec<ListItem>>();

        let list = List::new(listitems.clone());
        let layout_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.area());

        frame.render_widget(
            list.block(Block::new().borders(Borders::ALL)),
            layout_chunks[0],
        );

        frame.render_widget(
            Paragraph::new(String::new()).block(Block::new().borders(Borders::ALL)),
            layout_chunks[1],
        );
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
