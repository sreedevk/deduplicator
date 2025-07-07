use std::sync::mpsc::Sender;
use std::time::Duration;

use anyhow::{Context, Result};
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::widgets::Paragraph;
use ratatui::{DefaultTerminal, Frame};

use crate::pipeline::Message;

pub struct Tui {
    app_tx: Sender<Message>,
}

impl Tui {
    pub fn new(app_tx: Sender<Message>) -> Self {
        Self { app_tx }
    }

    pub fn start(&self) -> Result<()> {
        let terminal = ratatui::init();
        self.run(terminal).expect("ui loop failed.");
        ratatui::restore();

        Ok(())
    }

    fn poll_events() -> Result<Message> {
        match event::poll(Duration::from_millis(250)).context("event polling failed.")? {
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
        let handleable_event = Self::poll_events();

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

    fn draw(frame: &mut Frame) {
        frame.render_widget(Paragraph::new("Hello, World"), frame.area());
    }

    fn run(&self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(Self::draw)?;
            match self.handle_events() {
                Ok(Message::Exit) => break,
                _ => {}
            }
        }

        Ok(())
    }
}
