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
/// A basic prototype GUI markdown viewer using `inquire` to select a markdown file and `egui_commonmark`
/// to display it. Relative links are resolved relative to the parent directory of the current markdown
/// file, so navigation between linked documents works correctly.
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
    let selected_path = std::path::PathBuf::from(&selected_file);
    let canonical_initial_path = selected_path.canonicalize().unwrap_or(selected_path);
    // Keep the process CWD in sync with the file so egui_extras resolves
    // relative image URIs correctly from the start.
    if let Some(parent) = canonical_initial_path.parent() {
        let _ = std::env::set_current_dir(parent);
    }

    let markdown_content = std::fs::read_to_string(&canonical_initial_path)
        .unwrap_or_else(|_| format!("# Error\nFailed to read `{}`.", selected_file.display()));

    // Set up Native GUI Options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title(format!("Viewing: {}", selected_file.display())),
        ..Default::default()
    };

    // Start the egui app passing BOTH the content and the starting path
    eframe::run_native(
        "Markdown Viewer",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(MarkdownApp::new(
                markdown_content,
                canonical_initial_path,
            )))
        }),
    )
}

/// The state holder for our egui app
struct MarkdownApp {
    /// The actual raw markdown text currently loaded
    content: String,
    /// The canonicalized path of the file we are viewing (so we know its parent folder)
    current_file_path: std::path::PathBuf,
    /// Required by egui_commonmark for rendering images/styles
    cache: CommonMarkCache,
}

impl MarkdownApp {
    fn new(content: String, current_file_path: std::path::PathBuf) -> Self {
        Self {
            content,
            current_file_path,
            cache: CommonMarkCache::default(),
        }
    }

    /// Tries to resolve a clicked relative link, read the file, and swap it in.  Returns `true`
    /// on success so the caller can update the window title.
    fn handle_link_click(&mut self, clicked_url: &str) -> bool {
        // Strip any fragment identifier (#anchor) — it's not part of the file path.
        let url_path = match clicked_url.split_once('#') {
            Some((path, _fragment)) => path,
            None => clicked_url,
        };
        if url_path.is_empty() {
            return false; // Pure anchor link with no file component
        }

        // 1. Get the directory of the currently open file.
        //    `current_file_path` is always canonicalized (absolute), so `parent()` is reliable.
        let current_dir = match self.current_file_path.parent() {
            Some(parent) => parent.to_path_buf(),
            None => std::path::PathBuf::from("."),
        };

        // 2. Resolve the clicked path relative to that directory.
        let mut target_path = current_dir.join(url_path);

        // 3. Canonicalize to resolve '..' / '.' components and confirm the file exists.
        //    Setting the CWD first ensures canonicalize works even when target_path is still
        //    relative (e.g. if the initial canonicalization in main() failed).
        if let Ok(canonical) = target_path.canonicalize() {
            target_path = canonical;
        }

        // 4. Try to read the new file and update state.
        match std::fs::read_to_string(&target_path) {
            Ok(new_content) => {
                self.content = new_content;
                self.current_file_path = target_path.clone();
                // Keep CWD in sync so future canonicalize() calls and image loading work correctly.
                if let Some(new_dir) = target_path.parent() {
                    let _ = std::env::set_current_dir(new_dir);
                }
                // Clear the cache so egui_commonmark doesn't carry over scroll positions.
                self.cache = CommonMarkCache::default();
                true
            }
            Err(e) => {
                eprintln!("Failed to follow relative link {clicked_url:?}: {e}");
                false
            }
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

        // egui_commonmark dispatches link clicks by pushing OutputCommand::OpenUrl onto the
        // context output. Intercept here: handle relative links ourselves, re-queue external ones.
        let clicked_url = ui.ctx().output_mut(|o| {
            let pos = o
                .commands
                .iter()
                .position(|cmd| matches!(cmd, egui::OutputCommand::OpenUrl(_)));
            pos.map(|idx| {
                if let egui::OutputCommand::OpenUrl(open_url) = o.commands.remove(idx) {
                    open_url.url
                } else {
                    unreachable!()
                }
            })
        });

        if let Some(url) = clicked_url {
            if url.starts_with("http://") || url.starts_with("https://") {
                // Re-queue external links for the platform to open in the browser.
                ui.ctx().open_url(egui::output::OpenUrl::new_tab(url));
            } else if self.handle_link_click(&url) {
                // Navigation succeeded — update the window title to the new file path.
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Title(format!(
                        "Viewing: {}",
                        self.current_file_path.display()
                    )));
            }
        }
    }
}
