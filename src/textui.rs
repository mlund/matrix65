use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, TableState,
    },
    Frame, Terminal,
};

use crate::filehost;
use crate::serial;
use serialport::SerialPort;

struct FilesApp {
    state: TableState,
    items: Vec<filehost::Record>,
    port: Box<dyn SerialPort>,
    toggle_sort: bool,
}

impl FilesApp {
    fn new(port: &mut Box<dyn SerialPort>, filehost_items: &[filehost::Record]) -> FilesApp {
        FilesApp {
            state: TableState::default(),
            items: filehost_items.to_vec(),
            port: port.try_clone().unwrap(),
            toggle_sort: false,
        }
    }

    fn keypress(&mut self, key: crossterm::event::KeyCode) -> io::Result<()> {
        match key {
            KeyCode::Down => self.next(),
            KeyCode::Up => self.previous(),
            KeyCode::Char('r') => self.run(false)?,
            KeyCode::Char('R') => self.run(true)?,
            KeyCode::Char('s') => self.sort_filehost(),
            _ => { }
        }
        Ok(())
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

    /// Toggles filehost file sorting by date or title
    pub fn sort_filehost(&mut self) {
        if self.toggle_sort {
            self.items.sort_by_key(|i| i.published.clone());
            self.items.reverse();
        } else {
            self.items.sort_by_key(|i| i.title.clone());
        }
        self.toggle_sort = !self.toggle_sort;
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

    fn selected_url(&self) -> String {
        let sel = self.state.selected().unwrap_or(0);
        let item = &self.items[sel];
        format!("https://files.mega65.org/{}", &item.location)
    }

    /// Transfer and run selected file
    pub fn run(&mut self, reset_before_run: bool) -> std::io::Result<()> {
        let url = self.selected_url();
        match url.ends_with(".prg") {
            true => serial::handle_prg(&mut self.port, &url, reset_before_run, true),
            false => Ok(()),
        }
    }
}

struct App {
    files: FilesApp,
    show_help: bool,
    messages: Vec<String>,
}

impl App {
    fn new(port: &mut Box<dyn SerialPort>, filehost_items: &[filehost::Record]) -> App {
        App {
            files: FilesApp::new(port, filehost_items),
            show_help: false,
            messages: vec!["Matrix65 welcomes you to the FileHost!".to_string()],
        }
    }

    pub fn keypress(&mut self, key: crossterm::event::KeyCode) -> io::Result<()> {
        self.files.keypress(key)
    }

    /// Set OK message if previous message is something else
    pub fn ok_message(&mut self) {
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
) -> Result<(), Box<dyn Error>> {
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
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    app.add_message("Downloading and running...");
                    terminal.draw(|f| ui(f, &mut app))?;
                }
                _ => {}
            }
            app.keypress(key.code)?;
            app.ok_message();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(4), Constraint::Length(8)].as_ref())
        .split(f.size());

    let files_widget = make_files_widget(&app.files.items);
    f.render_stateful_widget(files_widget, chunks[0], &mut app.files.state);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    let fileinfo_widget = make_fileinfo_widget(app);
    f.render_widget(fileinfo_widget, chunks[0]);

    let messages_widget = make_messages_widget(&app.messages);
    f.render_widget(messages_widget, chunks[1]);

    if app.show_help {
        render_help_widget(f);
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

fn render_help_widget<B: Backend>(f: &mut Frame<B>) {
    let area = centered_rect(35, 30, f.size());
    let block = Block::default()
        .title(Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ))
        .style(Style::default().bg(Color::Red))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let text = vec![
        Spans::from(Span::styled(
            "Run selection (r)",
            Style::default().fg(Color::White),
        )),
        Spans::from(Span::styled(
            "Reset & run selection (R)",
            Style::default().fg(Color::White),
        )),
        Spans::from(Span::styled(
            "Save selection to local disk (w)",
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
        Spans::from(Span::styled("Quit (q)", Style::default().fg(Color::White))),
    ];
    let paragraph = Paragraph::new(text.clone())
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(Clear, area);
    //this clears out the background
    f.render_widget(paragraph, area);
}

fn make_fileinfo_widget(app: &App) -> Paragraph {
    let sel = app.files.state.selected().unwrap_or(0);
    let item = &app.files.items[sel];
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

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
