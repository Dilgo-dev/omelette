use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

const OMNI_RED: Color = Color::Rgb(178, 34, 34);
const OMNI_BG: Color = Color::Rgb(20, 20, 20);
const OMNI_INK: Color = Color::Rgb(245, 240, 232);

pub fn draw(f: &mut Frame, _app: &App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled("omni", Style::default().fg(OMNI_INK).add_modifier(Modifier::BOLD)),
        Span::styled("db", Style::default().fg(OMNI_RED).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(
            "one TUI for every database",
            Style::default().fg(OMNI_INK).add_modifier(Modifier::ITALIC),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).style(Style::default().bg(OMNI_BG)));
    f.render_widget(title, chunks[0]);

    let body = Paragraph::new("Press q to quit.")
        .alignment(Alignment::Center)
        .style(Style::default().bg(OMNI_BG).fg(OMNI_INK))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(body, chunks[1]);

    let help = Paragraph::new("q: quit").style(Style::default().fg(OMNI_RED));
    f.render_widget(help, chunks[2]);
}
