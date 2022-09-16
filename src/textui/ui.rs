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
    widgets::{
        Block, BorderType, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table,
        TableState,
    },
    Frame,
};

use crate::filehost;
use crate::textui::{App, AppWidgets};

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(4), Constraint::Length(8)].as_ref())
        .split(f.size());

    let files_widget = make_files_widget(&app.filetable.items);
    f.render_stateful_widget(files_widget, chunks[0], &mut app.filetable.state);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    let fileinfo_widget = make_fileinfo_widget(&app.filetable);
    f.render_widget(fileinfo_widget, chunks[0]);

    let messages_widget = make_messages_widget(&app.messages);
    f.render_widget(messages_widget, chunks[1]);

    if app.active_widget == AppWidgets::Help {
        render_help_widget(f);
    }

    if app.active_widget == AppWidgets::FileAction {
        render_prg_widget(f, &mut app.file_action, app.busy);
    }

    if app.active_widget == AppWidgets::CBMBrowser {
        render_cbm_selector_widget(f, &mut app.cbm_browser, app.busy);
    }
}

// Widget with logging information
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

/// Popup widget with helful information
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

/// Widget for selecting files inside CBM disk images
fn render_cbm_selector_widget<B: Backend>(
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

/// Popup widget with selectable actions for PRG/D81 files
fn render_prg_widget<B: Backend>(
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

/// Widget showing details about a selected filehost item
fn make_fileinfo_widget(filetable: &StatefulTable<filehost::Record>) -> Paragraph {
    let sel = filetable.state.selected().unwrap_or(0);
    let item = &filetable.items[sel];
    let fileinfo_text = vec![
        Spans::from(format!("Title:     {}", item.title)),
        Spans::from(format!("Filename:  {}", item.filename)),
        Spans::from(format!("Category:  {} - {}", item.category, item.kind)),
        Spans::from(format!("Author:    {}", item.author)),
        Spans::from(format!("Published: {}", item.published)),
        Spans::from(format!("Rating:    {}", item.rating)),
    ];
    let block = Block::default()
        .title(Span::styled(
            "File Info",
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    Paragraph::new(fileinfo_text)
        .block(block)
        .alignment(Alignment::Left)
}

/// Table with all FileHost records
fn make_files_widget(filehost_items: &[filehost::Record]) -> Table {
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

    pub fn next(&mut self) {
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

    pub fn previous(&mut self) {
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

    pub fn is_selected(&self) -> bool {
        self.state.selected() != None
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

pub struct StatefulTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
}

impl<T> StatefulTable<T> {
    pub fn with_items(items: Vec<T>) -> StatefulTable<T> {
        StatefulTable {
            state: TableState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
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

    pub fn previous(&mut self) {
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

    #[allow(dead_code)]
    pub fn is_selected(&self) -> bool {
        self.state.selected() != None
    }

    #[allow(dead_code)]
    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
