use crate::textui::centered_rect;
use tui::{
    backend::Backend,
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

// pub fn keypress(&key: crossterm::event::KeyCode) -> io::Result<()> {
//     match key {
//         KeyCode::Down => self.next(),
//         KeyCode::Up => self.previous(),
//         KeyCode::Char('r') => self.run(false)?,
//         KeyCode::Char('R') => self.run(true)?,
//         KeyCode::Char('s') => self.sort_filehost(),
//         _ => { }
//     }
//     Ok(())
// }

/// Handles actions on selected files, e.g running, downloading, etc.
pub fn render_prg_widget<B: Backend>(f: &mut Frame<B>, items_str: &[&str]) {
    let area = centered_rect(15, 15, f.size());
    let block = Block::default()
        .title(Span::styled(
            "File actions",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ))
        .style(Style::default().bg(Color::Red))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let items: Vec<ListItem> = items_str.iter().map(|i| ListItem::new(*i)).collect();
    let action_list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    );

    f.render_widget(Clear, area);
    f.render_widget(action_list, area);
}
