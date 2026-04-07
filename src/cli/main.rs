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
        Mode::Normal => handle_notebook(app, code)?,
        Mode::Goto => handle_goto(app, code),
    }
    Ok(())
}

fn handle_notebook(app: &mut App, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::F(5) => app.run_active_cell(),
        KeyCode::Up => app.scroll_up(1),
        KeyCode::Down => app.scroll_down(1),
        KeyCode::PageUp => app.scroll_up(10),
        KeyCode::PageDown => app.scroll_down(10),
        KeyCode::Char('q') if app.active_query.is_empty() => app.should_quit = true,
        KeyCode::Char('g') if app.active_query.is_empty() => app.open_goto(),
        KeyCode::Char('n') if app.active_query.is_empty() => app.new_cell(),
        KeyCode::Char('a') if app.active_query.is_empty() => app.add_placeholder()?,
        KeyCode::Enter => app.active_newline(),
        KeyCode::Backspace => app.active_pop(),
        KeyCode::Char(c) => app.active_push(c),
        _ => {}
    }
    Ok(())
}

fn handle_goto(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.close_goto(),
        KeyCode::Up | KeyCode::Char('k') => app.goto_prev(),
        KeyCode::Down | KeyCode::Char('j') => app.goto_next(),
        KeyCode::Tab => app.goto_toggle(),
        KeyCode::Enter => app.goto_commit(),
        _ => {}
    }
}
