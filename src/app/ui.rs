use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Widget},
    Terminal,
};
use anyhow::Result;
use std::io;

pub struct Ui;

impl Ui {
    pub fn render_frame(term: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        term.draw(|f| {
            let size = f.size();
            let block = Block::default().title("Block").borders(Borders::ALL);

            f.render_widget(block, size);
        })?;
        Ok(())
    }
}
