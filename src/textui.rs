use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};

use crate::filehost;

struct App {
    state: TableState,
    filehost_items: Vec<filehost::Record>,
}

impl App {
    fn new(filehost_itemsss: &[filehost::Record]) -> App {
        App {
            state: TableState::default(),
            filehost_items: filehost_itemsss.to_vec(),
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
}

pub fn start_tui(filehost_items: &[filehost::Record]) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(filehost_items);
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
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Max(8)].as_ref())
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
                .title(Span::styled("🌈 Filehost entries", Style::default().add_modifier(Modifier::BOLD),))
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
        .title(Span::styled("File Info", Style::default().add_modifier(Modifier::BOLD),))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    Paragraph::new(fileinfo_text).block(block).alignment(Alignment::Left)
}
