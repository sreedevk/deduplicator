pub mod app;
mod ui;

use std::io::{self, Stdout};

use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::Terminal;

use crate::params::Params;
use crate::pipeline::DedupReport;
use app::{App, Key, Outcome};

type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn run(report: DedupReport, params: &Params) -> Result<()> {
    if report.groups.is_empty() {
        println!("No duplicates found matching your search criteria.");
        return Ok(());
    }

    let mut app = App::from_groups(report.groups);
    let mut terminal = setup_terminal()?;
    let result = event_loop(&mut terminal, &mut app, params);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> Result<Tui> {
    install_panic_hook();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Tui) -> Result<()> {
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();
    Ok(())
}

fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original(info);
    }));
}

fn event_loop(terminal: &mut Tui, app: &mut App, params: &Params) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::draw(frame, app, params))?;
        if app.should_quit {
            break;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if let Some(translated) = translate(key) {
                match app.handle_key(translated) {
                    Outcome::Quit => {}
                    Outcome::Continue => {}
                    Outcome::Delete => perform_deletion(app),
                    Outcome::Open(path) => {
                        if let Err(err) = open::that(&path) {
                            app.status = Some(format!("open failed: {err}"));
                        }
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn translate(key: KeyEvent) -> Option<Key> {
    Some(match key.code {
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Tab => Key::Tab,
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Esc,
        KeyCode::Char(' ') => Key::Space,
        KeyCode::Char(c) => Key::Char(c),
        _ => return None,
    })
}

fn perform_deletion(app: &mut App) {
    let paths = app.marked_paths();
    let mut deleted = Vec::new();
    let mut freed: u64 = 0;
    let mut failures: u64 = 0;

    for path in &paths {
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        match std::fs::remove_file(path) {
            Ok(_) => {
                deleted.push(path.clone());
                freed += size;
            }
            Err(_) => failures += 1,
        }
    }

    app.apply_deletion(&deleted);
    app.status = Some(match failures {
        0 => format!("deleted {}, freed {}", deleted.len(), bytesize::ByteSize::b(freed)),
        _ => format!(
            "deleted {}, {failures} failed, freed {}",
            deleted.len(),
            bytesize::ByteSize::b(freed)
        ),
    });
}
