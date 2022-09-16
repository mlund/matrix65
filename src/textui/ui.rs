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

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::filehost;
use crate::textui::StatefulList;
use crate::textui::{App, AppWidgets};

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(4), Constraint::Length(8)].as_ref())
        .split(f.size());

    let files_widget = make_files_widget(&app.files.filetable.items);
    f.render_stateful_widget(files_widget, chunks[0], &mut app.files.filetable.state);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    let fileinfo_widget = app.files.make_widget();
    f.render_widget(fileinfo_widget, chunks[0]);

    let messages_widget = make_messages_widget(&app.messages);
    f.render_widget(messages_widget, chunks[1]);

    if app.current_widget == AppWidgets::Help {
        render_help_widget(f);
    }

    if app.current_widget == AppWidgets::FileAction {
        render_prg_widget(f, &mut app.file_action, app.busy);
    }

    if app.current_widget == AppWidgets::CBMBrowser {
        render_cbm_selector_widget(f, &mut app.files.cbm_browser, app.busy);
    }
}

// Make messages widget
fn make_messages_widget(app_messages: &[String]) -> List {
    let messages: Vec<ListItem> = app_messages
        .iter()
        .enumerate()
        .rev()
        .map(|(i, m)| {
            let content = vec![Spans::from(Span::raw(format!("{}: {}", i + 1, m)))];
            ListItem::new(content)
        })
        .collect();
    List::new(messages).block(Block::default().borders(Borders::ALL).title(Span::styled(
        "Messages",
        Style::default().add_modifier(Modifier::BOLD),
    )))
}

fn render_help_widget<B: Backend>(f: &mut Frame<B>) {
    let area = centered_rect(50, 10, f.size());
    let block = Block::default()
        .title(Span::styled(
            "Help",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::SLOW_BLINK)
                .fg(Color::White),
        ))
        .style(Style::default().bg(Color::Blue))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let text = vec![
        Spans::from(Span::styled(
            "Matrix Mode Serial Communicator for MEGA65\n",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Spans::from(Span::styled(
            "Copyright (c) 2022 Wombat - Apache/MIT Licensed",
            Style::default().fg(Color::White),
        )),
        Spans::from(Span::styled("", Style::default().fg(Color::White))),
        Spans::from(Span::styled(
            "Select item (enter)",
            Style::default().fg(Color::White),
        )),
        Spans::from(Span::styled(
            "Toggle sorting by title or date (s)",
            Style::default().fg(Color::White),
        )),
        Spans::from(Span::styled(
            "Toggle help (h)",
            Style::default().fg(Color::White),
        )),
        Spans::from(Span::styled(
            "Reset MEGA65 (R)",
            Style::default().fg(Color::White),
        )),
        Spans::from(Span::styled("Quit (q)", Style::default().fg(Color::White))),
    ];
    let paragraph = Paragraph::new(text.clone())
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(Clear, area);
    //this clears out the background
    f.render_widget(paragraph, area);
}

/// helper function to create a centered rectangle of given width and height
fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let ymargin = match r.height > height {
        true => (r.height - height) / 2,
        false => 1,
    };
    let xmargin = match r.width > width {
        true => (r.width - width) / 2,
        false => 1,
    };
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(ymargin),
                Constraint::Length(height),
                Constraint::Length(ymargin),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(xmargin),
                Constraint::Length(width),
                Constraint::Length(xmargin),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

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
    let area = centered_rect(35, 10, f.size());
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
    let area = centered_rect(30, 7, f.size());
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

pub fn make_files_widget(filehost_items: &[filehost::Record]) -> Table {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["Title", "Type", "Author"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(0);
    let rows = filehost_items.iter().map(|item| {
        let col_data = item.columns();
        let height = col_data
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = col_data.iter().map(|c| Cell::from(*c));
        Row::new(cells).height(height as u16).bottom_margin(0)
    });
    let table = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                    "🌈 Filehost entries",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
        )
        .highlight_style(selected_style)
        .highlight_symbol("")
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ]);
    table
}
