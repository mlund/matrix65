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
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};

use crate::filehost;
use crate::serial;
use serialport::SerialPort;

struct App {
    state: TableState,
    filehost_items: Vec<filehost::Record>,
    show_help: bool,
    port: Box<dyn SerialPort>,
    status_line: String,
}

impl App {
    fn new(port: &mut Box<dyn SerialPort>, filehost_items: &[filehost::Record]) -> App {
        App {
            state: TableState::default(),
            filehost_items: filehost_items.to_vec(),
            show_help: false,
            port: port.try_clone().unwrap(),
            status_line: String::new(),
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filehost_items.len() - 1 {
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
                    self.filehost_items.len() - 1
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
        let item = &self.filehost_items[sel];
        format!("https://files.mega65.org/{}", &item.location)
    }

    /// Transfer and run selected file
    pub fn run(&mut self, reset_before_run: bool) -> std::io::Result<()> {
        let url = self.selected_url();
        serial::handle_prg(&mut self.port, &url, reset_before_run, true)
    }

    pub fn set_status_line(&mut self, text: &str) {
        self.status_line = String::from(text);
    }

    #[allow(dead_code)]
    pub fn clear_status_line(&mut self) {
        self.status_line.clear();
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
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    app.set_status_line("Busy...");
                    terminal.draw(|f| ui(f, &mut app))?;
                }
                _ => {}
            }
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                KeyCode::Char('h') | KeyCode::Enter => app.show_help = !app.show_help,
                KeyCode::Char('r') => app.run(false)?,
                KeyCode::Char('R') => app.run(true)?,
                _ => {}
            }
            app.set_status_line("Ready");
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(4),
                Constraint::Length(8),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["Title", "Type", "Rating"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(0);
    let rows = app.filehost_items.iter().map(|item| {
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
            Constraint::Length(30),
            Constraint::Min(10),
        ]);
    f.render_stateful_widget(table, chunks[0], &mut app.state);

    let fileinfo_widget = make_fileinfo_widget(app);
    f.render_widget(fileinfo_widget, chunks[1]);

    // Show help pop-up
    if app.show_help {
        app.status_line = String::from("hejsa");
        render_help_popup(f);
    }

    // Status widget
    let par = Paragraph::new(vec![Spans::from((&app.status_line).to_string())])
        .block(
            Block::default()
                .title(Span::styled(
                    "Status",
                    Style::default().add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Left);
    f.render_widget(par, chunks[2]);
}

fn render_help_popup<B: Backend>(f: &mut Frame<B>) {
    let area = centered_rect(30, 30, f.size());
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
        Spans::from(Span::styled("Run (r)", Style::default().fg(Color::White))),
        Spans::from(Span::styled(
            "Reset & Run (R)",
            Style::default().fg(Color::White),
        )),
        Spans::from(Span::styled(
            "Download (d)",
            Style::default().fg(Color::White),
        )),
        Spans::from(Span::styled("Help (h)", Style::default().fg(Color::White))),
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
    let sel = app.state.selected().unwrap_or(0);
    let item = &app.filehost_items[sel];
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
