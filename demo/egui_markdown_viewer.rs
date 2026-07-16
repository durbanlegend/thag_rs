/*[toml]
[dependencies]
thag_proc_macros = { version = "1, thag-auto" }
thag_styling = { version = "1, thag-auto", features = ["inquire_theming"] }
pulldown-cmark = "0.13"

[features]
default = ["eframe/wgpu", "egui_commonmark/better_syntax_highlighting","egui_commonmark/svg","egui_commonmark/fetch"]

# Make sure the result runs fast
[profile.dev]
opt-level = 3     # Apply maximum performance optimizations
*/
/// A basic prototype GUI markdown viewer using `inquire` to select a markdown file and `egui_commonmark`
/// to display it. Relative links are resolved relative to the parent directory of the current markdown
/// file, so navigation between linked documents works correctly. Supports back/forward history.
/// We also use the `eframe` WGPU renderer for fast rendering.
/// Image links such as `[![alt](img)](url)` are preprocessed to inject a visible `[↗ alt](url)` text
/// link alongside the (non-clickable) image, working around a known egui_commonmark limitation where
/// such patterns produce an invisible zero-size hyperlink.
/// See the `md-viewer` crate for a professional quality installable example using `egui_commonmark`.
//# Purpose: Prototype a markdown viewer using the `egui_commonmark` crate.
//# Categories: crates, demo, gui, prototype, tools
//# Usage: egui_markdown_viewer [OPTIONS] [path_to_file]
use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::path::{Path, PathBuf};
use thag_styling::{
    auto_help, file_navigator, help_system::check_help_and_exit, themed_inquire_config,
};

file_navigator! {}

/// Preprocesses markdown to work around egui_commonmark's handling of image links.
///
/// `[![alt](img)](url)` produces an invisible zero-size hyperlink because the alt text
/// goes into `image.alt_text` rather than `link.text`, leaving the link with no
/// renderable content. We inject `[↗ alt](url)` immediately after each such pattern
/// so the user has a visible, clickable text link alongside the image.
fn preprocess_markdown(content: &str) -> String {
    // Fast path: no image links present.
    if !content.contains("[![") {
        return content.to_string();
    }

    // Collect (byte_end_of_link, target_url, alt_text) for each image-link.
    let mut insertions: Vec<(usize, String, String)> = Vec::new();
    let mut link_url: Option<String> = None;
    let mut current_alt = String::new();
    let mut has_image = false;

    for (event, range) in Parser::new_ext(content, Options::empty()).into_offset_iter() {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => {
                link_url = Some(dest_url.to_string());
                current_alt.clear();
                has_image = false;
            }
            Event::Start(Tag::Image { .. }) if link_url.is_some() => {
                has_image = true;
            }
            // Alt text lives inside the Image event, not the Link event.
            Event::Text(text) if has_image && link_url.is_some() => {
                current_alt.push_str(&text);
            }
            Event::End(TagEnd::Link) if has_image => {
                if let Some(url) = link_url.take() {
                    insertions.push((range.end, url, current_alt.clone()));
                }
                has_image = false;
            }
            Event::End(TagEnd::Link) => {
                link_url = None;
            }
            _ => {}
        }
    }

    if insertions.is_empty() {
        return content.to_string();
    }

    let mut result = String::with_capacity(content.len() + insertions.len() * 32);
    let mut pos = 0;
    for (offset, url, alt) in &insertions {
        result.push_str(&content[pos..*offset]);
        if alt.is_empty() {
            result.push_str(&format!("[\u{2197} link]({url})"));
        } else {
            result.push_str(&format!("[\u{2197} {alt}]({url})"));
        }
        pos = *offset;
    }
    result.push_str(&content[pos..]);
    result
}

fn main() -> eframe::Result<()> {
    let help = auto_help!();
    check_help_and_exit(&help);

    let args: Vec<String> = env::args().collect();

    let selected_file: PathBuf = if args.len() > 1 {
        let input_path = Path::new(&args[1]);
        if !input_path.exists() {
            eprintln!(
                "Error: Input directory does not exist: {}",
                input_path.display()
            );
            std::process::exit(1);
        }
        if input_path.is_dir() {
            eprintln!("Error: Input file is a directory: {}", input_path.display());
            std::process::exit(1);
        }
        input_path.to_path_buf()
    } else {
        inquire::set_global_render_config(themed_inquire_config());

        let mut navigator = FileNavigator::new();
        select_file(&mut navigator, Some("md"), false).unwrap()
    };
    let selected_path = PathBuf::from(&selected_file);
    let canonical_initial_path = selected_path.canonicalize().unwrap_or(selected_path);
    // Keep the process CWD in sync with the file so egui_extras resolves
    // relative image URIs correctly from the start.
    if let Some(parent) = canonical_initial_path.parent() {
        let _ = std::env::set_current_dir(parent);
    }

    let raw_content = std::fs::read_to_string(&canonical_initial_path).unwrap_or_else(|_| {
        format!(
            "# Error\nFailed to read `{}`.",
            canonical_initial_path.display()
        )
    });
    let markdown_content = preprocess_markdown(&raw_content);

    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title(format!("Viewing: {}", canonical_initial_path.display())),
        ..Default::default()
    };

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

/// Pending navigation action triggered by the toolbar buttons.
enum NavAction {
    None,
    Back,
    Forward,
}

/// The state holder for our egui app.
struct MarkdownApp {
    /// The actual raw markdown text currently loaded.
    content: String,
    /// The canonicalized path of the file we are viewing (so we know its parent folder).
    current_file_path: PathBuf,
    /// Required by egui_commonmark for rendering images/styles.
    cache: CommonMarkCache,
    /// Ordered list of visited file paths.
    history: Vec<PathBuf>,
    /// Current position within `history`.
    history_index: usize,
}

impl MarkdownApp {
    fn new(content: String, path: PathBuf) -> Self {
        Self {
            content,
            current_file_path: path.clone(),
            cache: CommonMarkCache::default(),
            history: vec![path],
            history_index: 0,
        }
    }

    fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    fn can_go_forward(&self) -> bool {
        self.history_index + 1 < self.history.len()
    }

    /// Load `path` from disk and update content, current path, cache, and CWD.
    /// Returns `true` on success.
    fn load_file(&mut self, path: PathBuf) -> bool {
        match std::fs::read_to_string(&path) {
            Ok(new_content) => {
                // Keep CWD in sync so future canonicalize() calls and image loading work correctly.
                if let Some(dir) = path.parent() {
                    let _ = std::env::set_current_dir(dir);
                }
                self.content = preprocess_markdown(&new_content);
                self.current_file_path = path;
                // Clear the cache so egui_commonmark doesn't carry over scroll positions.
                self.cache = CommonMarkCache::default();
                true
            }
            Err(e) => {
                eprintln!("Failed to read {:?}: {e}", path);
                false
            }
        }
    }

    /// Navigate one step back in history. Returns `true` on success.
    fn go_back(&mut self) -> bool {
        if self.can_go_back() {
            self.history_index -= 1;
            let path = self.history[self.history_index].clone();
            self.load_file(path)
        } else {
            false
        }
    }

    /// Navigate one step forward in history. Returns `true` on success.
    fn go_forward(&mut self) -> bool {
        if self.can_go_forward() {
            self.history_index += 1;
            let path = self.history[self.history_index].clone();
            self.load_file(path)
        } else {
            false
        }
    }

    /// Resolve a clicked relative link, load it, and push it onto history (discarding any
    /// forward entries). Returns `true` on success so the caller can update the window title.
    fn handle_link_click(&mut self, clicked_url: &str) -> bool {
        // Strip any fragment identifier (#anchor) — it's not part of the file path.
        let url_path = match clicked_url.split_once('#') {
            Some((path, _fragment)) => path,
            None => clicked_url,
        };
        if url_path.is_empty() {
            return false; // Pure anchor link with no file component.
        }

        // Resolve relative to the current file's directory.
        // `current_file_path` is always canonicalized (absolute), so `parent()` is reliable.
        let current_dir = match self.current_file_path.parent() {
            Some(parent) => parent.to_path_buf(),
            None => PathBuf::from("."),
        };
        let mut target_path = current_dir.join(url_path);
        // Canonicalize to resolve '..' / '.' and confirm the file exists.
        // Setting CWD first (in load_file) ensures canonicalize works for relative fallbacks.
        if let Ok(canonical) = target_path.canonicalize() {
            target_path = canonical;
        }

        if self.load_file(target_path.clone()) {
            // Discard forward history and record the new entry.
            self.history.truncate(self.history_index + 1);
            self.history.push(target_path);
            self.history_index = self.history.len() - 1;
            true
        } else {
            false
        }
    }
}

impl eframe::App for MarkdownApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Pre-compute toolbar state before any closures borrow self.
        let can_go_back = self.can_go_back();
        let can_go_forward = self.can_go_forward();
        let back_tip = self
            .history_index
            .checked_sub(1)
            .and_then(|i| self.history.get(i))
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let forward_tip = self
            .history
            .get(self.history_index + 1)
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let current_path_label = self.current_file_path.display().to_string();

        let mut nav_action = NavAction::None;

        egui::Panel::top("nav_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(can_go_back, egui::Button::new("◀ Back"))
                    .on_hover_text(&back_tip)
                    .clicked()
                {
                    nav_action = NavAction::Back;
                }
                if ui
                    .add_enabled(can_go_forward, egui::Button::new("Forward ▶"))
                    .on_hover_text(&forward_tip)
                    .clicked()
                {
                    nav_action = NavAction::Forward;
                }
                ui.separator();
                ui.label(&current_path_label);
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
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

        // Execute whichever navigation was requested in this frame.
        let navigated = if let Some(url) = clicked_url {
            if url.starts_with("http://") || url.starts_with("https://") {
                // Re-queue external links for the platform to open in the browser.
                ui.ctx().open_url(egui::output::OpenUrl::new_tab(url));
                false
            } else {
                self.handle_link_click(&url)
            }
        } else {
            match nav_action {
                NavAction::Back => self.go_back(),
                NavAction::Forward => self.go_forward(),
                NavAction::None => false,
            }
        };

        if navigated {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Title(format!(
                    "Viewing: {}",
                    self.current_file_path.display()
                )));
        }
    }
}
