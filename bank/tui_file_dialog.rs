/*[toml]
[dependencies]
crossterm = "0.28.1"
tui = "0.19.0"
tui-file-dialog = "0.1.0"
*/
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Result};
use tui::{
    backend::{Backend, CrosstermBackend},
    widgets::{Block, Borders},
    Frame, Terminal,
};

use tui_file_dialog::{bind_keys, FileDialog};

struct App {
    // 1. Add the `FileDialog` to the tui app.
    file_dialog: FileDialog,
}

impl App {
    pub fn new(file_dialog: FileDialog) -> Self {
        Self { file_dialog }
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, App::new(FileDialog::new(60, 40)?));

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
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

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let block = Block::default()
        .title(format!(
            "Selected file: {}",
            app.file_dialog
                .selected_file
                .as_ref()
                .map_or("None".to_string(), |f| f.to_string_lossy().to_string())
        ))
        .borders(Borders::ALL);
    f.render_widget(block, f.size());

    // 3. Call the draw function of the file dialog in order to render it.
    app.file_dialog.draw(f);
}
