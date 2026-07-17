/*[toml]
[dependencies]
egui_commonmark_extended = "0.25"
egui_extras = { version = "0.35", features = ["svg_text"] } # Slow
thag_proc_macros = { version = "1, thag-auto" }
thag_styling = { version = "1, thag-auto", features = ["inquire_theming"] }

[patch.crates-io]
egui_commonmark_extended = { git = "https://github.com/aydiler/md-viewer", branch = "main" }

[features]
default = ["eframe/wgpu", "egui_commonmark_extended/better_syntax_highlighting","egui_commonmark_extended/svg","egui_commonmark_extended/fetch"]

# Make sure the result runs fast
[profile.dev]
opt-level = 3     # Apply maximum performance optimizations
*/
/// A fast little GUI markdown viewer using `inquire` to select a markdown file and `egui_commonmark` with
/// `eframe`'s WGPU feature to render it. Relative links are resolved relative to the parent directory of the
/// current markdown file, so navigation between linked documents works correctly. Supports back/forward history
/// and light/dark/system theme switching via `egui_theme_switch`.
/// Note: `[![alt](img)](url)` image links are a known `egui_commonmark` limitation — the link wrapping
/// an image produces an invisible zero-size hyperlink. If you want a clickable link alongside an image,
/// add an explicit text link in the markdown below it. You may also notice that it does not handle banners
/// well.
/// See the `md-viewer` crate for a professional quality installable example using `egui_commonmark`
/// vendored to address some issues.
//# Purpose: Prototype a markdown viewer using the `egui_commonmark` crate.
//# Categories: crates, demo, gui, prototype, tools
//# Usage: egui_markdown_viewer [OPTIONS] [path_to_file]
use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use thag_styling::{
    auto_help, file_navigator, help_system::check_help_and_exit, themed_inquire_config,
};

file_navigator! {}

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

    let markdown_content = fs::read_to_string(&canonical_initial_path).unwrap_or_else(|_| {
        format!(
            "# Error\nFailed to read `{}`.",
            canonical_initial_path.display()
        )
    });

    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_title(format!(
                "egui_markdown_viewer: {}",
                canonical_initial_path.display()
            )),
        ..Default::default()
    };

    eframe::run_native(
        "Markdown Viewer",
        options,
        Box::new(move |cc| {
            setup_fonts(&cc.egui_ctx);
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
        match fs::read_to_string(&path) {
            Ok(new_content) => {
                // Keep CWD in sync so future canonicalize() calls and image loading work correctly.
                if let Some(dir) = path.parent() {
                    let _ = std::env::set_current_dir(dir);
                }
                self.content = new_content;
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
                ui.label("Theme");
                egui_theme_switch::global_theme_switch(ui);
                ui.separator();
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
                    "egui_markdown_viewer: {}",
                    self.current_file_path.display()
                )));
        }
    }
}

// Copied from aydiler/md-viewer under MIT licence.
fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let mut loaded_fonts: HashSet<String> = HashSet::new();

    // Try to load each font from its possible paths
    for (font_name, font_path) in SYSTEM_FONT_PATHS {
        // Skip if we already loaded this font
        if loaded_fonts.contains(*font_name) {
            continue;
        }

        let path = Path::new(font_path);
        if path.exists() {
            match fs::read(path) {
                Ok(font_data) => {
                    log::info!("Loaded font fallback: {} from {}", font_name, font_path);

                    fonts.font_data.insert(
                        font_name.to_string(),
                        egui::FontData::from_owned(font_data).into(),
                    );

                    // Noto Sans is the primary body face so regular text shares the
                    // Noto Sans family — and its baseline/ascent metrics — with the
                    // Noto Sans Bold used for `**strong**` (issue #39). Otherwise
                    // regular text stays on egui's bundled Ubuntu-Light and bold spans
                    // sit on a slightly different baseline. Other scripts (CJK, Arabic,
                    // …) remain appended fallbacks.
                    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                        if *font_name == "NotoSans" {
                            family.insert(0, font_name.to_string());
                        } else {
                            family.push(font_name.to_string());
                        }
                    }

                    // Also add text fonts to monospace for code blocks with Unicode
                    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                        family.push(font_name.to_string());
                    }

                    loaded_fonts.insert(font_name.to_string());
                }
                Err(e) => {
                    log::debug!("Failed to read font {}: {}", font_path, e);
                }
            }
        }
    }

    if loaded_fonts.is_empty() {
        log::warn!("No system fonts loaded. Unicode characters may show as red triangles.");
        log::warn!("Install noto-fonts and noto-fonts-cjk for full Unicode support.");
    } else {
        log::info!(
            "Loaded {} font fallbacks for Unicode support",
            loaded_fonts.len()
        );
    }

    // setup_strong_font_family(&mut fonts);

    ctx.set_fonts(fonts);
}

// Copied from aydiler/md-viewer under MIT licence.
// /// Register a named bold font family for Markdown strong text.
// fn setup_strong_font_family(fonts: &mut egui::FontDefinitions) {
//     let mut strong_family = Vec::new();

//     // Try to load a true bold face before appending proportional fallbacks.
//     for font_path in STRONG_FONT_PATHS {
//         let path = Path::new(font_path);
//         if path.exists() {
//             match fs::read(path) {
//                 Ok(font_data) => {
//                     log::info!("Loaded Markdown strong font: {}", font_path);
//                     fonts.font_data.insert(
//                         STRONG_FONT_FAMILY.to_string(),
//                         egui::FontData::from_owned(font_data).into(),
//                     );
//                     strong_family.push(STRONG_FONT_FAMILY.to_string());
//                     break;
//                 }
//                 Err(e) => {
//                     log::debug!("Failed to read Markdown strong font {}: {}", font_path, e);
//                 }
//             }
//         }
//     }

//     // Keep normal proportional fallbacks so bold text can still render broad Unicode.
//     if let Some(proportional) = fonts.families.get(&egui::FontFamily::Proportional) {
//         strong_family.extend(
//             proportional
//                 .iter()
//                 .filter(|font_name| font_name.as_str() != STRONG_FONT_FAMILY)
//                 .cloned(),
//         );
//     }

//     if !matches!(
//         strong_family.first(),
//         Some(font_name) if font_name.as_str() == STRONG_FONT_FAMILY
//     ) {
//         log::warn!("No Markdown strong font found. Install NotoSans-Bold for true bold rendering.");
//     }

//     // The renderer selects this named family for strong spans; register it even
//     // without a bold face so documents do not panic on systems missing Noto Sans Bold.
//     fonts.families.insert(
//         egui::FontFamily::Name(STRONG_FONT_FAMILY.into()),
//         strong_family,
//     );
// }

/// System font paths for fallback (Linux and Windows common paths)
const SYSTEM_FONT_PATHS: &[(&str, &str)] = &[
    // Noto Sans for extended Latin, Greek, Cyrillic
    ("NotoSans", "/usr/share/fonts/noto/NotoSans-Regular.ttf"),
    ("NotoSans", "/usr/share/fonts/TTF/NotoSans-Regular.ttf"),
    (
        "NotoSans",
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
    ),
    // Linux CJK fonts (Chinese, Japanese, Korean)
    (
        "NotoSansCJK",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    ),
    (
        "NotoSansCJK",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    ),
    (
        "NotoSansCJK",
        "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
    ),
    // Windows CJK fonts (Chinese, Japanese, Korean)
    ("MicrosoftYaHei", "C:/Windows/Fonts/msyh.ttc"),
    ("MicrosoftYaHeiUI", "C:/Windows/Fonts/msyh.ttc"),
    ("SimSun", "C:/Windows/Fonts/simsun.ttc"),
    ("NSimSun", "C:/Windows/Fonts/simsun.ttc"),
    ("DengXian", "C:/Windows/Fonts/Deng.ttf"),
    ("MicrosoftJhengHei", "C:/Windows/Fonts/msjh.ttc"),
    ("YuGothic", "C:/Windows/Fonts/YuGothM.ttc"),
    ("MalgunGothic", "C:/Windows/Fonts/malgun.ttf"),
    // Arabic
    (
        "NotoSansArabic",
        "/usr/share/fonts/noto/NotoSansArabic-Regular.ttf",
    ),
    (
        "NotoSansArabic",
        "/usr/share/fonts/TTF/NotoSansArabic-Regular.ttf",
    ),
    // Hebrew
    (
        "NotoSansHebrew",
        "/usr/share/fonts/noto/NotoSansHebrew-Regular.ttf",
    ),
    (
        "NotoSansHebrew",
        "/usr/share/fonts/TTF/NotoSansHebrew-Regular.ttf",
    ),
    // Devanagari (Hindi, Sanskrit)
    (
        "NotoSansDevanagari",
        "/usr/share/fonts/noto/NotoSansDevanagari-Regular.ttf",
    ),
    (
        "NotoSansDevanagari",
        "/usr/share/fonts/TTF/NotoSansDevanagari-Regular.ttf",
    ),
    // Thai
    (
        "NotoSansThai",
        "/usr/share/fonts/noto/NotoSansThai-Regular.ttf",
    ),
    (
        "NotoSansThai",
        "/usr/share/fonts/TTF/NotoSansThai-Regular.ttf",
    ),
    // Symbols (math, arrows, etc.)
    (
        "NotoSansSymbols",
        "/usr/share/fonts/noto/NotoSansSymbols-Regular.ttf",
    ),
    (
        "NotoSansSymbols",
        "/usr/share/fonts/TTF/NotoSansSymbols-Regular.ttf",
    ),
    (
        "NotoSansSymbols2",
        "/usr/share/fonts/noto/NotoSansSymbols2-Regular.ttf",
    ),
    (
        "NotoSansSymbols2",
        "/usr/share/fonts/TTF/NotoSansSymbols2-Regular.ttf",
    ),
    // DejaVu Sans - covers warning sign (U+26A0) and other misc symbols
    ("DejaVuSans", "/usr/share/fonts/TTF/DejaVuSans.ttf"),
    ("DejaVuSans", "/usr/share/fonts/dejavu/DejaVuSans.ttf"),
    (
        "DejaVuSans",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ),
];
