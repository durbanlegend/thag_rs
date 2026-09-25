/*[toml]
[package]
name = "tmtheme_demo"
version = "0.1.0"
edition = "2024"

[dependencies]
eframe = "0.36"
egui_extras = { version = "0.36", features = ["syntect"] }
# Same feature set egui_extras uses (pure-Rust regex), so we don't pull in a second regex engine.
syntect = { version = "5.3", default-features = false, features = ["default-fancy"] }
 */
/// Minimal egui app: highlight a Rust snippet with a syntect theme loaded from a `.tmTheme` file.
///
/// Usage: `thag demo/egui_syntect_highlighting.rs -- path/to/MyTheme.tmTheme`
/// (defaults to `theme.tmTheme` in the current directory)
//# Purpose: demo and test `egui_extras` code block formatting
//# Categories: crates, demo, styling, technique
//# Argument: PATH: Path to a `syntect` `.tmTheme` file. There are a few examples in the `thag_rs/assets/sublime_themes` directory and `egui_extras` has its own.
use eframe::egui;
use egui_extras::syntax_highlighting::{self, CodeTheme, SyntectSettings};
use syntect::highlighting::{Theme, ThemeSet};

const CODE: &str = r#"use std::collections::HashMap;

/// A tiny word counter.
#[derive(Debug, Default)]
struct Counter {
    counts: HashMap<String, usize>,
}

impl Counter {
    fn add(&mut self, word: &str) {
        *self.counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
}

fn main() {
    let mut counter = Counter::default();
    for word in "the quick brown fox jumps over the lazy dog".split_whitespace() {
        counter.add(word);
    }

    // Most common words first.
    let mut pairs: Vec<_> = counter.counts.iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(a.1));

    let answer: i32 = 42;
    println!("{pairs:?} {answer}");
}
"#;

struct App {
    /// Must stay at a stable address: egui_extras memoizes highlighting by the *address* of this.
    settings: SyntectSettings,
    /// The theme's own background colour (egui_extras only applies foreground colours).
    background: Option<egui::Color32>,
}

impl App {
    fn new(theme: Theme) -> Self {
        let background = theme
            .settings
            .background
            .map(|c| egui::Color32::from_rgb(c.r, c.g, c.b));

        // `CodeTheme` picks its syntect theme through a private enum, and then looks the name up in
        // `settings.ts.themes`. We can't select a custom theme by name, so instead we overwrite
        // every slot with ours. Whichever one `CodeTheme` asks for, it gets the .tmTheme.
        let mut settings = SyntectSettings::default();
        for slot in settings.ts.themes.values_mut() {
            *slot = theme.clone();
        }

        Self {
            settings,
            background,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut frame = egui::Frame::central_panel(ui.style());
        if let Some(bg) = self.background {
            frame = frame.fill(bg);
        }

        egui::CentralPanel::default().frame(frame).show(ui, |ui| {
            // Only used to pick font size / dark-vs-light fallback; the colours come from the .tmTheme.
            let code_theme = CodeTheme::from_style(ui.style());

            let job = syntax_highlighting::highlight_with(
                ui.ctx(),
                ui.style(),
                &code_theme,
                CODE,
                "rs", // syntax name ("Rust") or file extension
                &self.settings,
            );

            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                ui.add(
                    egui::Label::new(job)
                        .selectable(true)
                        .wrap_mode(egui::TextWrapMode::Extend),
                );
            });
        });
    }
}

fn main() -> eframe::Result {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "theme.tmTheme".to_owned());

    let theme = ThemeSet::get_theme(&path).unwrap_or_else(|err| {
        eprintln!("Could not load theme {path:?}: {err}");
        std::process::exit(1);
    });

    eframe::run_native(
        "tmTheme demo",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(App::new(theme)))),
    )
}
