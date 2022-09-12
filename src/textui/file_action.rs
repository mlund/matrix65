use crate::textui::{centered_rect, StatefulList};

use tui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem},
    Frame,
};

/// Handles actions on selected files, e.g running, downloading, etc.
pub fn render_prg_widget<B: Backend>(
    f: &mut Frame<B>,
    action_list: &mut StatefulList<String>,
    busy: bool,
) {
    let background_color = match busy {
        true => Color::DarkGray,
        false => Color::Blue,
    };
    let area = centered_rect(15, 15, f.size());
    let block = Block::default()
        .title(Span::styled(
            "File actions",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ))
        .style(Style::default().bg(background_color))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let items: Vec<ListItem> = action_list
        .items
        .iter()
        .map(|i| ListItem::new(i.as_str()))
        .collect();
    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("*");

    f.render_widget(Clear, area);
    f.render_stateful_widget(list, area, &mut action_list.state);
}
