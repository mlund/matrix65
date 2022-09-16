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

use crossterm::{
    event::KeyCode,
};

use anyhow::Result;
use tui::{
    layout::Alignment,
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, ListState, Paragraph, TableState},
};

use crate::filehost;
use crate::serial;
use serialport::SerialPort;
pub mod terminal;
mod ui;

pub struct FilesApp {
    pub filetable: StatefulTable<filehost::Record>,
    pub port: Box<dyn SerialPort>,
    toggle_sort: bool,
    /// Selected CBM disk
    pub cbm_disk: Option<Box<dyn cbm::disk::Disk>>,
    /// Browser for files CBM disk images (d81 etc)
    pub cbm_browser: StatefulList<String>,
}

impl FilesApp {
    pub fn new(port: &mut Box<dyn SerialPort>, filehost_items: &[filehost::Record]) -> FilesApp {
        FilesApp {
            filetable: StatefulTable::with_items(filehost_items.to_vec()),
            port: port.try_clone().unwrap(),
            toggle_sort: false,
            cbm_disk: None,
            cbm_browser: StatefulList::with_items(Vec::<String>::new()),
        }
    }

    pub fn make_widget(&self) -> Paragraph {
        let sel = self.filetable.state.selected().unwrap_or(0);
        let item = &self.filetable.items[sel];
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

    pub fn keypress(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            KeyCode::Down => self.filetable.next(),
            KeyCode::Up => self.filetable.previous(),
            KeyCode::Char('s') => self.sort_filehost(),
            _ => {}
        }
        Ok(())
    }

    /// Toggles filehost file sorting by date or title
    fn sort_filehost(&mut self) {
        if self.toggle_sort {
            self.filetable.items.sort_by_key(|i| i.published.clone());
            self.filetable.items.reverse();
        } else {
            self.filetable.items.sort_by_key(|i| i.title.clone());
        }
        self.toggle_sort = !self.toggle_sort;
    }

    pub fn selected_url(&self) -> String {
        let sel = self.filetable.state.selected().unwrap_or(0);
        let item = &self.filetable.items[sel];
        format!("https://files.mega65.org/{}", &item.location)
    }

    /// Transfer and run selected file
    pub fn run(&mut self, reset_before_run: bool) -> Result<()> {
        let url = self.selected_url();
        if url.ends_with(".prg") {
            serial::handle_prg(&mut self.port, &url, reset_before_run, true)?;
        } else if url.ends_with(".d81") & self.cbm_disk.is_some() & self.cbm_browser.is_selected() {
            let selected_file = self.cbm_browser.state.selected().unwrap();
            let (load_address, bytes) =
                crate::io::cbm_load_file(self.cbm_disk.as_ref().unwrap().as_ref(), selected_file)?;
            serial::handle_prg_from_bytes(
                &mut self.port,
                &bytes,
                load_address,
                reset_before_run,
                true,
            )?;
            self.cbm_browser.unselect();
            self.cbm_disk = None;
        } else {
            return Err(anyhow::Error::msg("Cannot run selection"));
        }
        Ok(())
    }
}

/// Specified the currently active widget of the TUI
#[derive(PartialEq, Eq)]
pub enum AppWidgets {
    FileSelector,
    FileAction,
    CBMBrowser,
    Help,
}

pub struct App {
    /// FileHost file browser
    files: FilesApp,
    /// Status messages presented in the UI
    messages: Vec<String>,
    /// Holds the active widget
    current_widget: AppWidgets,
    /// Browser for actions on a single file
    file_action: StatefulList<String>,
    /// Set to true when UI is unresponsive
    busy: bool,
}

impl App {
    fn new(port: &mut Box<dyn SerialPort>, filehost_items: &[filehost::Record]) -> App {
        App {
            files: FilesApp::new(port, filehost_items),
            messages: vec![
                "Matrix65 welcomes you to the FileHost!".to_string(),
                "Press 'h' for help".to_string(),
            ],
            current_widget: AppWidgets::FileSelector,
            file_action: StatefulList::with_items(vec![
                "Run".to_string(),
                "Reset and Run".to_string(),
                "Open CBM disk...".to_string(),
                "Cancel".to_string(),
            ]),
            busy: false,
        }
    }

    pub fn set_current_widget(&mut self, widget: AppWidgets) {
        self.current_widget = widget;
    }

    /// Populate and activate CBM disk browser
    fn activate_cbm_browser(&mut self) -> Result<()> {
        self.busy = false;
        self.set_current_widget(AppWidgets::CBMBrowser);
        let url = self.files.selected_url();
        self.files.cbm_disk = Some(crate::io::cbm_open(&url)?);
        if self.files.cbm_disk.is_some() {
            let dir = self.files.cbm_disk.as_ref().unwrap().directory()?;
            let files: Vec<String> = dir
                .iter()
                .map(|i| format!("{}.{}", i.filename.to_string(), i.file_attributes.file_type))
                .collect();
            self.files.cbm_browser.items = files;
        }
        Ok(())
    }

    pub fn keypress(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('h') => {
                if self.current_widget != AppWidgets::Help {
                    self.set_current_widget(AppWidgets::Help);
                } else {
                    self.set_current_widget(AppWidgets::FileSelector);
                }
            }

            // Escape jumps back to filehost selector
            KeyCode::Esc => {
                self.set_current_widget(AppWidgets::FileSelector);
                self.file_action.unselect();
            }

            KeyCode::Enter => {
                match self.current_widget {
                    // Enter in file selector triggers an action on the selected file
                    AppWidgets::FileSelector => {
                        self.current_widget = AppWidgets::FileAction;
                        if !self.file_action.is_selected() {
                            self.file_action.state.select(Some(0));
                        }
                    }
                    // Enter in action widget trigges an action on the prg
                    AppWidgets::FileAction => {
                        self.set_current_widget(AppWidgets::FileSelector);
                        match self.file_action.state.selected() {
                            Some(0) => self.files.run(false)?, // run
                            Some(1) => self.files.run(true)?,  // reset, then run
                            Some(2) => self.activate_cbm_browser()?,
                            _ => {}
                        };
                        self.file_action.unselect();
                    }
                    AppWidgets::CBMBrowser => {
                        match self.files.cbm_browser.state.selected() {
                            _ => {
                                self.files.run(false)?;
                                self.busy = false;
                                self.current_widget = AppWidgets::FileSelector;
                            }
                        };
                        self.file_action.unselect();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        match self.current_widget {
            AppWidgets::CBMBrowser => self.files.cbm_browser.keypress(key),
            AppWidgets::FileAction => self.file_action.keypress(key),
            AppWidgets::FileSelector => self.files.keypress(key),
            _ => Ok(()),
        }
    }

    /// Set OK message if previous message is something else
    pub fn _ok_message(&mut self) {
        let ok_text = "Ready".to_string();
        if *self.messages.last().unwrap() != ok_text {
            self.messages.push(ok_text);
        }
    }

    pub fn add_message(&mut self, message: &str) {
        self.messages.push(message.to_string());
    }

    #[allow(dead_code)]
    pub fn clear_status_line(&mut self) {
        //self.messages.clear();
    }
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

    pub fn is_selected(&self) -> bool {
        self.state.selected() != None
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn keypress(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            KeyCode::Down => self.next(),
            KeyCode::Up => self.previous(),
            _ => {}
        }
        Ok(())
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

    #[allow(dead_code)]
    pub fn is_selected(&self) -> bool {
        self.state.selected() != None
    }

    #[allow(dead_code)]
    pub fn unselect(&mut self) {
        self.state.select(None);
    }

}
