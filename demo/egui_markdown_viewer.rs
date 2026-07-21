/*[toml]
[dependencies]
egui_commonmark = { git = "https://github.com/durbanlegend/egui_commonmark", features = ["better_syntax_highlighting", "svg", "fetch"] }

egui_extras = { version = "0.35", features = ["svg"] }
thag_proc_macros = { version = "1, thag-auto" }
thag_styling = { version = "1, thag-auto", features = ["inquire_theming"] }
resvg = { version = "0.45", features = ["text"] }
fontdb = { version = "0.23", features = ["fs"] }
rfd = { version = "0.15" }

[features]
default = ["eframe/wgpu", "egui_commonmark/better_syntax_highlighting","egui_commonmark/svg","egui_commonmark/fetch"]

# Make sure the result runs fast
[profile.dev]
opt-level = 3     # Apply maximum performance optimizations
*/
/// A fast little GUI markdown viewer using `inquire` to select a markdown file and `egui_commonmark` with
/// `eframe`'s WGPU feature to render it. Relative links are resolved relative to the parent directory of the
/// current markdown file, so navigation between linked documents works correctly. Supports back/forward history,
/// light/dark/system theme switching via `egui_theme_switch`, zoom (Cmd-= / Cmd-- / Cmd-0), and opening a new
/// file without quitting via a native file dialog (Cmd-O / "Open…" button).
/// Improved readability over the egui defaults: near-black text in light mode, near-white
/// in dark mode, warm paper background, higher-contrast code block backgrounds, and GitHub-style syntax
/// highlighting for code blocks.
/// Note: `[![alt](img)](url)` image links are a known `egui_commonmark` limitation — the link wrapping
/// an image produces an invisible zero-size hyperlink. If you want a clickable link alongside an image,
/// add an explicit text link in the markdown below it. You may also notice that it does not handle banners
/// well.
/// See the `md-viewer` crate for a professional quality installable example using `egui_commonmark`
/// vendored to address some issues.
/// The MSRV of this program is 1.92.
//# Purpose: Prototype a markdown viewer using the `egui_commonmark` crate.
//# Categories: crates, demo, gui, prototype, tools
//# Usage: egui_markdown_viewer [OPTIONS] [path_to_file]
use eframe::egui;
use egui::load::{BytesPoll, ImageLoadResult, ImageLoader, ImagePoll, LoadError, SizeHint};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use rfd::FileDialog;
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use thag_styling::{
    auto_help, file_navigator, help_system::check_help_and_exit, themed_inquire_config,
};

file_navigator! {}

/// Applies contrast colours to both egui themes; font sizes are always left at
/// egui defaults so toggling never causes a scroll-position jump.
///
/// `enhanced = true`  — high-contrast colours (near-white/near-black text, warm backgrounds).
/// `enhanced = false` — stock egui colours.
///
/// Called once at startup and again whenever the toolbar "Contrast+/-" toggle changes.
/// `image_loading_spinners` is kept `false` in both modes.
fn apply_style(ctx: &egui::Context, enhanced: bool) {
    // ── Dark mode ─────────────────────────────────────────────────────────────────────────
    ctx.set_visuals_of(egui::Theme::Dark, {
        let mut v = egui::Visuals::dark();
        if enhanced {
            v.widgets.noninteractive.fg_stroke.color = egui::Color32::from_gray(240);
            v.panel_fill = egui::Color32::from_gray(32);
            v.window_fill = egui::Color32::from_gray(32);
            v.code_bg_color = egui::Color32::from_gray(55);
            v.hyperlink_color = egui::Color32::from_rgb(100, 185, 255);
        }
        v.image_loading_spinners = false; // always off in a document reader
        v
    });

    // ── Light mode ────────────────────────────────────────────────────────────────────────
    ctx.set_visuals_of(egui::Theme::Light, {
        let mut v = egui::Visuals::light();
        if enhanced {
            v.widgets.noninteractive.fg_stroke.color = egui::Color32::from_gray(5);
            v.panel_fill = egui::Color32::from_rgb(252, 252, 248);
            v.window_fill = egui::Color32::from_rgb(252, 252, 248);
            v.code_bg_color = egui::Color32::from_rgb(225, 225, 230);
            v.hyperlink_color = egui::Color32::from_rgb(0, 100, 210);
        }
        v.image_loading_spinners = false; // always off in a document reader
        v
    });
}

/// Rewrites relative image paths in Markdown to absolute `file://` URIs so they
/// load correctly regardless of platform CWD behaviour.
///
/// Paths that already carry a URI scheme (`http://`, `file://`, `data:`, …) are
/// left untouched. If a relative path cannot be resolved (file does not exist)
/// it is also left untouched so existing error behaviour is preserved.
///
/// Note: processes the raw text, so a path inside a fenced code block is also
/// rewritten if it matches the image syntax — an acceptable trade-off for the
/// cross-platform fix.
fn absolutize_image_paths(content: &str, base_dir: &Path) -> String {
    let mut out = String::with_capacity(content.len() + 128);
    let mut rest = content;

    while let Some(bang) = rest.find("![") {
        out.push_str(&rest[..bang]);
        rest = &rest[bang..];

        // Find `](`  — alt text must not contain `]`
        let Some(close_bracket) = rest.find("](") else {
            out.push_str(&rest[..2]);
            rest = &rest[2..];
            continue;
        };

        let prefix = &rest[..close_bracket + 2]; // `![alt](`
        rest = &rest[close_bracket + 2..];

        let Some(close_paren) = rest.find(')') else {
            out.push_str(prefix);
            continue;
        };

        let inner = &rest[..close_paren]; // path, possibly with `"title"`
        rest = &rest[close_paren + 1..];

        // Split optional title: `path "title"` or `path 'title'`
        let (raw_path, title_suffix) = inner
            .find(" \"")
            .or_else(|| inner.find(" '"))
            .map_or((inner.trim(), ""), |i| (&inner[..i], &inner[i..]));

        let is_schemed = raw_path.starts_with("http://")
            || raw_path.starts_with("https://")
            || raw_path.starts_with("file://")
            || raw_path.starts_with("data:");

        if is_schemed {
            out.push_str(prefix);
            out.push_str(inner);
            out.push(')');
        } else {
            match base_dir.join(raw_path).canonicalize() {
                Ok(abs) => {
                    out.push_str(prefix);
                    out.push_str(&path_to_file_uri(&abs));
                    out.push_str(title_suffix);
                    out.push(')');
                }
                Err(_) => {
                    // File not found — leave unchanged so the viewer shows a
                    // broken-image placeholder rather than silently doing nothing.
                    out.push_str(prefix);
                    out.push_str(inner);
                    out.push(')');
                }
            }
        }
    }
    out.push_str(rest);
    out
}

/// Converts an absolute `Path` to a `file://` URI that is valid on all platforms.
/// Windows paths (`C:\…`) become `file:///C:/…`; Unix paths become `file:///…`.
fn path_to_file_uri(path: &Path) -> String {
    let s = path.to_string_lossy().into_owned();
    #[cfg(windows)]
    {
        s = s.replace('\\', "/");
    }
    // Unix absolute paths start with `/`; Windows paths start with the drive letter.
    if s.starts_with('/') {
        format!("file://{s}") // file:// + /unix/path = file:///unix/path
    } else {
        format!("file:///{s}") // file:/// + C:/... = file:///C:/...
    }
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
    let initial_base_dir = canonical_initial_path
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();
    // Keep CWD in sync for canonicalize() calls inside the viewer.
    let _ = env::set_current_dir(&initial_base_dir);

    let raw_content = std::fs::read_to_string(&canonical_initial_path).unwrap_or_else(|_| {
        format!(
            "# Error\nFailed to read `{}`.",
            canonical_initial_path.display()
        )
    });
    // Rewrite relative image paths to absolute file:// URIs so they resolve
    // correctly on all platforms (CWD-based resolution fails on Windows).
    let markdown_content = absolutize_image_paths(&raw_content, &initial_base_dir);

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
            // Register our fast SVG loader BEFORE the first frame triggers
            // egui_commonmark's `prepare_show`, which calls `install_image_loaders`.
            // Since `install_image_loaders` skips any loader whose ID is already
            // registered, the default `SvgLoader::default()` (which calls the
            // 20-second `load_system_fonts()`) is never constructed.
            cc.egui_ctx.add_image_loader(Arc::new(FastSvgLoader::new()));
            apply_style(&cc.egui_ctx, true);
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
    /// Required by `egui_commonmark` for rendering images/styles.
    cache: CommonMarkCache,
    /// Ordered list of visited file paths.
    history: Vec<PathBuf>,
    /// Current position within `history`.
    history_index: usize,
    /// Multiplicative scale applied to content text only (toolbar stays at 1×).
    content_zoom: f32,
    /// Whether high-contrast colours are active (`true`) or stock egui colours (`false`).
    enhanced_contrast: bool,
}

impl MarkdownApp {
    fn new(content: String, path: PathBuf) -> Self {
        Self {
            content,
            current_file_path: path.clone(),
            cache: CommonMarkCache::default(),
            history: vec![path],
            history_index: 0,
            content_zoom: 1.0,
            enhanced_contrast: true,
        }
    }

    const fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    const fn can_go_forward(&self) -> bool {
        self.history_index + 1 < self.history.len()
    }

    /// Load `path` from disk and update content, current path, cache, and CWD.
    /// Returns `true` on success.
    fn load_file(&mut self, path: PathBuf) -> bool {
        match std::fs::read_to_string(&path) {
            Ok(raw_content) => {
                let base_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
                // Keep CWD in sync for future canonicalize() calls.
                let _ = env::set_current_dir(&base_dir);
                // Rewrite relative image paths to absolute file:// URIs.
                self.content = absolutize_image_paths(&raw_content, &base_dir);
                self.current_file_path = path;
                // Clear the cache so egui_commonmark doesn't carry over stale state.
                self.cache = CommonMarkCache::default();
                true
            }
            Err(e) => {
                eprintln!("Failed to read {path:?}: {e}");
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
        let current_dir = self
            .current_file_path
            .parent()
            .map_or_else(|| PathBuf::from("."), |parent| parent.to_path_buf());
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
    #[allow(clippy::too_many_lines)]
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
        // Snapshot for use inside closures (closures can't borrow self).
        let content_zoom = self.content_zoom;

        let mut nav_action = NavAction::None;
        let mut open_file_requested = false;
        // Zoom is stored in self.content_zoom and applied only to the content
        // text styles, so the toolbar always stays at its natural size.
        let mut new_content_zoom = self.content_zoom;
        let zoom_label = format!("{:.0}%", self.content_zoom * 100.0);
        let enhanced_contrast = self.enhanced_contrast;
        let mut new_enhanced_contrast = enhanced_contrast;

        // ── Global keyboard shortcuts ─────────────────────────────────────────────────────
        // Collect key states first, then act — acting inside input() re-locks
        // the context and causes a deadlock.
        let (close, open_key, zoom_in_key, zoom_out_key) = ui.ctx().input(|i| {
            (
                i.modifiers.command && (i.key_pressed(egui::Key::W) || i.key_pressed(egui::Key::Q)),
                i.modifiers.command && i.key_pressed(egui::Key::O),
                i.modifiers.command && i.key_pressed(egui::Key::Equals),
                i.modifiers.command && i.key_pressed(egui::Key::Minus),
            )
        });
        if close {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if open_key {
            open_file_requested = true;
        }
        if zoom_in_key {
            new_content_zoom = (new_content_zoom * 1.1).min(3.0);
        }
        if zoom_out_key {
            new_content_zoom = (new_content_zoom / 1.1).max(0.4);
        }

        egui::Panel::top("nav_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Theme");
                egui_theme_switch::global_theme_switch(ui);
                if ui
                    .selectable_label(
                        enhanced_contrast,
                        if enhanced_contrast {
                            "Contrast-"
                        } else {
                            "Contrast+"
                        },
                    )
                    .on_hover_text(if enhanced_contrast {
                        "Restore default egui contrast"
                    } else {
                        "Apply high-contrast colours"
                    })
                    .clicked()
                {
                    new_enhanced_contrast = !enhanced_contrast;
                }
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
                if ui
                    .button("Open…")
                    .on_hover_text("Open a markdown file (Cmd-O)")
                    .clicked()
                {
                    open_file_requested = true;
                }
                ui.separator();
                // Reserve space for the right-hand controls before sizing the
                // path label, so the label never spills into the zoom widgets.
                // Estimate: − 20 + label 35 + + 20 + ↺ 20 + sep 8 + Quit 40
                //           + inter-widget spacing ≈ 200 px at toolbar font size.
                let right_reserve = 200.0_f32;
                let label_max = (ui.available_width() - right_reserve).max(40.0);
                ui.scope(|ui| {
                    ui.set_max_width(label_max);
                    ui.add(egui::Label::new(&current_path_label).truncate())
                        .on_hover_text(&current_path_label);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button("Quit")
                        .on_hover_text("Close (Cmd-W / Ctrl-W)")
                        .clicked()
                    {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    ui.separator();
                    // Zoom controls (content text only; toolbar stays at 1×).
                    // RTL layout: first-added = rightmost, so render ↺, +, label, −
                    // to produce the visual sequence  − | 100% | + | ↺  left-to-right.
                    if ui
                        .small_button("↺")
                        .on_hover_text("Reset content zoom to 100%")
                        .clicked()
                    {
                        new_content_zoom = 1.0;
                    }
                    if ui
                        .small_button("+")
                        .on_hover_text("Zoom content in (Cmd-=)")
                        .clicked()
                    {
                        new_content_zoom = (new_content_zoom * 1.1).min(3.0);
                    }
                    ui.label(&zoom_label);
                    if ui
                        .small_button("−")
                        .on_hover_text("Zoom content out (Cmd-−)")
                        .clicked()
                    {
                        new_content_zoom = (new_content_zoom / 1.1).max(0.4);
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            // Floating scrollbar that reserves its own layout space so the
            // code-block copy icon is never obscured by the bar on hover.
            {
                let scroll = &mut ui.style_mut().spacing.scroll;
                scroll.floating = true;
                scroll.content_margin = egui::Margin::same(10); // Adds padding between content & scrollbar
                scroll.bar_width = 10.0; // default ~8-10 px;
                scroll.dormant_handle_opacity = 0.15; // thumb nearly invisible at rest
                scroll.interact_handle_opacity = 0.55; // visible but soft on hover
                scroll.active_handle_opacity = 0.80; // clear while dragging
            }
            // Give each file path its own scroll-state key so that:
            //   • navigating to a new file always starts at the top, and
            //   • back/forward navigation restores the previous scroll position.
            egui::ScrollArea::vertical()
                .id_salt(&current_path_label)
                .show(ui, |ui| {
                    // Scale content text styles locally so the toolbar is unaffected.
                    // Read the current base sizes from the inherited style so zoom
                    let cz = content_zoom;
                    if (cz - 1.0).abs() > 0.005 {
                        use egui::{FontFamily, FontId, TextStyle};
                        let s = ui.style_mut();
                        let base_body =
                            s.text_styles.get(&TextStyle::Body).map_or(14.0, |f| f.size);
                        let base_mono = s
                            .text_styles
                            .get(&TextStyle::Monospace)
                            .map_or(14.0, |f| f.size);
                        let base_heading = s
                            .text_styles
                            .get(&TextStyle::Heading)
                            .map_or(21.0, |f| f.size);
                        let base_small = s
                            .text_styles
                            .get(&TextStyle::Small)
                            .map_or(10.0, |f| f.size);
                        s.text_styles.insert(
                            TextStyle::Body,
                            FontId::new(base_body * cz, FontFamily::Proportional),
                        );
                        s.text_styles.insert(
                            TextStyle::Monospace,
                            FontId::new(base_mono * cz, FontFamily::Monospace),
                        );
                        s.text_styles.insert(
                            TextStyle::Heading,
                            FontId::new(base_heading * cz, FontFamily::Proportional),
                        );
                        s.text_styles.insert(
                            TextStyle::Small,
                            FontId::new(base_small * cz, FontFamily::Proportional),
                        );
                    }
                    CommonMarkViewer::new()
                        .syntax_theme_dark("base16-ocean.dark")
                        .syntax_theme_light("InspiredGitHub")
                        .enable_scroll_to_heading(true)
                        .show(ui, &mut self.cache, &self.content);
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
        } else if open_file_requested {
            // Show a native file-open dialog. This call blocks the egui render loop
            // until the user picks a file or cancels; that is expected behaviour.
            let start_dir = self
                .current_file_path
                .parent()
                .map_or_else(|| PathBuf::from("."), |p| p.to_path_buf());
            if let Some(path) = FileDialog::new()
                .add_filter("Markdown", &["md", "markdown"])
                .set_directory(&start_dir)
                .pick_file()
            {
                let canonical = path.canonicalize().unwrap_or(path);
                if self.load_file(canonical.clone()) {
                    self.history.truncate(self.history_index + 1);
                    self.history.push(canonical);
                    self.history_index = self.history.len() - 1;
                    true
                } else {
                    false
                }
            } else {
                false // user cancelled
            }
        } else {
            match nav_action {
                NavAction::Back => self.go_back(),
                NavAction::Forward => self.go_forward(),
                NavAction::None => false,
            }
        };

        // Commit zoom change (may have come from keyboard or toolbar buttons).
        self.content_zoom = new_content_zoom;

        // Apply style toggle if it changed this frame.
        if new_enhanced_contrast != enhanced_contrast {
            self.enhanced_contrast = new_enhanced_contrast;
            apply_style(ui.ctx(), new_enhanced_contrast);
        }

        if navigated {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Title(format!(
                    "egui_markdown_viewer: {}",
                    self.current_file_path.display()
                )));
        }
    }
}

// ─── Fast SVG loader ──────────────────────────────────────────────────────────
//
// `egui_extras::install_image_loaders` (called lazily by `egui_commonmark` on
// first render) only registers its built-in `SvgLoader` if no loader with that
// ID already exists:
//
//   if !ctx.is_loader_installed(SvgLoader::ID) { ctx.add_image_loader(...) }
//
// `SvgLoader::default()` calls `fontdb::Database::load_system_fonts()`, which
// on macOS scans `/System/Library/AssetsV2` — thousands of downloadable font
// assets — and takes ~20 seconds.
//
// By pre-registering `FastSvgLoader` with the **same ID**, we prevent that
// constructor from ever running.  Our loader calls `load_fonts_dir` on a small
// set of known directories, reading font files directly without the macOS
// CoreText/AssetsV2 scan, which is fast (<1 s).

struct FastSvgLoader {
    state: Mutex<FastSvgState>,
}

struct FastSvgState {
    pass_index: u64,
    cache: HashMap<String, HashMap<SizeHint, FastSvgEntry>>,
    options: resvg::usvg::Options<'static>,
}

struct FastSvgEntry {
    last_used: u64,
    result: Result<Arc<egui::ColorImage>, String>,
}

impl FastSvgLoader {
    /// Must match `egui::generate_loader_id!(SvgLoader)` as evaluated inside
    /// the `egui_extras::loaders::svg_loader` module:
    /// `concat!(module_path!(), "::", "SvgLoader")`
    const ID: &'static str = "egui_extras::loaders::svg_loader::SvgLoader";

    fn new() -> Self {
        let mut options = resvg::usvg::Options::default();

        // Populate fontdb from known directories instead of calling
        // `load_system_fonts()`.  On macOS this deliberately skips
        // `/System/Library/AssetsV2`, which is what causes the ~20 s delay.
        let db = options.fontdb_mut();

        #[cfg(target_os = "macos")]
        {
            db.load_fonts_dir("/System/Library/Fonts/");
            db.load_fonts_dir("/Library/Fonts/");
            if let Ok(home) = env::var("HOME") {
                db.load_fonts_dir(Path::new(&home).join("Library/Fonts"));
            }
        }
        #[cfg(target_os = "linux")]
        {
            db.load_fonts_dir("/usr/share/fonts/");
            db.load_fonts_dir("/usr/local/share/fonts/");
            if let Ok(home) = env::var("HOME") {
                db.load_fonts_dir(Path::new(&home).join(".fonts"));
                db.load_fonts_dir(Path::new(&home).join(".local/share/fonts"));
            }
        }
        #[cfg(target_os = "windows")]
        {
            db.load_fonts_dir("C:/Windows/Fonts/");
            if let Ok(profile) = env::var("USERPROFILE") {
                db.load_fonts_dir(
                    Path::new(&profile).join("AppData/Local/Microsoft/Windows/Fonts"),
                );
            }
        }

        log::info!(
            "FastSvgLoader: initialised ({} font faces)",
            options.fontdb.faces().count()
        );

        Self {
            state: Mutex::new(FastSvgState {
                pass_index: 0,
                cache: HashMap::default(),
                options,
            }),
        }
    }
}

impl ImageLoader for FastSvgLoader {
    fn id(&self) -> &str {
        Self::ID
    }

    fn load(&self, ctx: &egui::Context, uri: &str, size_hint: SizeHint) -> ImageLoadResult {
        if !Path::new(uri)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
        {
            return Err(LoadError::NotSupported);
        }

        let mut state = self.state.lock().unwrap();
        let FastSvgState {
            pass_index,
            cache,
            options,
        } = &mut *state;

        let bucket = cache.entry(uri.to_owned()).or_default();
        if let Some(entry) = bucket.get_mut(&size_hint) {
            entry.last_used = *pass_index;
            return match entry.result.clone() {
                Ok(image) => Ok(ImagePoll::Ready { image }),
                Err(err) => Err(LoadError::Loading(err)),
            };
        }

        match ctx.try_load_bytes(uri) {
            Ok(BytesPoll::Ready { bytes, .. }) => {
                let result =
                    egui_extras::image::load_svg_bytes_with_size(&bytes, size_hint, options)
                        .map(Arc::new);
                bucket.insert(
                    size_hint,
                    FastSvgEntry {
                        last_used: *pass_index,
                        result: result.clone(),
                    },
                );
                match result {
                    Ok(image) => Ok(ImagePoll::Ready { image }),
                    Err(err) => Err(LoadError::Loading(err)),
                }
            }
            Ok(BytesPoll::Pending { size }) => Ok(ImagePoll::Pending { size }),
            Err(err) => Err(err),
        }
    }

    fn forget(&self, uri: &str) {
        self.state.lock().unwrap().cache.retain(|key, _| key != uri);
    }

    fn forget_all(&self) {
        self.state.lock().unwrap().cache.clear();
    }

    fn byte_size(&self) -> usize {
        self.state
            .lock()
            .unwrap()
            .cache
            .values()
            .flat_map(|bucket| bucket.values())
            .map(|entry| match &entry.result {
                Ok(image) => image.pixels.len() * std::mem::size_of::<egui::Color32>(),
                Err(err) => err.len(),
            })
            .sum()
    }

    fn end_pass(&self, pass_index: u64) {
        let mut state = self.state.lock().unwrap();
        state.pass_index = pass_index;
        state.cache.retain(|_key, bucket| {
            if 2 <= bucket.len() {
                bucket.retain(|_, entry| pass_index <= entry.last_used + 1);
            }
            !bucket.is_empty()
        });
    }
}
