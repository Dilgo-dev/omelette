use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::app::{App, Cell, CellStatus, GotoFocus, Mode};

const OMNI_RED: Color = Color::Rgb(178, 34, 34);
const OMNI_BG: Color = Color::Rgb(20, 20, 20);
const OMNI_INK: Color = Color::Rgb(245, 240, 232);

const GUTTER: &str = "\u{258c}";
const CURSOR: &str = "\u{2588}";

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    let bg = Block::default().style(Style::default().bg(OMNI_BG));
    f.render_widget(bg, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    draw_header(f, chunks[0], app);
    draw_notebook(f, chunks[1], app);
    draw_footer(f, chunks[2], app);

    if app.mode == Mode::Goto {
        draw_goto(f, area, app);
    }
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let label = app.current().map_or("no connection", |c| c.label.as_str());
    let engine = app.current().map_or("none", |c| match c.engine {
        omelette::core::engine::Engine::Sqlite => "sqlite",
        omelette::core::engine::Engine::Postgres => "postgres",
        omelette::core::engine::Engine::Mysql => "mysql",
        omelette::core::engine::Engine::Mongo => "mongo",
    });
    let tables_list: String = if app.tables.is_empty() {
        "(none)".to_owned()
    } else {
        app.tables
            .iter()
            .take(6)
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let width = area.width as usize;
    let left = format!("\u{2500}[ {label} \u{00b7} {engine} ]");
    let right = format!("tables: {tables_list} \u{2500}");
    let middle_len = width.saturating_sub(left.chars().count() + right.chars().count());
    let middle: String = std::iter::repeat_n('\u{2500}', middle_len).collect();
    let header_line = format!("{left}{middle}{right}");

    let title_line = Line::from(vec![
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
    ]);

    let p = Paragraph::new(vec![
        title_line,
        Line::from(Span::styled(header_line, Style::default().fg(OMNI_INK))),
    ])
    .style(Style::default().bg(OMNI_BG));
    f.render_widget(p, area);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let base = " up/down: scroll \u{00b7} F5: run \u{00b7} n: new cell \u{00b7} g: goto \u{00b7} Esc: leave \u{00b7} q: quit";
    let line = app
        .status
        .as_ref()
        .map_or_else(|| base.to_owned(), |s| format!("{base}    [{s}]"));
    let p = Paragraph::new(line).style(Style::default().fg(OMNI_RED).bg(OMNI_BG));
    f.render_widget(p, area);
}

fn draw_notebook(f: &mut Frame, area: Rect, app: &App) {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let inner_width = area.width.saturating_sub(2) as usize;

    for (idx, cell) in app.cells.iter().enumerate() {
        push_finalized_cell(&mut lines, idx + 1, cell, inner_width);
    }
    push_active_cell(&mut lines, app.cells.len() + 1, &app.active_query);

    let total = lines.len() as u16;
    let visible = area.height;
    let max_offset = total.saturating_sub(visible);
    let offset = max_offset.saturating_sub(app.scroll_offset.min(max_offset));

    let p = Paragraph::new(lines)
        .style(Style::default().bg(OMNI_BG).fg(OMNI_INK))
        .wrap(Wrap { trim: false })
        .scroll((offset, 0));
    f.render_widget(p, area);
}

fn gutter_span() -> Span<'static> {
    Span::styled(GUTTER.to_owned(), Style::default().fg(OMNI_RED))
}

fn push_finalized_cell(lines: &mut Vec<Line<'static>>, n: usize, cell: &Cell, width: usize) {
    let status_str = match cell.status {
        CellStatus::Ok => format!("{} ms", cell.duration_ms),
        CellStatus::Error => "error".to_owned(),
    };

    let query_first_line = cell.query.lines().next().unwrap_or("").to_owned();
    let label = format!(" [{n}] ");
    let used = 1 + label.chars().count() + query_first_line.chars().count();
    let pad = width.saturating_sub(used + status_str.chars().count() + 1);
    let padding: String = std::iter::repeat_n(' ', pad).collect();

    lines.push(Line::from(vec![
        gutter_span(),
        Span::styled(
            label,
            Style::default().fg(OMNI_RED).add_modifier(Modifier::BOLD),
        ),
        Span::styled(query_first_line, Style::default().fg(OMNI_INK)),
        Span::raw(padding),
        Span::styled(status_str, Style::default().fg(OMNI_INK)),
        Span::raw(" "),
    ]));

    for extra in cell.query.lines().skip(1) {
        lines.push(Line::from(vec![
            gutter_span(),
            Span::raw("     "),
            Span::styled(extra.to_owned(), Style::default().fg(OMNI_INK)),
        ]));
    }

    lines.push(Line::from(vec![gutter_span()]));

    if let Some(err) = &cell.error {
        for line in err.lines() {
            lines.push(Line::from(vec![
                gutter_span(),
                Span::raw("     "),
                Span::styled(line.to_owned(), Style::default().fg(OMNI_RED)),
            ]));
        }
        lines.push(Line::from(vec![gutter_span()]));
    } else if let Some(qr) = &cell.result {
        if qr.columns.is_empty() {
            lines.push(Line::from(vec![
                gutter_span(),
                Span::raw("     "),
                Span::styled("(no columns)".to_owned(), Style::default().fg(OMNI_INK)),
            ]));
        } else {
            let widths = compute_widths(qr);
            let header: String = qr
                .columns
                .iter()
                .zip(widths.iter())
                .map(|(c, w)| pad_cell(c, *w))
                .collect::<Vec<_>>()
                .join("  ");
            lines.push(Line::from(vec![
                gutter_span(),
                Span::raw("     "),
                Span::styled(
                    header,
                    Style::default().fg(OMNI_INK).add_modifier(Modifier::BOLD),
                ),
            ]));
            for row in qr.rows.iter().take(20) {
                let row_str: String = row
                    .iter()
                    .zip(widths.iter())
                    .map(|(v, w)| pad_cell(&json_to_string(v), *w))
                    .collect::<Vec<_>>()
                    .join("  ");
                lines.push(Line::from(vec![
                    gutter_span(),
                    Span::raw("     "),
                    Span::styled(row_str, Style::default().fg(OMNI_INK)),
                ]));
            }
        }
        lines.push(Line::from(vec![gutter_span()]));
        let n_rows = qr.rows.len();
        let label = if n_rows == 1 {
            " 1 row ".to_owned()
        } else {
            format!(" {n_rows} rows ")
        };
        let dashes = width.saturating_sub(5 + label.chars().count() + 1);
        let line: String = format!(
            "\u{2500}\u{2500}\u{2500}{label}{}",
            std::iter::repeat_n('\u{2500}', dashes).collect::<String>()
        );
        lines.push(Line::from(vec![
            gutter_span(),
            Span::raw("  "),
            Span::styled(line, Style::default().fg(OMNI_INK)),
        ]));
        lines.push(Line::from(vec![gutter_span()]));
    }
}

fn push_active_cell(lines: &mut Vec<Line<'static>>, n: usize, buffer: &str) {
    let label = format!(" [{n}] ");
    let mut buf_lines = buffer.split('\n');
    let first = buf_lines.next().unwrap_or("").to_owned();
    let suffix = "(editing)";
    lines.push(Line::from(vec![
        gutter_span(),
        Span::styled(
            label,
            Style::default().fg(OMNI_RED).add_modifier(Modifier::BOLD),
        ),
        Span::styled(first, Style::default().fg(OMNI_INK)),
        Span::styled(CURSOR.to_owned(), Style::default().fg(OMNI_RED)),
        Span::raw("  "),
        Span::styled(suffix.to_owned(), Style::default().fg(OMNI_INK)),
    ]));
    for extra in buf_lines {
        lines.push(Line::from(vec![
            gutter_span(),
            Span::raw("     "),
            Span::styled(extra.to_owned(), Style::default().fg(OMNI_INK)),
        ]));
    }
    lines.push(Line::from(vec![gutter_span()]));
}

fn compute_widths(qr: &omelette::core::result::QueryResult) -> Vec<usize> {
    let mut widths: Vec<usize> = qr
        .columns
        .iter()
        .map(|c| c.chars().count().min(24))
        .collect();
    for row in &qr.rows {
        for (i, v) in row.iter().enumerate() {
            if let Some(w) = widths.get_mut(i) {
                let s = json_to_string(v);
                *w = (*w).max(s.chars().count().min(24));
            }
        }
    }
    widths
}

fn pad_cell(s: &str, w: usize) -> String {
    let truncated: String = s.chars().take(w).collect();
    let len = truncated.chars().count();
    let pad = w.saturating_sub(len);
    format!("{truncated}{}", " ".repeat(pad))
}

fn json_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => "NULL".to_owned(),
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn draw_goto(f: &mut Frame, area: Rect, app: &App) {
    let popup = centered_rect(60, 70, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" goto ")
        .style(Style::default().bg(OMNI_BG).fg(OMNI_INK))
        .border_style(Style::default().fg(OMNI_RED));
    f.render_widget(block, popup);

    let inner = Rect {
        x: popup.x + 1,
        y: popup.y + 1,
        width: popup.width.saturating_sub(2),
        height: popup.height.saturating_sub(2),
    };

    let halves = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let conn_lines: Vec<Line> = app
        .connections
        .connections
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let active = app.goto_focus == GotoFocus::Connections && i == app.goto_conn_idx;
            let style = if active {
                Style::default()
                    .fg(OMNI_INK)
                    .bg(OMNI_RED)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(OMNI_INK)
            };
            Line::from(Span::styled(format!(" {} ", c.label), style))
        })
        .collect();

    let conn_p = Paragraph::new(conn_lines).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .title(" connections ")
            .style(Style::default().fg(OMNI_INK)),
    );
    f.render_widget(conn_p, halves[0]);

    let table_lines: Vec<Line> = app
        .tables
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let active = app.goto_focus == GotoFocus::Tables && i == app.goto_table_idx;
            let style = if active {
                Style::default()
                    .fg(OMNI_INK)
                    .bg(OMNI_RED)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(OMNI_INK)
            };
            Line::from(Span::styled(format!(" {} ", t.name), style))
        })
        .collect();

    let table_p = Paragraph::new(table_lines).block(
        Block::default()
            .borders(Borders::NONE)
            .title(" tables ")
            .style(Style::default().fg(OMNI_INK)),
    );
    f.render_widget(table_p, halves[1]);
}

const fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_width = r.width * percent_x / 100;
    let popup_height = r.height * percent_y / 100;
    let x = r.x + (r.width.saturating_sub(popup_width)) / 2;
    let y = r.y + (r.height.saturating_sub(popup_height)) / 2;
    Rect {
        x,
        y,
        width: popup_width,
        height: popup_height,
    }
}
