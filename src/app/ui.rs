use anyhow::Result;
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Widget},
    Frame, Terminal,
};

pub struct Ui;

impl Ui {
    fn generate_file_list() -> impl Widget {
        let tasks: Vec<ListItem> = vec!["Sreedev"; 100]
            .into_iter()
            .map(|item| ListItem::new(vec![Spans::from(Span::raw(item))]))
            .collect();

        List::new(tasks)
            .block(Block::default().borders(Borders::ALL).title("List"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ")
    }

    fn generate_info_bar() -> impl Widget {
        Block::default().title("Description").borders(Borders::ALL)
    }

    fn generate_file_desc() -> impl Widget {
        Block::default().title("Description").borders(Borders::ALL)
    }

    pub fn render_frame(term: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        term.draw(|f| {
            let windows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Ratio(2, 16), Constraint::Ratio(14, 16)].as_ref())
                .split(f.size());

            let subwindows = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)].as_ref())
                .split(windows[1]);

            f.render_widget(Self::generate_info_bar(), windows[0]);
            f.render_widget(Self::generate_file_list(), subwindows[0]);
            f.render_widget(Self::generate_file_desc(), subwindows[1]);
        })?;
        Ok(())
    }
}
