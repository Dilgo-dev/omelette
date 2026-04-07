mod app;
mod connections;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
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
    if let Some(arg) = std::env::args().nth(1) {
        app.open_file(std::path::Path::new(&arg))?;
    }
    let res = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn run<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let tick = Duration::from_millis(40);
    while !app.should_quit {
        terminal.draw(|f| ui::draw(f, app))?;
        if event::poll(tick)? {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                if app.splash_active() {
                    app.dismiss_splash();
                } else {
                    handle_key(app, key.code, key.modifiers)?;
                }
            }
        } else {
            app.tick_splash();
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode, mods: KeyModifiers) -> Result<()> {
    if mods.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('b') {
        app.toggle_explorer();
        return Ok(());
    }
    if mods.contains(KeyModifiers::CONTROL)
        && code == KeyCode::Char(' ')
        && app.mode == Mode::Normal
        && !(app.explorer_open && app.explorer_focused)
    {
        app.complete_open();
        return Ok(());
    }
    match app.mode {
        Mode::Complete => handle_complete(app, code),
        Mode::Preview => handle_preview(app, code),
        Mode::Normal if app.explorer_open && app.explorer_focused => handle_explorer(app, code),
        Mode::Normal => handle_notebook(app, code)?,
        Mode::Goto => handle_goto(app, code),
    }
    Ok(())
}

fn handle_explorer(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('j') | KeyCode::Down => app.explorer_next(),
        KeyCode::Char('k') | KeyCode::Up => app.explorer_prev(),
        KeyCode::Char(' ') | KeyCode::Right | KeyCode::Left => app.explorer_toggle_expand(),
        KeyCode::Enter => app.explorer_pick(),
        KeyCode::Char('r') => app.explorer_run_quick(),
        KeyCode::Tab | KeyCode::Esc => app.explorer_toggle_focus(),
        KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
}

fn handle_complete(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab => app.complete_next(),
        KeyCode::Char('k') | KeyCode::Up => app.complete_prev(),
        KeyCode::Enter => app.complete_commit(),
        KeyCode::Esc => app.complete_close(),
        _ => {}
    }
}

fn handle_preview(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => app.preview_commit(),
        KeyCode::Esc | KeyCode::Char('q') => app.preview_close(),
        _ => {}
    }
}

fn handle_notebook(app: &mut App, code: KeyCode) -> Result<()> {
    if code == KeyCode::Tab && app.explorer_open {
        app.explorer_toggle_focus();
        return Ok(());
    }
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
