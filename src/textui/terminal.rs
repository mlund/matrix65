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

use crate::textui::{ui, App, AppWidgets};
use anyhow::Result;
use matrix65::{filehost, M65Communicator};
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

/// This is the first entry for the TUI
pub fn start_tui(
    comm: &mut Box<dyn M65Communicator>,
    filehost_items: &[filehost::Record],
) -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(comm, filehost_items);
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
        terminal.draw(|f| ui::ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('h') => app.toggle_help(),
                KeyCode::Char('s') => app.sort_filehost(),
                KeyCode::Esc => app.return_to_filehost(),
                KeyCode::Up => app.previous_item(),
                KeyCode::Down => app.next_item(),
                KeyCode::Enter => {
                    if app.cbm_browser.is_selected() | app.file_action.is_selected() {
                        app.busy = true;
                        terminal.draw(|f| ui::ui(f, &mut app))?;
                    } else {
                        app.busy = false;
                    }
                }
                _ => {}
            }
            // These operations *may* fail (invalid port, corrupted file etc.)
            let result = match key.code {
                KeyCode::Char('R') => app.reset(),
                KeyCode::Enter => match app.active_widget {
                    AppWidgets::FileSelector => app.select_filehost_item(),
                    AppWidgets::FileAction => app.select_file_action(),
                    AppWidgets::CBMBrowser => app.select_cbm_item(),
                    _ => Ok(()),
                },
                _ => Ok(()),
            };
            // Gracefully recover and show error in the msg widget
            match result {
                Ok(()) => {}
                Err(error) => {
                    app.add_message(error.to_string().as_str());
                    app.active_widget = AppWidgets::FileSelector;
                    app.unselect_all();
                }
            }
        }
    }
}
