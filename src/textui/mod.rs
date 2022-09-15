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
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use anyhow::Result;
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

use crate::filehost;
use serialport::SerialPort;
mod cbm_browser;
mod file_action;
mod file_selector;
use file_selector::FilesApp;

/// Specified the currently active widget of the TUI
#[derive(PartialEq)]
enum AppWidgets {
    FileSelector,
    FileAction,
    CBMBrowser,
    Help,
}

struct App {
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

pub fn start_tui(
    port: &mut Box<dyn SerialPort>,
    filehost_items: &[filehost::Record],
) -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(port, filehost_items);
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {}", err)
    }
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('R') => {
                    crate::serial::reset(&mut app.files.port)?;
                    app.add_message("Reset MEGA65");
                }
                KeyCode::Enter => {
                    if app.files.cbm_browser.is_selected() {
                        app.busy = true;
                        terminal.draw(|f| ui(f, &mut app))?;
                    } else {
                        app.busy = false;
                    }
                }
                _ => {}
            }
            match app.keypress(key.code) {
                Ok(()) => {}
                Err(error) => {
                    app.add_message(error.to_string().as_str());
                    app.files.cbm_browser.unselect();
                    app.current_widget = AppWidgets::FileSelector;
                }
            }
            //app.ok_message();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(4), Constraint::Length(8)].as_ref())
        .split(f.size());

    let files_widget = file_selector::make_files_widget(&app.files.items);
    f.render_stateful_widget(files_widget, chunks[0], &mut app.files.state);

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
        file_action::render_prg_widget(f, &mut app.file_action, app.busy);
    }

    if app.current_widget == AppWidgets::CBMBrowser {
        cbm_browser::render_cbm_selector_widget(f, &mut app.files.cbm_browser, app.busy);
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
        false => 1
    };
    let xmargin = match r.width > width {
        true => (r.width - width) / 2,
        false => 1
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
