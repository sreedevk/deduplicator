use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyEvent};

use super::events;

pub struct EventHandler;

impl EventHandler {
    pub fn init() -> Result<events::Event> {
        if crossterm::event::poll(Duration::from_millis(10))? {
            match event::read()? {
                event::Event::Key(keycode) => Self::handle_keypress(keycode),
                _ => Ok(events::Event::Noop),
            }
        } else {
            Ok(events::Event::Noop)
        }
    }

    fn handle_keypress(keyevent: KeyEvent) -> Result<events::Event> {
        match keyevent.code {
            KeyCode::Char('q') => Ok(events::Event::Exit),
            _ => Ok(events::Event::Noop),
        }
    }
}
