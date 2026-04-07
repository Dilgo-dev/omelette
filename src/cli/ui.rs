use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table,
};

use crate::app::{App, Focus, Mode};
use omelette::core::engine::Engine;

const OMNI_RED: Color = Color::Rgb(178, 34, 34);
const OMNI_BG: Color = Color::Rgb(20, 20, 20);
const OMNI_INK: Color = Color::Rgb(245, 240, 232);
const DIM: Color = Color::Rgb(120, 120, 120);

const SQL_TEAL: Color = Color::Rgb(78, 205, 196);
const PG_BLUE: Color = Color::Rgb(70, 130, 200);
const MY_ORANGE: Color = Color::Rgb(255, 140, 50);
const MONGO_GREEN: Color = Color::Rgb(120, 200, 80);

const fn engine_color(e: Engine) -> Color {
    match e {
        Engine::Sqlite => SQL_TEAL,
        Engine::Postgres => PG_BLUE,
        Engine::Mysql => MY_ORANGE,
        Engine::Mongo => MONGO_GREEN,
    }
}

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    let bg = Block::default().style(Style::default().bg(OMNI_BG));
    f.render_widget(bg, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    draw_title(f, chunks[0]);
    draw_main(f, chunks[1], app);
    draw_help(f, chunks[2], app);

    match app.mode {
        Mode::ConfirmDelete => draw_confirm_delete(f, area, app),
        Mode::Rename => draw_rename(f, area, app),
        Mode::Normal => {}
    }
}

fn draw_title(f: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "omel",
            Style::default().fg(OMNI_INK).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "ette",
            Style::default().fg(OMNI_RED).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            "crack open your databases",
            Style::default().fg(OMNI_INK).add_modifier(Modifier::ITALIC),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(OMNI_BG)),
    );
    f.render_widget(title, area);
}

fn draw_main(f: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Min(0),
        ])
        .split(area);
    draw_connections_panel(f, cols[0], app);
    draw_schema_panel(f, cols[1], app);
    draw_body_panel(f, cols[2], app);
}

fn focus_block(title: &str, focused: bool) -> Block<'_> {
    let border = if focused { OMNI_RED } else { DIM };
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(OMNI_BG).fg(OMNI_INK))
        .border_style(Style::default().fg(border))
}

fn draw_connections_panel(f: &mut Frame, area: Rect, app: &App) {
    let block = focus_block(" connections ", app.focus == Focus::Connections);
    if app.connections.connections.is_empty() {
        let p = Paragraph::new("no connection yet\n\npress 'a' to add one")
            .style(Style::default().bg(OMNI_BG).fg(DIM))
            .block(block);
        f.render_widget(p, area);
        return;
    }
    let items: Vec<ListItem> = app
        .connections
        .connections
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let label = if app.mode == Mode::Rename && i == app.selected {
                format!("{}_", app.rename_buffer)
            } else {
                c.label.clone()
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", c.engine.badge()),
                    Style::default()
                        .fg(OMNI_BG)
                        .bg(engine_color(c.engine))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(label, Style::default().fg(OMNI_INK)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(OMNI_RED)
                .fg(OMNI_INK)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    if !app.connections.connections.is_empty() {
        state.select(Some(app.selected));
    }
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_schema_panel(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .tables
        .iter()
        .map(|t| {
            ListItem::new(Line::from(vec![
                Span::styled(" T ", Style::default().fg(OMNI_BG).bg(SQL_TEAL)),
                Span::raw(" "),
                Span::styled(t.name.clone(), Style::default().fg(OMNI_INK)),
            ]))
        })
        .collect();

    let block = focus_block(" schema ", app.focus == Focus::Schema);

    if items.is_empty() {
        let hint = if app.current().is_none() {
            "no connection"
        } else if app.loaded_id.is_none() {
            "press Tab to load"
        } else {
            "(no tables)"
        };
        let p = Paragraph::new(hint)
            .style(Style::default().bg(OMNI_BG).fg(DIM))
            .block(block);
        f.render_widget(p, area);
        return;
    }

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(OMNI_RED)
            .fg(OMNI_INK)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = ListState::default();
    state.select(Some(app.selected_table));
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_body_panel(f: &mut Frame, area: Rect, app: &App) {
    let block = focus_block(" preview ", app.focus == Focus::Preview);
    let Some(qr) = &app.preview else {
        let hint = if app.current().is_none() {
            "no connection".to_owned()
        } else if app.tables.is_empty() {
            "no table to preview".to_owned()
        } else {
            "select a table in the schema panel".to_owned()
        };
        let p = Paragraph::new(hint)
            .style(Style::default().bg(OMNI_BG).fg(DIM))
            .block(block);
        f.render_widget(p, area);
        return;
    };

    if qr.columns.is_empty() {
        let p = Paragraph::new("(no columns)")
            .style(Style::default().bg(OMNI_BG).fg(DIM))
            .block(block);
        f.render_widget(p, area);
        return;
    }

    let col_off = app
        .preview_col_offset
        .min(qr.columns.len().saturating_sub(1));
    let row_off = app.preview_row_offset.min(qr.rows.len().saturating_sub(1));

    let cols: Vec<&str> = qr.columns[col_off..].iter().map(String::as_str).collect();

    let header = Row::new(cols.iter().map(|c| {
        Cell::from(Span::styled(
            (*c).to_owned(),
            Style::default().fg(OMNI_RED).add_modifier(Modifier::BOLD),
        ))
    }));

    let body_rows = qr.rows[row_off..].iter().map(|row| {
        let cells: Vec<Cell> = row[col_off..]
            .iter()
            .map(|v| Cell::from(json_to_string(v)))
            .collect();
        Row::new(cells)
    });

    let widths: Vec<Constraint> = (0..cols.len()).map(|_| Constraint::Length(16)).collect();

    let table = Table::new(body_rows, widths)
        .header(header)
        .block(block)
        .style(Style::default().bg(OMNI_BG).fg(OMNI_INK));

    f.render_widget(table, area);
}

fn json_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => "NULL".to_owned(),
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn draw_help(f: &mut Frame, area: Rect, app: &App) {
    let text = match app.mode {
        Mode::Normal => match app.focus {
            Focus::Connections => "Tab: focus  j/k: move  a: add  r: rename  d: delete  q: quit",
            Focus::Schema => "Tab: focus  j/k: move  R: refresh  q: quit",
            Focus::Preview => "Tab: focus  hjkl: scroll  R: reload  q: quit",
        },
        Mode::ConfirmDelete => "y: confirm delete  n/Esc: cancel",
        Mode::Rename => "type new label  Enter: save  Esc: cancel",
    };
    let line = app
        .status
        .as_ref()
        .map_or_else(|| text.into(), |s| format!("{text}    [{s}]"));
    let p = Paragraph::new(line).style(Style::default().fg(OMNI_RED).bg(OMNI_BG));
    f.render_widget(p, area);
}

fn draw_confirm_delete(f: &mut Frame, area: Rect, app: &App) {
    let popup = centered_rect(50, 7, area);
    f.render_widget(Clear, popup);
    let label = app.current().map_or("?", |c| c.label.as_str());
    let body = format!("delete connection '{label}' ?\n\ny: confirm    n/Esc: cancel");
    let p = Paragraph::new(body)
        .alignment(Alignment::Center)
        .style(Style::default().bg(OMNI_BG).fg(OMNI_INK))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" confirm delete ")
                .style(Style::default().fg(OMNI_RED)),
        );
    f.render_widget(p, popup);
}

fn draw_rename(f: &mut Frame, area: Rect, app: &App) {
    let popup = centered_rect(60, 5, area);
    f.render_widget(Clear, popup);
    let body = format!("{}_", app.rename_buffer);
    let p = Paragraph::new(body)
        .alignment(Alignment::Left)
        .style(Style::default().bg(OMNI_BG).fg(OMNI_INK))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" rename ")
                .style(Style::default().fg(OMNI_RED)),
        );
    f.render_widget(p, popup);
}

const fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_width = r.width * percent_x / 100;
    let x = r.x + (r.width.saturating_sub(popup_width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width: popup_width,
        height,
    }
}
