// copyright 2022 mikael lund aka wombat
//
// licensed under the apache license, version 2.0 (the "license");
// you may not use this file except in compliance with the license.
// you may obtain a copy of the license at
//
//     http://www.apache.org/licenses/license-2.0
//
// unless required by applicable law or agreed to in writing, software
// distributed under the license is distributed on an "as is" basis,
// without warranties or conditions of any kind, either express or implied.
// see the license for the specific language governing permissions and
// limitations under the license.

use crate::textui::{centered_rect, StatefulList};
use tui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem},
    Frame,
};

/// Handles actions on selected files, e.g running, downloading, etc.
pub fn render_cbm_selector_widget<B: Backend>(
    f: &mut Frame<B>,
    file_list: &mut StatefulList<String>,
    busy: bool,
) {
    let background_color = match busy {
        true => Color::DarkGray,
        false => Color::Blue,
    };
    let area = centered_rect(20, 20, f.size());
    let block = Block::default()
        .title(Span::styled(
            "Select file on CBM disk",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ))
        .style(Style::default().bg(background_color))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let items: Vec<ListItem> = file_list
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
    f.render_stateful_widget(list, area, &mut file_list.state);
}
