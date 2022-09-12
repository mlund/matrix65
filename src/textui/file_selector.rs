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

use crate::textui::StatefulList;
use crate::filehost;
use crate::serial;
use crossterm::event::KeyCode;
use serialport::SerialPort;
use tui::{
    layout::{Alignment, Constraint},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState},
};

use anyhow::Result;

pub struct FilesApp {
    pub state: TableState,
    pub items: Vec<filehost::Record>,
    pub port: Box<dyn SerialPort>,
    toggle_sort: bool,
    /// Selected CBM disk
    cbm_disk: Option<Box<dyn cbm::disk::Disk>>,
    /// Browser for files CBM disk images (d81 etc)
    pub cbm_browser: StatefulList<String>,
}

impl FilesApp {
    pub fn new(port: &mut Box<dyn SerialPort>, filehost_items: &[filehost::Record]) -> FilesApp {
        FilesApp {
            state: TableState::default(),
            items: filehost_items.to_vec(),
            port: port.try_clone().unwrap(),
            toggle_sort: false,
            cbm_disk: None,
            cbm_browser: StatefulList::with_items(Vec::<String>::new()),
        }
    }

    pub fn keypress(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            KeyCode::Down => self.next(),
            KeyCode::Up => self.previous(),
            KeyCode::Char('r') => self.run(false)?,
            KeyCode::Char('R') => self.run(true)?,
            KeyCode::Char('s') => self.sort_filehost(),
            _ => {}
        }
        Ok(())
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

    /// Toggles filehost file sorting by date or title
    fn sort_filehost(&mut self) {
        if self.toggle_sort {
            self.items.sort_by_key(|i| i.published.clone());
            self.items.reverse();
        } else {
            self.items.sort_by_key(|i| i.title.clone());
        }
        self.toggle_sort = !self.toggle_sort;
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

    fn selected_url(&self) -> String {
        let sel = self.state.selected().unwrap_or(0);
        let item = &self.items[sel];
        format!("https://files.mega65.org/{}", &item.location)
    }

    /// Transfer and run selected file
    pub fn run(&mut self, reset_before_run: bool) -> Result<()> {
        let url = self.selected_url();
        if url.ends_with(".prg") {
            serial::handle_prg(&mut self.port, &url, reset_before_run, true)?;
        } else if url.ends_with(".d81") & self.cbm_disk.is_some() {
        }
        Ok(())
    }

    pub fn make_widget(&self) -> Paragraph {
        let sel = self.state.selected().unwrap_or(0);
        let item = &self.items[sel];
        let fileinfo_text = vec![
            Spans::from(format!("Title:      {}", item.title)),
            Spans::from(format!("Filename:   {}", item.filename)),
            Spans::from(format!("Category:   {} - {}", item.category, item.kind)),
            Spans::from(format!("Author:     {}", item.author)),
            Spans::from(format!("Published:  {}", item.published)),
            Spans::from(format!("Rating:     {}", item.rating)),
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
