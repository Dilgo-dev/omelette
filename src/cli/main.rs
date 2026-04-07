mod app;
mod connections;
mod ui;

use std::io;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use app::{App, Mode};

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let res = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn run<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui::draw(f, app))?;
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            handle_key(app, key.code)?;
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode) -> Result<()> {
    match app.mode {
        Mode::Normal => match code {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => app.select_next(),
            KeyCode::Char('k') | KeyCode::Up => app.select_prev(),
            KeyCode::Char('a') => app.add_placeholder()?,
            KeyCode::Char('d') => {
                if app.current().is_some() {
                    app.mode = Mode::ConfirmDelete;
                }
            }
            KeyCode::Char('r') => app.start_rename(),
            _ => {}
        },
        Mode::ConfirmDelete => match code {
            KeyCode::Char('y') => app.delete_selected()?,
            KeyCode::Char('n') | KeyCode::Esc => app.cancel_mode(),
            _ => {}
        },
        Mode::Rename => match code {
            KeyCode::Enter => app.commit_rename()?,
            KeyCode::Esc => app.cancel_mode(),
            KeyCode::Backspace => {
                app.rename_buffer.pop();
            }
            KeyCode::Char(c) => app.rename_buffer.push(c),
            _ => {}
        },
    }
    Ok(())
}
