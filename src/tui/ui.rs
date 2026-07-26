use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::params::Params;
use crate::formatter::Formatter;

use super::app::{strategy_label, App, Focus, Popup, StrategyScope, STRATEGIES};

pub fn draw(frame: &mut Frame, app: &App, params: &Params) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(rows[0]);

    draw_groups(frame, app, panes[0]);
    draw_files(frame, app, params, panes[1]);
    draw_footer(frame, app, rows[1]);

    match app.popup {
        Popup::Strategy { scope } => draw_strategy_popup(frame, app, scope),
        Popup::ConfirmDelete => draw_confirm_popup(frame, app),
        Popup::None => {}
    }
}

fn pane_block(title: &str, focused: bool) -> Block<'_> {
    let block = Block::default().borders(Borders::ALL).title(title.to_string());
    match focused {
        true => block.border_style(Style::new().yellow()),
        false => block,
    }
}

fn draw_groups(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .groups
        .iter()
        .enumerate()
        .map(|(i, g)| {
            ListItem::new(format!(
                "{:>3}  {} files  {}",
                i + 1,
                g.files.len(),
                bytesize::ByteSize::b(g.size())
            ))
        })
        .collect();

    let list = List::new(items)
        .block(pane_block("Groups", app.focus == Focus::Groups))
        .highlight_style(Style::new().reversed());

    let mut state = ListState::default();
    state.select(Some(app.group_cursor));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_files(frame: &mut Frame, app: &App, params: &Params, area: Rect) {
    let title = format!(
        "Group {}/{}",
        (app.group_cursor + 1).min(app.groups.len().max(1)),
        app.groups.len()
    );

    let items: Vec<ListItem> = match app.groups.get(app.group_cursor) {
        Some(g) => g
            .files
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let box_char = if g.marked[i] { "[x]" } else { "[ ]" };
                let path = Formatter::human_path(f, params, 0).unwrap_or_default();
                let size = Formatter::human_filesize(f).unwrap_or_default();
                let mtime = Formatter::human_mtime(f).unwrap_or_default();
                let line = format!("{box_char} {}  {}  {}", path.trim_end(), size.trim_start(), mtime);
                match g.marked[i] {
                    true => ListItem::new(Line::from(line).style(Style::new().red())),
                    false => ListItem::new(line),
                }
            })
            .collect(),
        None => Vec::new(),
    };

    let list = List::new(items)
        .block(pane_block(&title, app.focus == Focus::Files))
        .highlight_style(Style::new().reversed());

    let mut state = ListState::default();
    state.select(Some(app.file_cursor));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let hints = "[Tab]pane [Space]mark [s]group [S]all [o]pen [d]elete [q]uit";
    let left = match &app.status {
        Some(s) => s.clone(),
        None => format!(
            "marked {} · reclaim {}",
            app.marked_count(),
            bytesize::ByteSize::b(app.reclaimable_bytes())
        ),
    };
    let text = format!("{left}   {hints}");
    frame.render_widget(Paragraph::new(text), area);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

fn draw_strategy_popup(frame: &mut Frame, app: &App, scope: StrategyScope) {
    let title = match scope {
        StrategyScope::CurrentGroup => "Keep in this group",
        StrategyScope::AllGroups => "Keep in ALL groups",
    };
    let items: Vec<ListItem> = STRATEGIES
        .iter()
        .map(|s| ListItem::new(strategy_label(*s)))
        .collect();

    let area = centered_rect(28, (STRATEGIES.len() as u16) + 2, frame.area());
    frame.render_widget(Clear, area);

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::new().reversed());

    let mut state = ListState::default();
    state.select(Some(app.strategy_cursor));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_confirm_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(48, 3, frame.area());
    frame.render_widget(Clear, area);
    let text = format!(
        "Delete {} files, reclaim {}? [y/N]",
        app.marked_count(),
        bytesize::ByteSize::b(app.reclaimable_bytes())
    );
    frame.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Confirm")),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fileinfo::FileInfo;
    use crate::pipeline::DuplicateGroup;
    use crate::tui::app::App;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use std::path::PathBuf;
    use std::time::{Duration, UNIX_EPOCH};

    fn sample_app() -> App {
        let files = vec![
            FileInfo {
                path: PathBuf::from("/tmp/report.pdf").into_boxed_path(),
                size: 4_000_000,
                modified: UNIX_EPOCH + Duration::from_secs(1_700_000_000),
            },
            FileInfo {
                path: PathBuf::from("/tmp/report-copy.pdf").into_boxed_path(),
                size: 4_000_000,
                modified: UNIX_EPOCH + Duration::from_secs(1_700_100_000),
            },
        ];
        App::from_groups(vec![DuplicateGroup { hash: 0, files }])
    }

    #[test]
    fn draw_renders_without_panic_and_shows_content() {
        let app = sample_app();
        let params = crate::params::Params::default();
        let backend = TestBackend::new(100, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| draw(f, &app, &params)).unwrap();

        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect();

        assert!(text.contains("Groups"));
        assert!(text.contains("report"));
        assert!(text.contains("marked"));
    }
}
