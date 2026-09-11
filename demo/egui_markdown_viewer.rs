/*[toml]
[dependencies]
thag_proc_macros = { version = "1, thag-auto" }
thag_styling = { version = "1, thag-auto", features = ["inquire_theming"] }
# egui_extras = { version = "0.35", features = ["svg_text"] } # Slow

[features]
default = ["eframe/wgpu", "egui_commonmark/better_syntax_highlighting","egui_commonmark/svg","egui_commonmark/fetch"]

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

use std::path::{Path, PathBuf};
use thag_styling::{
    auto_help, file_navigator, help_system::check_help_and_exit, themed_inquire_config,
};

file_navigator! {}

/// Help text rendered in the F1 help window.
const HELP_TEXT: &str = "\
# Markdown Viewer — Help

## Keyboard Shortcuts

### File
| Key | Action |
|---|---|
| Cmd/Ctrl-o | Open a markdown file |
| Cmd/Ctrl-r | Refresh — reload the current file from disk |
| Cmd/Ctrl-w  /  Cmd/Ctrl-q | Quit |

### Navigation
| Key | Action |
|---|---|
| ◀ / ▶ buttons | Back / Forward in history |
| Cmd/Ctrl-t | Toggle the table of contents panel |

### Search
| Key | Action |
|---|---|
| Cmd/Ctrl-f | Open / close the search bar |
| Enter  or  ⬇ button | Next match |
| Shift-Enter  or  ⬆ button | Previous match |
| Escape | Close the search bar |

> **Note:** Search navigates to the section containing each match (section-level navigation).
> Inline text highlighting is planned for a future version.

### Zoom & Font
| Key | Action |
|---|---|
| Cmd/Ctrl-= | Zoom in |
| Cmd/Ctrl-− | Zoom out |
| Cmd/Ctrl-z | Reset zoom to 100% |
| Cmd/Ctrl-Shift-a | Enlarge font |
| Cmd/Ctrl-a | Reduce font |
| Cmd/Ctrl-0 | Reset font to 100% |

### Help
| Key | Action |
|---|---|
| F1 | Toggle this help screen |
";

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
            v.code_bg_color = egui::Color32::from_gray(100);
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
            v.panel_fill = egui::Color32::from_rgb(255, 255, 255);
            v.window_fill = egui::Color32::from_rgb(255, 255, 255);
            v.code_bg_color = egui::Color32::from_rgb(225, 225, 230);
            v.hyperlink_color = egui::Color32::from_rgb(0, 100, 210);
        }
        v.image_loading_spinners = false; // always off in a document reader
        v
    });

    // Ubuntu Mono renders visually larger than Ubuntu at equal point sizes (wider
    // per-character advance, larger x-height).  Nudge it down to 12.0 px so that
    // inline code and fenced code blocks feel balanced against 14 px body text.
    // In egui 0.35 font styles are stored per-theme, so set both.
    for theme in [egui::Theme::Dark, egui::Theme::Light] {
        ctx.style_mut_of(theme, |style| {
            use egui::{FontFamily, FontId, TextStyle};
            style.text_styles.insert(
                TextStyle::Monospace,
                FontId::new(12.0, FontFamily::Monospace),
            );
        });
    }
}

// ─── TOC / heading extraction ──────────────────────────────────────────────────

/// An entry in the table of contents, derived from one ATX heading in the document.
#[derive(Clone)]
struct TocEntry {
    /// Heading depth 1–6.
    level: u8,
    /// Display text (raw heading text; may include inline markup such as `**bold**`).
    text: String,
    /// The `{#slug}` injected into the rendered content, used as the scroll target.
    slug: String,
    /// Byte offset of the heading line in the *raw* file content (for search section lookup).
    byte_start: usize,
}

/// Converts heading text to a URL-safe slug: lowercased, non-alphanumeric runs replaced by `-`.
fn slugify(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    let mut prev_sep = true; // start true to drop any leading hyphens
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_sep = false;
        } else if !prev_sep {
            slug.push('-');
            prev_sep = true;
        }
    }
    if slug.ends_with('-') {
        slug.pop();
    }
    slug
}

/// Parses an ATX heading line and returns `(level, plain_text)`.
/// `plain_text` is the heading content with any trailing `{…}` attribute block stripped.
/// Returns `None` for non-heading lines, indented lines, or malformed ATX syntax.
fn parse_heading_line(line: &str) -> Option<(u8, &str)> {
    let hashes = line.bytes().take_while(|&b| b == b'#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    // ATX heading must have a space after the `#` run.
    let rest = line[hashes..].strip_prefix(' ')?;
    let text = rest.trim_end();
    if text.is_empty() {
        return None;
    }
    // Strip any trailing `{#id}` / `{.class}` attribute block.
    let plain = if let Some(brace) = text.rfind('{') {
        let attr = text[brace..].trim_end();
        if attr.ends_with('}') {
            text[..brace].trim_end()
        } else {
            text
        }
    } else {
        text
    };
    Some((
        u8::try_from(hashes).expect("Unexpected character in markdown heading line"),
        plain,
    ))
}

/// Returns the explicit `{#id}` from a heading line, if present.
fn extract_heading_id(line: &str) -> Option<&str> {
    let brace = line.rfind('{')?;
    let attr = line[brace..].trim_end();
    if attr.starts_with("{#") && attr.ends_with('}') {
        Some(&attr[2..attr.len() - 1])
    } else {
        None
    }
}

/// Scans `raw` markdown, builds a `Vec<TocEntry>` from ATX headings, and returns a version
/// of the content with `{#slug}` attributes injected into every heading that lacks one.
/// `byte_start` values in each `TocEntry` are byte offsets into `raw`.
fn extract_toc_and_inject_ids(raw: &str) -> (String, Vec<TocEntry>) {
    let mut out = String::with_capacity(raw.len() + 512);
    let mut toc = Vec::new();
    let mut slug_counts: HashMap<String, usize> = HashMap::new();
    let mut in_fence = false;
    let mut fence_char = b'`';
    let mut byte_pos: usize = 0;

    for line in raw.lines() {
        let line_byte_start = byte_pos;
        byte_pos += line.len() + 1; // +1 approximates the \n

        let trimmed = line.trim_start_matches(' ');

        // Track fenced code blocks (``` or ~~~, optionally indented up to 3 spaces).
        let is_fence_candidate = trimmed.starts_with("```") || trimmed.starts_with("~~~");
        if is_fence_candidate && line.len() - trimmed.len() <= 3 {
            let ch = trimmed.as_bytes()[0];
            if !in_fence {
                in_fence = true;
                fence_char = ch;
            } else if ch == fence_char {
                in_fence = false;
            }
            out.push_str(line);
            out.push('\n');
            continue;
        }

        if !in_fence {
            if let Some((level, plain_text)) = parse_heading_line(line) {
                let (slug, line_out) = if let Some(id) = extract_heading_id(line) {
                    // Preserve the existing explicit ID.
                    (id.to_string(), line.to_string())
                } else {
                    // Auto-generate a deduplicated slug.
                    let base = slugify(plain_text);
                    let count = slug_counts.entry(base.clone()).or_insert(0);
                    let slug = if *count == 0 {
                        base.clone()
                    } else {
                        format!("{base}-{count}")
                    };
                    *count += 1;
                    let injected = format!("{} {{#{slug}}}", line.trim_end());
                    (slug, injected)
                };

                toc.push(TocEntry {
                    level,
                    text: plain_text.to_string(),
                    slug,
                    byte_start: line_byte_start,
                });
                out.push_str(&line_out);
                out.push('\n');
                continue;
            }
        }

        out.push_str(line);
        out.push('\n');
    }

    (out, toc)
}

// ─── Image path absolutization ─────────────────────────────────────────────────

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
            .map_or_else(|| (inner.trim(), ""), |i| (&inner[..i], &inner[i..]));

        let is_schemed = raw_path.starts_with("http://")
            || raw_path.starts_with("https://")
            || raw_path.starts_with("file://")
            || raw_path.starts_with("data:");

        out.push_str(prefix);
        if is_schemed {
            out.push_str(inner);
        } else if let Ok(abs) = base_dir.join(raw_path).canonicalize() {
            out.push_str(&path_to_file_uri(&abs));
            out.push_str(title_suffix);
        } else {
            // File not found — leave unchanged so the viewer shows a
            // broken-image placeholder rather than silently doing nothing.
            out.push_str(inner);
        }
        out.push(')');
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
    // Keep the process CWD in sync with the file so egui_extras resolves
    // relative image URIs correctly from the start.
    if let Some(parent) = canonical_initial_path.parent() {
        let _ = std::env::set_current_dir(parent);
    }

    let markdown_content = std::fs::read_to_string(&canonical_initial_path).unwrap_or_else(|_| {
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

    /// Load `path` from disk and update content, TOC, `raw_content`, cache, and CWD.
    /// Returns `true` on success.
    fn load_file(&mut self, path: PathBuf) -> bool {
        match std::fs::read_to_string(&path) {
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
