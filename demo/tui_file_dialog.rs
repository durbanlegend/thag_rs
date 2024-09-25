/*[toml]
[dependencies]
crossterm = "0.28.1"
ratatui = "0.28.1"
thag_rs = { path = "/Users/donf/projects/thag_rs" }
*/
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Clear},
    Frame, Terminal,
};
use std::io::{self, Result};

use thag_rs::bind_keys;
use thag_rs::file_dialog::{DialogMode, FileDialog};

struct App<'a> {
    // 1. Add the `FileDialog` to the tui app.
    file_dialog: FileDialog<'a>,
}

impl<'a> App<'a> {
    pub fn new(file_dialog: FileDialog<'a>) -> Self {
        Self { file_dialog }
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(
        &mut terminal,
        App::new(FileDialog::new(60, 40, DialogMode::Open)?),
    );

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> io::Result<()> {
    terminal.clear()?;
    terminal.draw(|f| f.render_widget(Clear, f.area()))?;
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        // 2. Use the `bind_keys` macro to overwrite key bindings, when the file dialog is open.
        // The first argument of the macro is the expression that should be used to access the file
        // dialog.
        bind_keys!(
            app.file_dialog,
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('o') if key.modifiers == KeyModifiers::CONTROL => {
                        app.file_dialog.open()
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(());
                    }
                    _ => {}
                }
            }
        )
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let block = Block::default()
        .title(format!(
            "Selected file: {}",
            app.file_dialog
                .selected_file
                .as_ref()
                .map_or("None".to_string(), |f| f.to_string_lossy().to_string())
        ))
        .borders(Borders::ALL);
    // f.render_widget(Clear, f.area()); //this clears out the background
    f.render_widget(block, f.area());

    // 3. Call the draw function of the file dialog in order to render it.
    app.file_dialog.draw(f);
}
