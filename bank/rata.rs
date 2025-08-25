/*[toml]
[target.'cfg(not(target_os = "windows"))'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect", "ratatui_support"] }

[target.'cfg(target_os = "windows")'.dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["config", "ratatui_support"] }
*/

use color_eyre::Result;
use itertools::Itertools;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode},
    layout::Rect,
    style::Style,
    text::{Line, Span, Text},
    widgets::Widget,
    DefaultTerminal,
};
use thag_styling::{Role, ThemedStyle};

fn main() -> Result<()> {
    color_eyre::install()?;
    let hd1_style = Style::themed(Role::HD1);
    eprintln!("DEBUG: HD1 style in rata.rs = {:?}", hd1_style); // Add this line

    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}

struct App {
    hyperlink: Hyperlink<'static>,
}

impl App {
    fn new() -> Self {
        let text = Line::from(vec![
            "Example ".into(),
            Span::from(" Role::HD1").style(Style::themed(Role::HD1)),
            Span::from(" Role::HD2").style(Style::themed(Role::HD2)),
            Span::from(" Role::HD3").style(Style::themed(Role::HD3)),
            Span::from(" Role::ERR").style(Style::themed(Role::ERR)),
            Span::from(" Role::WARN").style(Style::themed(Role::WARN)),
            Span::from(" Role::EMPH").style(Style::themed(Role::EMPH)),
            Span::from(" Role::SUCC").style(Style::themed(Role::SUCC)),
            Span::from(" Role::NORM").style(Style::themed(Role::NORM)),
            Span::from(" Role::INFO").style(Style::themed(Role::INFO)),
            Span::from(" Role::CODE").style(Style::themed(Role::CODE)),
            Span::from(" Role::Subtle").style(Style::themed(Role::Subtle)),
            Span::from(" Role::DBUG").style(Style::themed(Role::DBUG)),
            Span::from(" Role::HINT").style(Style::themed(Role::HINT)),
            Span::from(" Role::TRCE").style(Style::themed(Role::TRCE)),
            // Span::from("hyperlink"),
        ]);
        let hyperlink = Hyperlink::new(text, "https://example.com");
        Self { hyperlink }
    }

    fn run(self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| frame.render_widget(&self.hyperlink, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break;
                }
            }
        }
        Ok(())
    }
}

/// A hyperlink widget that renders a hyperlink in the terminal using [OSC 8].
///
/// [OSC 8]: https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda
struct Hyperlink<'content> {
    text: Text<'content>,
    url: String,
}

impl<'content> Hyperlink<'content> {
    fn new(text: impl Into<Text<'content>>, url: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            url: url.into(),
        }
    }
}

impl Widget for &Hyperlink<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        (&self.text).render(area, buffer);

        // this is a hacky workaround for https://github.com/ratatui/ratatui/issues/902, a bug
        // in the terminal code that incorrectly calculates the width of ANSI escape sequences. It
        // works by rendering the hyperlink as a series of 2-character chunks, which is the
        // calculated width of the hyperlink text.
        for (i, two_chars) in self
            .text
            .to_string()
            .chars()
            .chunks(2)
            .into_iter()
            .enumerate()
        {
            let text = two_chars.collect::<String>();
            let hyperlink = format!("\x1B]8;;{}\x07{}\x1B]8;;\x07", self.url, text);
            buffer[(area.x + i as u16 * 2, area.y)].set_symbol(hyperlink.as_str());
        }
    }
}
