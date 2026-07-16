/*[toml]
[dependencies]
thag_proc_macros = { version = "1, thag-auto" }
thag_styling = { version = "1, thag-auto", features = ["inquire_theming"] }

[features]
default = ["egui_commonmark/better_syntax_highlighting","egui_commonmark/svg","egui_commonmark/fetch"]

# Make sure the result runs fast
[profile.dev]
opt-level = 3     # Apply maximum performance optimizations
*/
/// A basic prototype GUI markdown viewer using `inquire` to select a markdown file and `egui_commonmark`.
/// To display it. Relative links are only resolved relative to the current working directory, not as they
/// should be to the parent directory of the current markdown file.
/// See the `md-viewer` crate for a professional quality installable example using `egui_commonmark`.//# Purpose: Prototype a markdown viewer using the `egui_commonmark` crate.
//# Categories: crates, demo, gui, prototype, tools
use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use thag_styling::{file_navigator, themed_inquire_config};

file_navigator! {}

fn main() -> eframe::Result<()> {
    inquire::set_global_render_config(themed_inquire_config());

    let mut navigator = FileNavigator::new();
    let selected_file = select_file(&mut navigator, Some("md"), false).unwrap();
    let selected_file = selected_file.canonicalize().unwrap_or(selected_file);

    let markdown_content = std::fs::read_to_string(&selected_file)
        .unwrap_or_else(|_| format!("# Error\nFailed to read `{}`.", selected_file.display()));

    // Set up Native GUI Options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title(format!("Viewing: {}", selected_file.display())),
        ..Default::default()
    };

    // Start the egui application passing the Markdown content
    eframe::run_native(
        "Markdown Viewer",
        options,
        Box::new(|_cc| Ok(Box::new(MarkdownApp::new(markdown_content)))),
    )
}

/// The state holder for our egui app
struct MarkdownApp {
    content: String,
    cache: CommonMarkCache,
}

impl MarkdownApp {
    fn new(content: String) -> Self {
        Self {
            content,
            cache: CommonMarkCache::default(),
        }
    }
}

impl eframe::App for MarkdownApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            // Put markdown inside a scrollable area in case it is long
            egui::ScrollArea::vertical().show(ui, |ui| {
                CommonMarkViewer::new().show(ui, &mut self.cache, &self.content);
            });
        });
    }
}
