/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["ratatui_support"] }
ratatui = "0.29"
crossterm = "0.28"
*/
/// Comprehensive Ratatui Theming Showcase
///
/// This example demonstrates how to build a themed TUI application using ratatui
/// and thag_styling. It showcases various UI components styled with semantic roles
/// and demonstrates both the ThemedStyle trait and extension methods.
///
/// ```Rust
/// E.g. `thag demo/ratatui_theming_showcase`
/// ```
//# Purpose: Comprehensive showcase of ratatui integration with thag_styling themes
//# Categories: demo, theming, tui
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Tabs, Wrap,
    },
    Frame, Terminal,
};

use thag_styling::{
    integrations::{RatatuiStyleExt, ThemedStyle},
    Role,
};

use std::io;

#[derive(Default)]
struct App {
    tab_index: usize,
    scroll_position: usize,
    show_help: bool,
    progress: f64,
    log_entries: Vec<LogEntry>,
}

struct LogEntry {
    level: Role,
    message: String,
}

impl App {
    fn new() -> Self {
        Self {
            tab_index: 0,
            scroll_position: 0,
            show_help: false,
            progress: 0.0,
            log_entries: vec![
                LogEntry {
                    level: Role::Info,
                    message: "Application started successfully".to_string(),
                },
                LogEntry {
                    level: Role::Success,
                    message: "Connected to remote server".to_string(),
                },
                LogEntry {
                    level: Role::Warning,
                    message: "High memory usage detected (85%)".to_string(),
                },
                LogEntry {
                    level: Role::Error,
                    message: "Failed to save configuration file".to_string(),
                },
                LogEntry {
                    level: Role::Info,
                    message: "Processing batch job #1234".to_string(),
                },
                LogEntry {
                    level: Role::Success,
                    message: "Data synchronization completed".to_string(),
                },
                LogEntry {
                    level: Role::Error,
                    message: "Network timeout after 30 seconds".to_string(),
                },
            ],
        }
    }

    fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % 4;
    }

    fn previous_tab(&mut self) {
        self.tab_index = if self.tab_index > 0 {
            self.tab_index - 1
        } else {
            3
        };
    }

    fn scroll_up(&mut self) {
        self.scroll_position = self.scroll_position.saturating_sub(1);
    }

    fn scroll_down(&mut self) {
        self.scroll_position = self.scroll_position.saturating_add(1);
    }

    fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    fn update_progress(&mut self) {
        self.progress = (self.progress + 0.02) % 1.0;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('h') | KeyCode::F(1) => app.toggle_help(),
                KeyCode::Right | KeyCode::Tab => app.next_tab(),
                KeyCode::Left | KeyCode::BackTab => app.previous_tab(),
                KeyCode::Up => app.scroll_up(),
                KeyCode::Down => app.scroll_down(),
                KeyCode::Char('1') => app.tab_index = 0,
                KeyCode::Char('2') => app.tab_index = 1,
                KeyCode::Char('3') => app.tab_index = 2,
                KeyCode::Char('4') => app.tab_index = 3,
                _ => {}
            }
        }
        app.update_progress();
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Render header
    render_header(f, chunks[0]);

    // Render main content with tabs
    render_content(f, chunks[1], app);

    // Render footer
    render_footer(f, chunks[2]);

    // Render help overlay if needed
    if app.show_help {
        render_help_popup(f, size);
    }
}

fn render_header(f: &mut Frame, area: Rect) {
    let title = Paragraph::new("🎨 Thag Styling + Ratatui Integration Showcase")
        .style(Style::themed(Role::Heading1).bold())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::themed(Role::Subtle))
                .border_type(BorderType::Double),
        );
    f.render_widget(title, area);
}

fn render_content(f: &mut Frame, area: Rect, app: &App) {
    // Create tab titles
    let tab_titles: Vec<Line> = vec![
        Line::from("📊 Dashboard".fg(Color::themed(Role::Heading2))),
        Line::from("📝 Logs".fg(Color::themed(Role::Heading2))),
        Line::from("🎛️ Settings".fg(Color::themed(Role::Heading2))),
        Line::from("📋 About".fg(Color::themed(Role::Heading2))),
    ];

    // Create tabs widget
    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::themed(Role::Subtle)),
        )
        .select(app.tab_index)
        .style(Style::themed(Role::Normal))
        .highlight_style(Style::themed(Role::Emphasis).bold())
        .divider("│".fg(Color::themed(Role::Subtle)));

    // Split area for tabs and content
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    f.render_widget(tabs, content_chunks[0]);

    // Render tab content
    match app.tab_index {
        0 => render_dashboard_tab(f, content_chunks[1], app),
        1 => render_logs_tab(f, content_chunks[1], app),
        2 => render_settings_tab(f, content_chunks[1]),
        3 => render_about_tab(f, content_chunks[1]),
        _ => {}
    }
}

fn render_dashboard_tab(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Progress bar
            Constraint::Length(8), // Metrics
            Constraint::Min(0),    // Status
        ])
        .split(area);

    // Progress bar
    let progress_bar = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::themed(Role::Subtle))
                .title("📈 Progress")
                .title_style(Style::themed(Role::Heading3)),
        )
        .gauge_style(Style::themed(Role::Success))
        .percent((app.progress * 100.0) as u16)
        .label(format!("{:.1}%", app.progress * 100.0));
    f.render_widget(progress_bar, chunks[0]);

    // Metrics grid
    let metrics_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(chunks[1]);

    // CPU Usage
    let cpu_widget = Paragraph::new(Text::from(vec![
        Line::from(""),
        Line::from(Span::styled("85%", Style::themed(Role::Warning).bold())),
        Line::from(""),
        Line::from(Span::styled("High", Style::themed(Role::Warning))),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().with_role(Role::Warning))
            .title("🖥️  CPU")
            .title_style(Style::themed(Role::Heading3)),
    );
    f.render_widget(cpu_widget, metrics_chunks[0]);

    // Memory Usage
    let memory_widget = Paragraph::new(Text::from(vec![
        Line::from(""),
        Line::from(Span::styled("2.4 GB", Style::themed(Role::Success).bold())),
        Line::from(""),
        Line::from(Span::styled("Normal", Style::themed(Role::Success))),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().with_role(Role::Success))
            .title("💾 Memory")
            .title_style(Style::themed(Role::Heading3)),
    );
    f.render_widget(memory_widget, metrics_chunks[1]);

    // Network
    let network_widget = Paragraph::new(Text::from(vec![
        Line::from(""),
        Line::from(Span::styled("OFFLINE", Style::themed(Role::Error).bold())),
        Line::from(""),
        Line::from(Span::styled("Disconnected", Style::themed(Role::Error))),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().with_role(Role::Error))
            .title("🌐 Network")
            .title_style(Style::themed(Role::Heading3)),
    );
    f.render_widget(network_widget, metrics_chunks[2]);

    // System Status
    let status_text = Text::from(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("System Status: ", Style::themed(Role::Normal)),
            Span::styled("RUNNING", Style::themed(Role::Success).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Uptime: ", Style::themed(Role::Subtle)),
            Span::styled("2d 14h 32m", Style::themed(Role::Code)),
        ]),
        Line::from(vec![
            Span::styled("Last Update: ", Style::themed(Role::Subtle)),
            Span::styled("2025-09-15 09:42:13", Style::themed(Role::Code)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Active Processes: ", Style::themed(Role::Normal)),
            Span::styled("127", Style::themed(Role::Info).bold()),
        ]),
        Line::from(vec![
            Span::styled("Load Average: ", Style::themed(Role::Normal)),
            Span::styled("1.23, 1.56, 1.78", Style::themed(Role::Warning)),
        ]),
    ]);

    let status_widget = Paragraph::new(status_text)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::themed(Role::Subtle))
                .title("ℹ️  System Information")
                .title_style(Style::themed(Role::Heading3)),
        );
    f.render_widget(status_widget, chunks[2]);
}

fn render_logs_tab(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .log_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let level_icon = match entry.level {
                Role::Error => "❌",
                Role::Warning => "⚠️ ",
                Role::Success => "✅",
                Role::Info => "ℹ️ ",
                _ => "📝",
            };

            let timestamp = format!(
                "2025-09-15 {:02}:{:02}:{:02}",
                9 + i / 10,
                (i * 7) % 60,
                (i * 13) % 60
            );

            let content = vec![Line::from(vec![
                Span::styled(format!("[{}] ", timestamp), Style::themed(Role::Subtle)),
                Span::styled(level_icon, Style::themed(entry.level)),
                Span::styled(format!(" {}", entry.message), Style::themed(entry.level)),
            ])];

            ListItem::new(content)
        })
        .collect();

    let logs_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::themed(Role::Subtle))
                .title("📋 Application Logs")
                .title_style(Style::themed(Role::Heading3)),
        )
        .highlight_style(Style::themed(Role::Emphasis))
        .highlight_symbol("▶ ");

    f.render_widget(logs_list, area);

    // Add scrollbar
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    let mut scrollbar_state = ScrollbarState::new(app.log_entries.len());
    scrollbar_state = scrollbar_state.position(app.scroll_position);
    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

fn render_settings_tab(f: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Theme settings
            Constraint::Length(6), // Display settings
            Constraint::Min(0),    // Advanced settings
        ])
        .split(area);

    // Theme Settings
    let theme_text = Text::from(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Current Theme: ", Style::themed(Role::Normal)),
            Span::styled("Auto-detected", Style::themed(Role::Success).bold()),
        ]),
        Line::from(vec![
            Span::styled("Color Support: ", Style::themed(Role::Normal)),
            Span::styled("True Color (16M)", Style::themed(Role::Info)),
        ]),
        Line::from(""),
    ]);

    let theme_widget = Paragraph::new(theme_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::themed(Role::Subtle))
            .title("🎨 Theme Settings")
            .title_style(Style::themed(Role::Heading3)),
    );
    f.render_widget(theme_widget, chunks[0]);

    // Display Settings
    let display_text = Text::from(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Terminal: ", Style::themed(Role::Normal)),
            Span::styled(
                std::env::var("TERM").unwrap_or_else(|_| "unknown".to_string()),
                Style::themed(Role::Code),
            ),
        ]),
        Line::from(vec![
            Span::styled("Size: ", Style::themed(Role::Normal)),
            Span::styled(
                format!("{}×{}", area.width, area.height),
                Style::themed(Role::Code),
            ),
        ]),
        Line::from(""),
    ]);

    let display_widget = Paragraph::new(display_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::themed(Role::Subtle))
            .title("🖥️  Display Settings")
            .title_style(Style::themed(Role::Heading3)),
    );
    f.render_widget(display_widget, chunks[1]);

    // Advanced Settings
    let advanced_text = Text::from(vec![
        Line::from(""),
        Line::from("Configuration options:"),
        Line::from(""),
        Line::from(vec![
            Span::styled("• Theme detection: ", Style::themed(Role::Normal)),
            Span::styled("Automatic", Style::themed(Role::Success)),
        ]),
        Line::from(vec![
            Span::styled("• Color scheme: ", Style::themed(Role::Normal)),
            Span::styled("System default", Style::themed(Role::Info)),
        ]),
        Line::from(vec![
            Span::styled("• Refresh rate: ", Style::themed(Role::Normal)),
            Span::styled("60 FPS", Style::themed(Role::Code)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::themed(Role::Subtle)),
            Span::styled("h", Style::themed(Role::Code).bold()),
            Span::styled(" for help", Style::themed(Role::Subtle)),
        ]),
    ]);

    let advanced_widget = Paragraph::new(advanced_text)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::themed(Role::Subtle))
                .title("⚙️  Advanced Configuration")
                .title_style(Style::themed(Role::Heading3)),
        );
    f.render_widget(advanced_widget, chunks[2]);
}

fn render_about_tab(f: &mut Frame, area: Rect) {
    let about_text = Text::from(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "🎨 Thag Styling + Ratatui Integration",
            Style::themed(Role::Heading1).bold(),
        )]),
        Line::from(""),
        Line::from("This showcase demonstrates the seamless integration between"),
        Line::from("thag_styling's semantic theming system and ratatui's TUI framework."),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Features demonstrated:",
            Style::themed(Role::Heading2).bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("✅ ", Style::themed(Role::Success)),
            Span::styled("Automatic theme detection", Style::themed(Role::Normal)),
        ]),
        Line::from(vec![
            Span::styled("✅ ", Style::themed(Role::Success)),
            Span::styled("Semantic role-based styling", Style::themed(Role::Normal)),
        ]),
        Line::from(vec![
            Span::styled("✅ ", Style::themed(Role::Success)),
            Span::styled("ThemedStyle trait integration", Style::themed(Role::Normal)),
        ]),
        Line::from(vec![
            Span::styled("✅ ", Style::themed(Role::Success)),
            Span::styled(
                "Extension methods for existing widgets",
                Style::themed(Role::Normal),
            ),
        ]),
        Line::from(vec![
            Span::styled("✅ ", Style::themed(Role::Success)),
            Span::styled(
                "Consistent color schemes across components",
                Style::themed(Role::Normal),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Code examples:",
            Style::themed(Role::Heading2).bold(),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "// Using ThemedStyle trait",
            Style::themed(Role::Code),
        )]),
        Line::from(vec![Span::styled(
            "let style = Style::themed(Role::Error);",
            Style::themed(Role::Code),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "// Using extension methods",
            Style::themed(Role::Code),
        )]),
        Line::from(vec![Span::styled(
            "let themed = base_style.with_role(Role::Success);",
            Style::themed(Role::Code),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Built with 🤖 using:",
            Style::themed(Role::Emphasis),
        )]),
        Line::from(vec![
            Span::styled("• ", Style::themed(Role::Subtle)),
            Span::styled("thag_styling", Style::themed(Role::Info).bold()),
            Span::styled(" - Semantic terminal theming", Style::themed(Role::Normal)),
        ]),
        Line::from(vec![
            Span::styled("• ", Style::themed(Role::Subtle)),
            Span::styled("ratatui", Style::themed(Role::Info).bold()),
            Span::styled(
                " - Terminal user interface library",
                Style::themed(Role::Normal),
            ),
        ]),
        Line::from(vec![
            Span::styled("• ", Style::themed(Role::Subtle)),
            Span::styled("crossterm", Style::themed(Role::Info).bold()),
            Span::styled(
                " - Cross-platform terminal API",
                Style::themed(Role::Normal),
            ),
        ]),
    ]);

    let about_widget = Paragraph::new(about_text)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::themed(Role::Subtle))
                .title("📖 About")
                .title_style(Style::themed(Role::Heading3)),
        );
    f.render_widget(about_widget, area);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let footer_text = Line::from(vec![
        Span::styled("Press ", Style::themed(Role::Subtle)),
        Span::styled("q", Style::themed(Role::Code).bold()),
        Span::styled(" to quit • ", Style::themed(Role::Subtle)),
        Span::styled("h", Style::themed(Role::Code).bold()),
        Span::styled(" for help • ", Style::themed(Role::Subtle)),
        Span::styled("Tab/Arrow", Style::themed(Role::Code).bold()),
        Span::styled(" to navigate • ", Style::themed(Role::Subtle)),
        Span::styled("1-4", Style::themed(Role::Code).bold()),
        Span::styled(" for direct tab access", Style::themed(Role::Subtle)),
    ]);

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::themed(Role::Subtle)),
        );
    f.render_widget(footer, area);
}

fn render_help_popup(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 50, area);

    f.render_widget(Clear, popup_area);

    let help_text = Text::from(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "🎨 Thag Styling + Ratatui Help",
            Style::themed(Role::Heading1).bold(),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::themed(Role::Heading2).bold(),
        )]),
        Line::from(vec![
            Span::styled("  Tab / →     ", Style::themed(Role::Code)),
            Span::styled("Next tab", Style::themed(Role::Normal)),
        ]),
        Line::from(vec![
            Span::styled("  Shift+Tab / ←  ", Style::themed(Role::Code)),
            Span::styled("Previous tab", Style::themed(Role::Normal)),
        ]),
        Line::from(vec![
            Span::styled("  1, 2, 3, 4  ", Style::themed(Role::Code)),
            Span::styled("Direct tab access", Style::themed(Role::Normal)),
        ]),
        Line::from(vec![
            Span::styled("  ↑ / ↓       ", Style::themed(Role::Code)),
            Span::styled("Scroll content", Style::themed(Role::Normal)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Actions:",
            Style::themed(Role::Heading2).bold(),
        )]),
        Line::from(vec![
            Span::styled("  h / F1      ", Style::themed(Role::Code)),
            Span::styled("Toggle this help", Style::themed(Role::Normal)),
        ]),
        Line::from(vec![
            Span::styled("  q           ", Style::themed(Role::Code)),
            Span::styled("Quit application", Style::themed(Role::Normal)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Theming Features:",
            Style::themed(Role::Heading2).bold(),
        )]),
        Line::from(vec![Span::styled(
            "• Automatic theme detection",
            Style::themed(Role::Success),
        )]),
        Line::from(vec![Span::styled(
            "• Semantic role-based coloring",
            Style::themed(Role::Info),
        )]),
        Line::from(vec![Span::styled(
            "• Consistent styling across widgets",
            Style::themed(Role::Emphasis),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press any key to close this help",
            Style::themed(Role::Subtle).italic(),
        )]),
        Line::from(""),
    ]);

    let help_popup = Paragraph::new(help_text)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::themed(Role::Info))
                .border_type(BorderType::Thick)
                .title("Help")
                .title_style(Style::themed(Role::Heading1).bold()),
        );

    f.render_widget(help_popup, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert_eq!(app.tab_index, 0);
        assert!(!app.log_entries.is_empty());
    }

    #[test]
    fn test_navigation() {
        let mut app = App::new();

        // Test tab navigation
        app.next_tab();
        assert_eq!(app.tab_index, 1);

        app.previous_tab();
        assert_eq!(app.tab_index, 0);

        // Test wrapping
        app.tab_index = 3;
        app.next_tab();
        assert_eq!(app.tab_index, 0);
    }

    #[test]
    fn test_themed_styles() {
        // Test ThemedStyle trait implementations
        let error_style = Style::themed(Role::Error);
        let success_color = Color::themed(Role::Success);

        // Styles should not be default
        assert_ne!(error_style, Style::default());
        assert_ne!(success_color, Color::Reset);
    }

    #[test]
    fn test_extension_methods() {
        // Test RatatuiStyleExt trait
        let base_style = Style::default().bold();
        let themed_style = base_style.with_role(Role::Info);

        // Should preserve original modifiers
        assert!(themed_style
            .add_modifier
            .intersects(ratatui::style::Modifier::BOLD));
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry {
            level: Role::Warning,
            message: "Test message".to_string(),
        };

        assert_eq!(entry.level, Role::Warning);
        assert_eq!(entry.message, "Test message");
    }
}
