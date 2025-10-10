/*[toml]
[dependencies]
crossterm = { version = "0.28.1" }
thag_common = { version = "0.2, thag-auto" }
thag_styling = { version = "0.2, thag-auto", features = ["full"] }

 [features]
 # default = ["full"]
 default = ["color_detect", "crossterm_support", "inquire_theming", "nu_ansi_term_support", "ratatui_support"]

 # Core styling without external dependencies
 basic = []

 # Terminal color detection and background detection
 color_detect = ["thag_common/color_detect"]

 config = ["thag_common/config"]

 # Tools integration
 tools = []

 # Debug logging support
 debug_logging = ["thag_common/debug_logging"]

 # Inquire integration for themed UI
 inquire_theming = ["thag_styling/inquire_theming"]

 # Full ratatui integration
 ratatui_support = ["thag_styling/ratatui_support"]

 # Support for thag REPL
 nu_ansi_term_support = ["thag_styling/nu_ansi_term_support"]

 # Crossterm integration for cross-platform terminal manipulation
 crossterm_support = ["thag_styling/crossterm_support"]

 # Console integration for popular styling library
 console_support = ["thag_styling/console_support"]

 # All advanced features
 full = [
     "color_detect",
     "config",
     "console_support",
     "crossterm_support",
     "inquire_theming",
     "ratatui_support",
     "tools",
 ]
*/

/// Zero-configuration setup example for thag_styling
///
/// This example demonstrates how thag_styling can be used with zero configuration
/// while automatically detecting terminal capabilities and choosing appropriate themes.
///
/// Run with:
/// ```bash
/// cargo run -p thag_styling --example zero_config --features "color_detect,crossterm_support,ratatui_support,nu_ansi_term_support"
/// # Or use the full feature set:
/// cargo run -p thag_styling --example zero_config --features "full"
/// ```
#[cfg(feature = "ratatui_support")]
use std::error::Error;
use std::io::{self};
#[cfg(feature = "ratatui_support")]
use thag_styling::ThemedStyle;
use thag_styling::{paint_for_role, Role, TermAttributes};

fn main() -> io::Result<()> {
    println!("🎨 Thag Styling - Zero Configuration Demo\n");

    // Zero config step 1: Just use it! Terminal detection happens automatically
    show_automatic_detection();

    // Zero config step 2: Cross-library consistency without setup
    show_cross_library_consistency()?;

    // Zero config step 3: Interactive prompts work out of the box
    #[cfg(feature = "inquire_theming")]
    show_interactive_prompts()?;

    // Zero config step 4: Advanced features just work
    show_advanced_features();

    example_cli_app();
    Ok(())
}

fn show_automatic_detection() {
    println!("📡 Automatic Terminal Detection:");
    println!("  (No configuration required - everything detected automatically)\n");

    // Display what was detected
    let term_attrs = TermAttributes::get_or_init();
    println!("  🖥️  Terminal Capabilities:");
    println!("    Color Support: {:?}", term_attrs.color_support);
    println!("    Background:    {:?}", term_attrs.term_bg_luma);
    println!("    Theme:         {}", term_attrs.theme.name);

    if let Some([r, g, b]) = term_attrs.term_bg_rgb {
        println!("    BG Color:      RGB({r}, {g}, {b})");
    }

    println!("    How Set:       {:?}", term_attrs.how_initialized);
    println!();
}

fn show_cross_library_consistency() -> io::Result<()> {
    println!("🔗 Cross-Library Consistency:");
    println!("  (Same colors across all libraries, zero setup)\n");

    let messages = [
        (Role::Success, "✓ Operation completed successfully"),
        (Role::Error, "✗ Critical error occurred"),
        (Role::Warning, "⚠ Warning: proceed with caution"),
        (Role::Info, "ℹ Informational message"),
        (Role::Code, "let result = perform_operation();"),
        (Role::Emphasis, "This text is emphasized"),
        (Role::Heading1, "# Main Section Header"),
        (Role::Heading2, "## Subsection Header"),
        (Role::Normal, "Regular paragraph text"),
        (Role::Subtle, "Less important details"),
    ];

    // Show the same styling works across different contexts
    println!("  📝 Standard Output:");
    for (role, message) in &messages {
        println!(
            "    {}: {}",
            format!("{role:12}"),
            paint_for_role(*role, message)
        );
    }
    println!();

    // Crossterm integration example
    #[cfg(feature = "crossterm_support")]
    {
        use crossterm::style::ContentStyle;
        use crossterm::{execute, style::Print};

        println!("  🔧 Crossterm Integration:");
        let mut stdout = io::stdout();

        execute!(
            stdout,
            Print("    Success: "),
            Print(crossterm::style::StyledContent::new(
                ContentStyle::themed(Role::Success),
                "Crossterm themed content"
            )),
            Print("\n")
        )?;

        execute!(
            stdout,
            Print("    Error:   "),
            Print(crossterm::style::StyledContent::new(
                ContentStyle::themed(Role::Error),
                "Error message in crossterm"
            )),
            Print("\n")
        )?;
        println!();
    }

    // Ratatui integration example
    #[cfg(feature = "ratatui_support")]
    {
        use ratatui::style::Style;

        println!("  📊 Ratatui Integration:");
        let success_style = Style::themed(Role::Success);
        let error_style = Style::themed(Role::Error);

        println!("    Success Style: {success_style:?}");
        println!("    Error Style:   {error_style:?}");
        println!();

        ratatui_user_input_example();
    }

    // Nu-ANSI-Term integration example
    #[cfg(feature = "nu_ansi_term_support")]
    {
        use nu_ansi_term::Style;

        println!("  🐚 Nu-ANSI-Term Integration:");
        let success_style = Style::themed(Role::Success);
        let error_style = Style::themed(Role::Error);

        println!(
            "    {}",
            success_style.paint("Success: Nu-ANSI-Term themed content")
        );
        println!(
            "    {}",
            error_style.paint("Error: Error message in nu-ansi-term")
        );
        println!();
    }

    Ok(())
}

#[cfg(feature = "ratatui_support")]
fn ratatui_user_input_example() {
    use ratatui::{
        crossterm::event::{self, Event, KeyCode, KeyEventKind},
        layout::{Constraint, Layout, Position},
        style::{Style, Stylize},
        text::{Line, Span},
        widgets::{Block, List, ListItem, Paragraph},
        DefaultTerminal, Frame,
    };
    use thag_styling::{Role, ThemedStyle}; // Added for thag styling

    /// App holds the state of the application
    struct App {
        /// Current value of the input box
        input: String,
        /// Position of cursor in the editor area.
        character_index: usize,
        /// Current input mode
        input_mode: InputMode,
        /// History of recorded messages
        messages: Vec<String>,
    }

    enum InputMode {
        Normal,
        Editing,
    }

    impl App {
        const fn new() -> Self {
            Self {
                input: String::new(),
                input_mode: InputMode::Normal,
                messages: Vec::new(),
                character_index: 0,
            }
        }

        fn move_cursor_left(&mut self) {
            let cursor_moved_left = self.character_index.saturating_sub(1);
            self.character_index = self.clamp_cursor(cursor_moved_left);
        }

        fn move_cursor_right(&mut self) {
            let cursor_moved_right = self.character_index.saturating_add(1);
            self.character_index = self.clamp_cursor(cursor_moved_right);
        }

        fn enter_char(&mut self, new_char: char) {
            let index = self.byte_index();
            self.input.insert(index, new_char);
            self.move_cursor_right();
        }

        /// Returns the byte index based on the character position.
        ///
        /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
        /// the byte index based on the index of the character.
        fn byte_index(&self) -> usize {
            self.input
                .char_indices()
                .map(|(i, _)| i)
                .nth(self.character_index)
                .unwrap_or(self.input.len())
        }

        fn delete_char(&mut self) {
            let is_not_cursor_leftmost = self.character_index != 0;
            if is_not_cursor_leftmost {
                // Method "remove" is not used on the saved text for deleting the selected char.
                // Reason: Using remove on String works on bytes instead of the chars.
                // Using remove would require special care because of char boundaries.

                let current_index = self.character_index;
                let from_left_to_current_index = current_index - 1;

                // Getting all characters before the selected character.
                let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
                // Getting all characters after selected character.
                let after_char_to_delete = self.input.chars().skip(current_index);

                // Put all characters together except the selected one.
                // By leaving the selected one out, it is forgotten and therefore deleted.
                self.input = before_char_to_delete.chain(after_char_to_delete).collect();
                self.move_cursor_left();
            }
        }

        fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
            new_cursor_pos.clamp(0, self.input.chars().count())
        }

        fn reset_cursor(&mut self) {
            self.character_index = 0;
        }

        fn submit_message(&mut self) {
            self.messages.push(self.input.clone());
            self.input.clear();
            self.reset_cursor();
        }

        fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), Box<dyn Error>> {
            loop {
                terminal.draw(|frame| self.draw(frame))?;

                if let Event::Key(key) = event::read()? {
                    match self.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('e') => {
                                self.input_mode = InputMode::Editing;
                            }
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            _ => {}
                        },
                        InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                            KeyCode::Enter => self.submit_message(),
                            KeyCode::Char(to_insert) => self.enter_char(to_insert),
                            KeyCode::Backspace => self.delete_char(),
                            KeyCode::Left => self.move_cursor_left(),
                            KeyCode::Right => self.move_cursor_right(),
                            KeyCode::Esc => self.input_mode = InputMode::Normal,
                            _ => {}
                        },
                        InputMode::Editing => {}
                    }
                }
            }
        }

        fn draw(&self, frame: &mut Frame) {
            // Enlarge to accommodate thag styling welcome header
            let vertical = Layout::vertical([
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Min(2),
            ]);
            let [help_area, input_area, messages_area] = vertical.areas(frame.area());

            let (msg, style) = match self.input_mode {
                InputMode::Normal => (
                    vec![
                        "Press ".into(),
                        "q".italic(),
                        " to exit, ".into(),
                        "e".italic(),
                        " to start editing.".into(), // .bold() not needed
                    ],
                    Style::themed(Role::HD2), // for thag styling
                ),
                InputMode::Editing => (
                    vec![
                        "Press ".into(),
                        "Esc".italic(),
                        " to stop editing, ".into(),
                        "Enter".italic(),
                        " to record the message".into(),
                    ],
                    Style::themed(Role::HD2), // for thag styling
                ),
            };
            // Added thag styling intro
            let text = vec![
                Line::from("Welcome to the Thag-styled version of the Ratatui User Input example.")
                    .style(Style::themed(Role::HD1)),
                Line::from(msg).style(style), // Optional replace patch
            ];
            let help_message = Paragraph::new(text);
            frame.render_widget(help_message, help_area);

            let input = Paragraph::new(self.input.as_str())
                .style(match self.input_mode {
                    InputMode::Normal => Style::themed(Role::Normal), // Replace styles by thag styling
                    InputMode::Editing => Style::themed(Role::Code),
                })
                .block(
                    Block::bordered()
                        .title("Input")
                        .style(Style::themed(Role::HD3)), // title styling by thag
                );
            frame.render_widget(input, input_area);
            match self.input_mode {
                // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
                InputMode::Normal => {}

                // Make the cursor visible and ask ratatui to put it at the specified coordinates after
                // rendering
                #[allow(clippy::cast_possible_truncation)]
                InputMode::Editing => frame.set_cursor_position(Position::new(
                    // Draw the cursor at the current position in the input field.
                    // This position is can be controlled via the left and right arrow key
                    input_area.x + self.character_index as u16 + 1,
                    // Move one line down, from the border to the input line
                    input_area.y + 1,
                )),
            }

            let messages: Vec<ListItem> = self
                .messages
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    let content = Line::from(Span::raw(format!("{i}: {m}")));
                    ListItem::new(content)
                })
                .collect();
            let messages = List::new(messages)
                .block(
                    Block::bordered()
                        .title("Messages")
                        .style(Style::themed(Role::EMPH)), // title styling by thag
                )
                .style(Style::themed(Role::INFO)); // Added thag styling
            frame.render_widget(messages, messages_area);
        }
    }

    let terminal = ratatui::init();
    let _ = App::new().run(terminal);
    ratatui::restore();
}

#[cfg(feature = "inquire_theming")]
fn show_interactive_prompts() -> io::Result<()> {
    use inquire::set_global_render_config; // For Option B
    use inquire::{Confirm, Select, Text};
    use thag_styling::themed_inquire_config; // For Option B

    println!("💬 Interactive Prompts with `inquire`:");
    println!("  (Automatically themed to match your terminal)\n");

    // Option A: Use the themed inquire config - requires `with_render_config` at each `inquire` prompt.
    let config = thag_styling::themed_inquire_config();

    // Simple text input with theming
    if let Ok(name) = Text::new("What's your name?")
        .with_render_config(config.clone())
        .prompt()
    {
        println!("  Hello, {}! 👋\n", paint_for_role(Role::Emphasis, &name));
    }

    // Option B (recommended): Even simpler: set the default
    set_global_render_config(themed_inquire_config());

    // Selection with themed options
    let options = vec!["Coffee", "Tea", "Soft drink", "Water", "Other"];
    if let Ok(choice) = Select::new("What can I offer you?", options).prompt() {
        println!("\n  You chose: {}\n", paint_for_role(Role::Success, choice));
    }

    // Confirmation with theming
    if let Ok(confirmed) = Confirm::new("Continue with zero-config demo?")
        .with_default(true)
        .prompt()
    {
        if confirmed {
            println!("  {}", paint_for_role(Role::Success, "✓ Continuing..."));
        } else {
            println!("  {}", paint_for_role(Role::Info, "Demo stopped by user"));
        }
    }

    println!();
    Ok(())
}

fn show_advanced_features() {
    println!("🚀 Advanced Features (Zero Config):");
    println!("  (All features work automatically)\n");

    // Show color adaptation
    println!("  🎭 Automatic Color Adaptation:");
    let term_attrs = TermAttributes::get_or_init();
    match term_attrs.term_bg_luma {
        thag_styling::TermBgLuma::Light => {
            println!("    Light background detected → Using dark colors for contrast");
        }
        thag_styling::TermBgLuma::Dark => {
            println!("    Dark background detected → Using light colors for contrast");
        }
        _ => {
            println!("    Background auto-detected → Colors automatically optimized");
        }
    }

    // Show capability matching
    println!("\n  🎨 Capability Matching:");
    match term_attrs.color_support {
        thag_styling::ColorSupport::TrueColor => {
            println!("    True color support → Using RGB colors for maximum fidelity");
        }
        thag_styling::ColorSupport::Color256 => {
            println!("    256-color support → Using optimized 256-color palette");
        }
        thag_styling::ColorSupport::Basic => {
            println!("    Basic color support → Using safe 8-color palette");
        }
        _ => {
            println!("    Color support auto-detected → Best available colors selected");
        }
    }

    // Show theme selection
    println!("\n  🌈 Smart Theme Selection:");
    println!(
        "    Current theme: {}",
        paint_for_role(Role::Emphasis, &term_attrs.theme.name)
    );
    println!(
        "    Theme family: {}",
        term_attrs.theme.name.split('_').next().unwrap_or("unknown")
    );
    println!("    Automatically chosen for your terminal setup");

    // Performance info
    println!("\n  ⚡ Performance:");
    println!("    • Zero runtime detection overhead (cached on first use)");
    println!("    • Optimized color calculations");
    println!("    • Minimal memory footprint");
    println!("    • Feature-gated dependencies");

    println!();
}

/// Example of a typical CLI application using zero-config thag_styling
fn example_cli_app() {
    println!("📋 Example CLI Application Output:\n");

    // Simulate a typical CLI tool
    println!("{}", paint_for_role(Role::Heading1, "MyTool v1.0.0"));
    println!(
        "{}",
        paint_for_role(
            Role::Subtle,
            "A sample CLI application with automatic theming"
        )
    );
    println!();

    // Status messages
    println!("{}", paint_for_role(Role::Info, "ℹ Initializing..."));
    println!(
        "{}",
        paint_for_role(Role::Normal, "→ Loading configuration")
    );
    println!(
        "{}",
        paint_for_role(Role::Success, "✓ Configuration loaded")
    );
    println!();

    // Processing steps
    let steps = [
        ("Validating input", Role::Info),
        ("Processing data", Role::Normal),
        ("Generating output", Role::Normal),
        ("Writing results", Role::Success),
    ];

    for (step, role) in steps {
        println!("{}", paint_for_role(role, &format!("→ {}", step)));
    }
    println!();

    // Results
    println!("{}", paint_for_role(Role::Heading2, "## Results"));
    println!(
        "{} Files processed: {}",
        paint_for_role(Role::Normal, "•"),
        paint_for_role(Role::Emphasis, "1,234")
    );
    println!(
        "{} Errors: {}",
        paint_for_role(Role::Normal, "•"),
        paint_for_role(Role::Error, "0")
    );
    println!(
        "{} Warnings: {}",
        paint_for_role(Role::Normal, "•"),
        paint_for_role(Role::Warning, "3")
    );
    println!();

    println!(
        "{}",
        paint_for_role(Role::Success, "✓ Operation completed successfully!")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_config_demo() {
        // Ensure the demo functions don't panic
        show_automatic_detection();
        show_advanced_features();
        example_cli_app();
    }

    #[test]
    fn test_cross_library_consistency() {
        // Test that cross-library demo runs without errors
        assert!(show_cross_library_consistency().is_ok());
    }
}
