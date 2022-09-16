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

pub mod terminal;
mod ui;

use crate::{filehost, serial};
use anyhow::Result;
use crossterm::event::KeyCode;
use serialport::SerialPort;
use ui::{StatefulList, StatefulTable};

/// Specified the currently active widget of the TUI
#[derive(PartialEq, Eq)]
pub enum AppWidgets {
    FileSelector,
    FileAction,
    CBMBrowser,
    Help,
}

pub struct App {
    /// Holds the active widget
    active_widget: AppWidgets,
    /// Set to true when UI is unresponsive
    busy: bool,
    /// Browser for files CBM disk images (d81 etc)
    cbm_browser: StatefulList<String>,
    /// Selected CBM disk
    cbm_disk: Option<Box<dyn cbm::disk::Disk>>,
    /// Browser for actions on a single file
    file_action: StatefulList<String>,
    /// FileHost file browser
    filetable: StatefulTable<filehost::Record>,
    /// Status messages presented in the UI
    messages: Vec<String>,
    /// Serial port to communicate on
    port: Box<dyn SerialPort>,
    /// Determines how to sort the filehost table
    toggle_sort: bool,
}

impl App {
    fn new(port: &mut Box<dyn SerialPort>, filehost_items: &[filehost::Record]) -> App {
        App {
            messages: vec![
                "Matrix65 welcomes you to the FileHost!".to_string(),
                "Press 'h' for help".to_string(),
            ],
            active_widget: AppWidgets::FileSelector,
            file_action: StatefulList::with_items(vec![
                "Run".to_string(),
                "Reset and Run".to_string(),
                "Open CBM disk...".to_string(),
                "Cancel".to_string(),
            ]),
            busy: false,
            filetable: StatefulTable::with_items(filehost_items.to_vec()),
            port: port.try_clone().unwrap(),
            toggle_sort: false,
            cbm_disk: None,
            cbm_browser: StatefulList::with_items(Vec::<String>::new()),
        }
    }

    pub fn set_current_widget(&mut self, widget: AppWidgets) {
        self.active_widget = widget;
    }

    /// Populate and activate CBM disk browser
    fn activate_cbm_browser(&mut self) -> Result<()> {
        self.busy = false;
        self.set_current_widget(AppWidgets::CBMBrowser);
        let url = self.selected_url();
        self.cbm_disk = Some(crate::io::cbm_open(&url)?);
        if self.cbm_disk.is_some() {
            let dir = self.cbm_disk.as_ref().unwrap().directory()?;
            let files: Vec<String> = dir
                .iter()
                .map(|i| format!("{}.{}", i.filename.to_string(), i.file_attributes.file_type))
                .collect();
            self.cbm_browser.items = files;
        }
        Ok(())
    }

    // @todo this should be moved to ui.rs so that mod.rs is independent of crossterm
    pub fn keypress(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                match self.active_widget {
                    AppWidgets::FileSelector => self.select_filehost_item()?,
                    AppWidgets::FileAction => self.select_file_action()?,
                    AppWidgets::CBMBrowser => self.select_cbm_item()?,
                    _ => {}
                }
            }
            _ => {}
        }
        match self.active_widget {
            AppWidgets::CBMBrowser => match key {
                KeyCode::Down => {
                    self.cbm_browser.next();
                    Ok(())
                }
                KeyCode::Up => {
                    self.cbm_browser.previous();
                    Ok(())
                }
                _ => Ok(()),
            },
            AppWidgets::FileAction => match key {
                KeyCode::Down => {
                    self.file_action.next();
                    Ok(())
                }
                KeyCode::Up => {
                    self.file_action.previous();
                    Ok(())
                }
                _ => Ok(()),
            },
            AppWidgets::FileSelector => match key {
                KeyCode::Down => {
                    self.filetable.next();
                    Ok(())
                }
                KeyCode::Up => {
                    self.filetable.previous();
                    Ok(())
                }
                KeyCode::Char('s') => {
                    self.sort_filehost();
                    Ok(())
                }
                _ => Ok(()),
            },
            _ => Ok(()),
        }
    }

    fn escape_to_filehost_browser(&mut self) {
        self.set_current_widget(AppWidgets::FileSelector);
        self.file_action.unselect();
    }

    /// Select currently highlighted file in FileHost browser
    fn select_filehost_item(&mut self) -> Result<(), anyhow::Error> {
        // when selecting file, go to file action widget 
        self.active_widget = AppWidgets::FileAction;
        if !self.file_action.is_selected() {
            self.file_action.state.select(Some(0));
        };
        Ok(())
    }

    /// Select currently highlighted action in file action widget
    fn select_file_action(&mut self) -> Result<(), anyhow::Error> {
        // when done, return to filehost browser
        self.set_current_widget(AppWidgets::FileSelector);
        match self.file_action.state.selected() {
            Some(0) => self.run(false)?, // run
            Some(1) => self.run(true)?,  // reset, then run
            Some(2) => self.activate_cbm_browser()?,
            _ => {}
        };
        self.file_action.unselect();
        Ok(())
    }

    /// Select currently highlighted item in CBM browser
    fn select_cbm_item(&mut self) -> Result<(), anyhow::Error> {
        match self.cbm_browser.state.selected() {
            _ => {
                self.run(false)?;
                self.busy = false;
                self.active_widget = AppWidgets::FileSelector;
            }
        };
        self.cbm_browser.unselect();
        self.file_action.unselect();
        Ok(())
    }

    /// Toggles the help pop-up
    fn toggle_help(&mut self) {
        if self.active_widget != AppWidgets::Help {
            self.set_current_widget(AppWidgets::Help);
        } else {
            self.set_current_widget(AppWidgets::FileSelector);
        }
    }

    /// Set OK message if previous message is something else
    fn _ok_message(&mut self) {
        let ok_text = "Ready".to_string();
        if *self.messages.last().unwrap() != ok_text {
            self.messages.push(ok_text);
        }
    }

    fn add_message(&mut self, message: &str) {
        self.messages.push(message.to_string());
    }

    #[allow(dead_code)]
    pub fn clear_status_line(&mut self) {
        //self.messages.clear();
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
