use crate::textui::centered_rect;
use crossterm::event::KeyCode;
use std::io;

use tui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState},
    Frame,
};

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        let list = StatefulList {
            state: ListState::default(),
            items,
        };
        list
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

    fn _unselect(&mut self) {
        self.state.select(None);
    }

    pub fn keypress(&mut self, key: crossterm::event::KeyCode) -> io::Result<()> {
        match key {
            KeyCode::Down => self.next(),
            KeyCode::Up => self.previous(),
            _ => {}
        }
        Ok(())
    }
}

/// Handles actions on selected files, e.g running, downloading, etc.
pub fn render_prg_widget<B: Backend>(f: &mut Frame<B>, action_list: &mut StatefulList<String>) {
    let area = centered_rect(15, 15, f.size());
    let block = Block::default()
        .title(Span::styled(
            "File actions",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ))
        .style(Style::default().bg(Color::Blue))
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
