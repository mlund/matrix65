use crate::textui::centered_rect;
use tui::{
    backend::{Backend},
    layout::{Alignment},
    style::{Modifier, Style, Color},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Paragraph, Clear,
    },
    Frame,
};

/// Handles actions on selected files, e.g
/// running, downloading, etc.

pub fn render_prg_action_widget<B: Backend>(f: &mut Frame<B>) {
    let area = centered_rect(35, 30, f.size());
    let block = Block::default()
        .title(Span::styled(
            "PRG action",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ))
        .style(Style::default().bg(Color::Red))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let text = vec![
        Spans::from(Span::styled(
            "Run (r)",
            Style::default().fg(Color::White),
        )),
        Spans::from(Span::styled(
            "Reset & Run (R)",
            Style::default().fg(Color::White),
        )),
        Spans::from(Span::styled("Cancel", Style::default().fg(Color::White))),
    ];
    let paragraph = Paragraph::new(text.clone())
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(Clear, area);
    //this clears out the background
    f.render_widget(paragraph, area);
}
