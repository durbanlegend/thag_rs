/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["ratatui_support"] }
ratatui = "0.29"
crossterm = "0.28"
*/

/// Simple Ratatui + thag_styling Integration Demo
///
/// This demo shows how to create a basic themed TUI application using ratatui
/// and thag_styling's semantic role system.
///
/// E.g.:
/// ```
/// thag demo/ratatui_integration_demo.rs
/// ```
//# Purpose: Basic demonstration of ratatui integration with thag_styling
//# Categories: demo, gui, theming
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame, Terminal,
};

use thag_styling::{
    integrations::{RatatuiStyleExt, ThemedStyle},
    Role,
};

use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let res = run_demo(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    } else {
        println!("\n🎨 Demo completed! This shows how thag_styling integrates with ratatui.");
        println!("Key features demonstrated:");
        println!("  • Automatic theme detection based on terminal capabilities");
        println!("  • Semantic role-based styling (Error, Success, Warning, etc.)");
        println!("  • ThemedStyle trait for consistent theming");
        println!("  • Extension methods for flexible style composition");
    }

    Ok(())
}

fn run_demo<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut counter = 0;

    loop {
        terminal.draw(|f| draw_demo(f, counter))?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char(' ') => counter = (counter + 1) % 100,
                _ => {}
            }
        }
    }
    Ok(())
}

fn draw_demo(frame: &mut Frame, counter: usize) {
    let area = frame.area();

    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(5), // Progress
            Constraint::Length(8), // Status messages
            Constraint::Min(0),    // Instructions
        ])
        .split(area);

    // Title
    let title = Paragraph::new("🎨 Thag Styling + Ratatui Demo")
        .style(Style::themed(Role::Heading1).bold())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::themed(Role::Subtle)),
        );
    frame.render_widget(title, chunks[0]);

    // Progress bar demonstrating themed gauge
    let progress = (counter as f64 / 100.0 * 100.0) as u16;
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📊 Themed Progress")
                .title_style(Style::themed(Role::Heading2))
                .border_style(Style::themed(Role::Subtle)),
        )
        .gauge_style(Style::themed(Role::Success))
        .percent(progress)
        .label(format!("{}%", progress));
    frame.render_widget(gauge, chunks[1]);

    // Status messages demonstrating different roles
    let status_items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("✅ ", Style::themed(Role::Success)),
            Span::styled(
                "System initialization complete",
                Style::themed(Role::Success),
            ),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("⚠️  ", Style::themed(Role::Warning)),
            Span::styled("Memory usage at 75%", Style::themed(Role::Warning)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("❌ ", Style::themed(Role::Error)),
            Span::styled("Network connection failed", Style::themed(Role::Error)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("ℹ️  ", Style::themed(Role::Info)),
            Span::styled("Processing batch job #1234", Style::themed(Role::Info)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("🔧 ", Style::themed(Role::Subtle)),
            Span::styled("Debug: Internal state OK", Style::themed(Role::Subtle)),
        ])),
    ];

    let status_list = List::new(status_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("📋 System Status")
            .title_style(Style::themed(Role::Heading2))
            .border_style(Style::themed(Role::Subtle)),
    );
    frame.render_widget(status_list, chunks[2]);

    // Instructions and code examples
    let instructions = Text::from(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Code Example - ThemedStyle trait:",
            Style::themed(Role::Heading3),
        )]),
        Line::from(vec![Span::styled(
            "let style = Style::themed(Role::Error);",
            Style::themed(Role::Code),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Code Example - Extension methods:",
            Style::themed(Role::Heading3),
        )]),
        Line::from(vec![Span::styled(
            "let themed = base.with_role(Role::Success);",
            Style::themed(Role::Code),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Controls:",
            Style::themed(Role::Emphasis),
        )]),
        Line::from(vec![
            Span::styled("  Space", Style::themed(Role::Code)),
            Span::styled(" - Update progress  ", Style::themed(Role::Normal)),
            Span::styled("q/Esc", Style::themed(Role::Code)),
            Span::styled(" - Quit", Style::themed(Role::Normal)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "This demo automatically detects your terminal theme!",
            Style::themed(Role::Info),
        )]),
    ]);

    // Demonstrate extension method usage
    let instructions_widget = Paragraph::new(instructions)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("📖 Integration Examples")
                .title_style(Style::themed(Role::Heading2))
                // Using extension method instead of direct theming
                .border_style(Style::default().with_role(Role::Subtle)),
        );
    frame.render_widget(instructions_widget, chunks[3]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_themed_styles() {
        // Test that themed styles are different from defaults
        let error_style = Style::themed(Role::Error);
        let success_color = Color::themed(Role::Success);

        assert_ne!(error_style, Style::default());
        assert_ne!(success_color, Color::Reset);
    }

    #[test]
    fn test_extension_methods() {
        // Test the extension trait
        let base = Style::default().bold();
        let themed = base.with_role(Role::Info);

        // Should preserve original styling
        assert!(themed
            .add_modifier
            .intersects(ratatui::style::Modifier::BOLD));
    }
}
