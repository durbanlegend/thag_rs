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
/// current markdown file, so navigation between linked documents works correctly.
/// Supports back/forward history, light/dark/system theme switching via `egui_theme_switch`, zoom,
/// font scaling, opening a new file (Cmd/Ctrl-O), a left-side table of contents panel (§ / Cmd/Ctrl-T),
/// text search with match counter and section navigation (Cmd/Ctrl-F), refresh from disk (Cmd/Ctrl-R),
/// and a help screen (F1). Improved readability over the egui defaults: near-black text in light mode,
/// near-white in dark mode, warm paper background, higher-contrast code block backgrounds, and
/// GitHub-style syntax highlighting for code blocks.
///
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

#[cfg(target_os = "macos")]
const MOD: &str = "Cmd";

#[cfg(not(target_os = "macos"))]
const MOD: &str = "Ctrl";

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
    Some((hashes as u8, plain))
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
    let initial_base_dir = canonical_initial_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    // Keep CWD in sync for canonicalize() calls inside the viewer.
    let _ = env::set_current_dir(&initial_base_dir);

    let raw_content = std::fs::read_to_string(&canonical_initial_path).unwrap_or_else(|_| {
        format!(
            "# Error\nFailed to read `{}`.",
            canonical_initial_path.display()
        )
    });
    // Extract TOC headings and inject {#slug} attributes, then absolutize image paths.
    let (id_injected, toc) = extract_toc_and_inject_ids(&raw_content);
    let markdown_content = absolutize_image_paths(&id_injected, &initial_base_dir);

    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
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
                raw_content,
                canonical_initial_path,
                toc,
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
    /// Processed markdown text currently loaded (image paths absolutized, heading IDs injected).
    content: String,
    /// Raw file content as read from disk, used for text search.
    raw_content: String,
    /// The canonicalized path of the file we are viewing (so we know its parent folder).
    current_file_path: PathBuf,
    /// Required by `egui_commonmark` for rendering images/styles.
    cache: CommonMarkCache,
    /// Ordered list of visited file paths.
    history: Vec<PathBuf>,
    /// Current position within `history`.
    history_index: usize,
    /// Multiplicative scale applied to content text only (toolbar stays at 1×).
    font_scale: f32,
    /// Whether high-contrast colours are active (`true`) or stock egui colours (`false`).
    enhanced_contrast: bool,
    /// Table of contents entries extracted from the current document.
    toc: Vec<TocEntry>,
    /// Whether the TOC side panel is visible.
    show_toc: bool,
    /// Current search query string.
    search_query: String,
    /// Whether the search bar is visible.
    search_open: bool,
    /// When `true`, the search text field will grab keyboard focus on the next frame.
    search_focus: bool,
    /// Byte positions in `raw_content` of all query matches.
    search_matches: Vec<usize>,
    /// Index of the active match within `search_matches`.
    search_active: usize,
    /// Whether the F1 help window is visible.
    show_help: bool,
    /// Separate cache for the help window's `CommonMarkViewer`.
    help_cache: CommonMarkCache,
}

impl MarkdownApp {
    fn new(content: String, raw_content: String, path: PathBuf, toc: Vec<TocEntry>) -> Self {
        Self {
            content,
            raw_content,
            current_file_path: path.clone(),
            cache: CommonMarkCache::default(),
            history: vec![path],
            history_index: 0,
            font_scale: 1.0,
            enhanced_contrast: true,
            toc,
            show_toc: true,
            search_query: String::new(),
            search_open: false,
            search_focus: false,
            search_matches: Vec::new(),
            search_active: 0,
            show_help: false,
            help_cache: CommonMarkCache::default(),
        }
    }

    const fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    const fn can_go_forward(&self) -> bool {
        self.history_index + 1 < self.history.len()
    }

    /// Load `path` from disk, update content, TOC, raw_content, cache, and CWD.
    /// Returns `true` on success.
    fn load_file(&mut self, path: PathBuf) -> bool {
        match std::fs::read_to_string(&path) {
            Ok(raw) => {
                let base_dir = path
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .to_path_buf();
                // Keep CWD in sync for future canonicalize() calls.
                let _ = env::set_current_dir(&base_dir);
                let (id_injected, toc) = extract_toc_and_inject_ids(&raw);
                self.content = absolutize_image_paths(&id_injected, &base_dir);
                self.raw_content = raw;
                self.current_file_path = path;
                self.toc = toc;
                // Clear the cache so egui_commonmark doesn't carry over stale state.
                self.cache = CommonMarkCache::default();
                // Invalidate and rebuild search for the new content.
                self.search_matches.clear();
                self.search_active = 0;
                if !self.search_query.is_empty() {
                    self.rebuild_search();
                }
                true
            }
            Err(e) => {
                eprintln!("Failed to read {}: {e}", path.display());
                false
            }
        }
    }

    /// Reload the current file from disk without changing history.
    fn reload_file(&mut self) -> bool {
        let path = self.current_file_path.clone();
        self.load_file(path)
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
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
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

    /// Rebuild `search_matches` from `search_query` against `raw_content`.
    /// Resets `search_active` to 0.
    fn rebuild_search(&mut self) {
        self.search_matches.clear();
        self.search_active = 0;
        if self.search_query.is_empty() {
            return;
        }
        let query = self.search_query.to_lowercase();
        let haystack = self.raw_content.to_lowercase();
        let qlen = query.len().max(1);
        let mut start = 0;
        while start < haystack.len() {
            match haystack[start..].find(&query) {
                Some(rel) => {
                    let abs = start + rel;
                    self.search_matches.push(abs);
                    start = abs + qlen;
                }
                None => break,
            }
        }
    }

    /// Scroll the document to the heading section that contains the active search match.
    fn scroll_to_active_match(&mut self) {
        let Some(&byte_pos) = self.search_matches.get(self.search_active) else {
            return;
        };
        // Find the last heading whose byte_start is <= the match position.
        if let Some(entry) = self.toc.iter().rev().find(|e| e.byte_start <= byte_pos) {
            *self.cache.scroll_to_id_target_mut() = Some(entry.slug.clone());
        }
    }
}

impl eframe::App for MarkdownApp {
    #[allow(clippy::too_many_lines)]
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // ── Pre-compute read-only snapshots for use in closures ───────────────────────────
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
        let font_scale = self.font_scale;
        let enhanced_contrast = self.enhanced_contrast;
        let show_toc = self.show_toc;
        let search_open = self.search_open;
        let show_help = self.show_help;
        let match_count = self.search_matches.len();
        let search_active = self.search_active;

        // ── Mutable locals updated by keyboard / buttons, applied at end of frame ─────────
        let mut nav_action = NavAction::None;
        let mut open_file_requested = false;
        let mut refresh_requested = false;
        let mut new_font_scale = self.font_scale;
        let mut new_enhanced_contrast = enhanced_contrast;
        let mut new_show_toc = show_toc;
        let mut new_search_open = search_open;
        let mut new_show_help = show_help;
        let mut search_nav: i32 = 0; // +1 = next match, -1 = prev match
        let font_scale_label = format!("Aa {:.0}%", self.font_scale * 100.0);

        // ── Global keyboard shortcuts ─────────────────────────────────────────────────────
        // Collect all key states in one input() call to avoid re-locking the context.
        let (
            close,
            open_key,
            zoom_in_key,
            zoom_out_key,
            zoom_reset_key,
            font_enlarge_key,
            font_reduce_key,
            font_reset_key,
            cmd_t,
            cmd_f,
            cmd_r,
            f1_key,
            search_enter,
            search_shift_enter,
            search_escape,
        ) = ui.ctx().input(|i| {
            use egui::Key;
            (
                i.modifiers.command && (i.key_pressed(Key::W) || i.key_pressed(Key::Q)),
                i.modifiers.command && i.key_pressed(Key::O),
                i.modifiers.command && i.key_pressed(Key::Equals),
                i.modifiers.command && i.key_pressed(Key::Minus),
                i.modifiers.command && i.key_pressed(Key::Z),
                i.modifiers.command && i.modifiers.shift && i.key_pressed(Key::A),
                i.modifiers.command && i.key_pressed(Key::A),
                i.modifiers.command && i.key_pressed(Key::Num0),
                i.modifiers.command && i.key_pressed(Key::T),
                i.modifiers.command && i.key_pressed(Key::F),
                i.modifiers.command && i.key_pressed(Key::R),
                i.key_pressed(Key::F1),
                i.key_pressed(Key::Enter),
                i.modifiers.shift && i.key_pressed(Key::Enter),
                i.key_pressed(Key::Escape),
            )
        });

        // Act on shortcuts (zoom/font only when text field does not have focus).
        let wants_text = ui.ctx().egui_wants_keyboard_input();
        if close {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        } else if open_key {
            open_file_requested = true;
        } else if !wants_text {
            if zoom_in_key {
                let z = ui.ctx().zoom_factor();
                ui.ctx().set_zoom_factor((z * 1.1).min(3.0));
            } else if zoom_out_key {
                let z = ui.ctx().zoom_factor();
                ui.ctx().set_zoom_factor((z / 1.1).max(0.4));
            } else if zoom_reset_key {
                ui.ctx().set_zoom_factor(1.0);
            } else if font_enlarge_key {
                new_font_scale = (new_font_scale * 1.1).min(3.0);
            } else if font_reduce_key {
                new_font_scale = (new_font_scale / 1.1).max(0.4);
            } else if font_reset_key {
                new_font_scale = 1.0;
            }
        }

        // Feature toggles (independent of text focus).
        if cmd_t {
            new_show_toc = !show_toc;
        }
        if cmd_f {
            new_search_open = !search_open;
            if !search_open {
                // Opening the bar — request focus for the text field.
                self.search_focus = true;
            }
        }
        if cmd_r {
            refresh_requested = true;
        }
        if f1_key {
            new_show_help = !show_help;
        }
        // Search navigation (only meaningful when the bar is open).
        if new_search_open {
            if search_escape {
                new_search_open = false;
            } else if search_enter && !search_shift_enter {
                search_nav = 1;
            } else if search_shift_enter {
                search_nav = -1;
            }
        }

        // ── Top panel: nav bar ────────────────────────────────────────────────────────────
        egui::Panel::top("nav_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                egui_theme_switch::global_theme_switch(ui);
                if ui
                    .selectable_label(
                        enhanced_contrast,
                        if enhanced_contrast {
                            "◑➖"
                        } else {
                            "◑➕"
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
                    .add_enabled(can_go_back, egui::Button::new("◀"))
                    .on_hover_text(&back_tip)
                    .clicked()
                {
                    nav_action = NavAction::Back;
                }
                if ui
                    .add_enabled(can_go_forward, egui::Button::new("▶"))
                    .on_hover_text(&forward_tip)
                    .clicked()
                {
                    nav_action = NavAction::Forward;
                }
                ui.separator();
                if ui
                    .button("📖…")
                    .on_hover_text(format!("Open a markdown file ({MOD}-O)"))
                    .clicked()
                {
                    open_file_requested = true;
                }
                if ui
                    .button("🔄")
                    .on_hover_text(format!("Reload current file from disk ({MOD}-R)"))
                    .clicked()
                {
                    refresh_requested = true;
                }
                if ui
                    .selectable_label(new_show_toc, "§")
                    .on_hover_text(format!("Toggle table of contents ({MOD}-T)"))
                    .clicked()
                {
                    new_show_toc = !new_show_toc;
                }
                if ui
                    .selectable_label(new_search_open, "🔍")
                    .on_hover_text(format!("Search ({MOD}-F)"))
                    .clicked()
                {
                    new_search_open = !new_search_open;
                    if !search_open {
                        self.search_focus = true;
                    }
                }

                ui.separator();

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button("Quit") // 👋
                        .on_hover_text(format!("Close ({MOD}-W)"))
                        .clicked()
                    {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui
                        .selectable_label(new_show_help, "❓")
                        .on_hover_text("Help (F1)")
                        .clicked()
                    {
                        new_show_help = !new_show_help;
                    }
                    ui.separator();
                    // Font scale controls (content text only; toolbar stays at 1×).
                    // RTL layout: first-added = rightmost, so render `+, label, −`
                    // to produce the visual sequence `− | 100% | +` left-to-right.
                    if ui
                        .small_button("+")
                        .on_hover_text(format!("Enlarge fonts ({MOD}-Shift-A)"))
                        .clicked()
                    {
                        new_font_scale = (new_font_scale * 1.1).min(3.0);
                    }
                    if ui
                        .button(&font_scale_label)
                        .on_hover_text(format!("Reset font size to 100% ({MOD}-0)"))
                        .clicked()
                    {
                        new_font_scale = 1.0;
                    }
                    if ui
                        .small_button("−")
                        .on_hover_text(format!("Reduce fonts ({MOD}-A)"))
                        .clicked()
                    {
                        new_font_scale = (new_font_scale / 1.1).max(0.4);
                    }
                    ui.separator();
                    // Zoom controls.
                    let zoom = ui.ctx().zoom_factor();
                    if ui
                        .small_button("+")
                        .on_hover_text(format!("Zoom in ({MOD}-=)"))
                        .clicked()
                    {
                        ui.ctx().set_zoom_factor((zoom * 1.1).min(3.0));
                    }
                    if ui
                        .button(format!("↕{:.0}%", zoom * 100.0))
                        .on_hover_text(format!("Reset zoom to 100% ({MOD}-Z)"))
                        .clicked()
                    {
                        ui.ctx().set_zoom_factor(1.0);
                    }
                    if ui
                        .small_button("−")
                        .on_hover_text(format!("Zoom out ({MOD}-−)"))
                        .clicked()
                    {
                        ui.ctx().set_zoom_factor((zoom / 1.1).max(0.4));
                    }
                    ui.separator();
                });
            });
        });

        // ── Top panel: search bar (shown when search is open) ─────────────────────────────
        if new_search_open {
            egui::Panel::top("search_bar").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Find in document…")
                            .desired_width(280.0),
                    );
                    // Grab focus when the bar first opens.
                    if self.search_focus {
                        response.request_focus();
                        self.search_focus = false;
                    }
                    if response.changed() {
                        self.rebuild_search();
                        // search_active already reset inside rebuild_search
                    }

                    // Match counter.
                    if match_count == 0 && !self.search_query.is_empty() {
                        ui.weak("No matches");
                    } else if match_count > 0 {
                        ui.weak(format!("{} / {}", search_active + 1, match_count));
                    }

                    // Prev / next buttons.
                    let has_matches = match_count > 0;
                    if ui
                        .add_enabled(has_matches, egui::Button::new("⬆"))
                        .on_hover_text("Previous match (Shift-Enter)")
                        .clicked()
                    {
                        search_nav = -1;
                    }
                    if ui
                        .add_enabled(has_matches, egui::Button::new("⬇"))
                        .on_hover_text("Next match (Enter)")
                        .clicked()
                    {
                        search_nav = 1;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button("x")
                            .on_hover_text("Close search (Esc)")
                            .clicked()
                        {
                            new_search_open = false;
                        }
                    });
                });
            });
        }

        // ── Left side panel: table of contents ───────────────────────────────────────────
        if new_show_toc {
            egui::Panel::left("toc")
                .resizable(true)
                .default_size(220.0)
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    ui.strong("Contents");
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .id_salt("toc_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for entry in &self.toc {
                                let indent = f32::from(entry.level.saturating_sub(1)) * 10.0;
                                // Reserve a background slot BEFORE the row so it sits
                                // behind the text in the draw-list (correct z-order).
                                let bg_idx = ui.painter().add(egui::Shape::Noop);
                                // Full-width row: click/hover sense covers the whole
                                // row, not just the label text.
                                let row_h = ui.spacing().interact_size.y;
                                let (row_rect, row_resp) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), row_h),
                                    egui::Sense::click(),
                                );
                                if row_resp.hovered() {
                                    ui.painter().set(
                                        bg_idx,
                                        egui::Shape::rect_filled(
                                            row_rect,
                                            egui::CornerRadius::ZERO,
                                            ui.visuals().widgets.hovered.weak_bg_fill,
                                        ),
                                    );
                                }
                                // Paint the truncated label in the indented portion.
                                // allocate_new_ui does not advance the parent cursor
                                // (row_rect was already accounted for above), and the
                                // explicit Layout prevents put()'s centred-and-justified
                                // default from pushing text to the middle of the panel.
                                if ui.is_rect_visible(row_rect) {
                                    // Paint the text directly so we control both position
                                    // and truncation without fighting the layout system.
                                    let color = ui.visuals().text_color();
                                    let font_id = egui::TextStyle::Body.resolve(ui.style());
                                    let max_w = (row_rect.width() - indent).max(0.0);
                                    let mut job = egui::text::LayoutJob::single_section(
                                        entry.text.clone(),
                                        egui::TextFormat {
                                            font_id,
                                            color,
                                            ..Default::default()
                                        },
                                    );
                                    job.wrap.max_rows = 1;
                                    job.wrap.overflow_character = Some('\u{2026}'); // …
                                    job.wrap.max_width = max_w;
                                    let galley = ui.ctx().fonts_mut(|f| f.layout_job(job));
                                    let y = row_rect.center().y - galley.size().y * 0.5;
                                    ui.painter().galley(
                                        egui::pos2(row_rect.min.x + indent, y),
                                        galley,
                                        color,
                                    );
                                }
                                if row_resp
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    *self.cache.scroll_to_id_target_mut() =
                                        Some(entry.slug.clone());
                                }
                            }
                        });
                });
        } // end TOC panel

        // ── Central panel: the markdown document ──────────────────────────────────────────
        egui::CentralPanel::default().show(ui, |ui| {
            // Floating scrollbar that reserves its own layout space so the
            // code-block copy icon is never obscured by the bar on hover.
            {
                let scroll = &mut ui.style_mut().spacing.scroll;
                scroll.floating = true;
                scroll.content_margin = egui::Margin::same(10);
                scroll.bar_width = 10.0;
                scroll.dormant_handle_opacity = 0.15;
                scroll.interact_handle_opacity = 0.55;
                scroll.active_handle_opacity = 0.80;
            }
            // Give each file path its own scroll-state key so that:
            //   • navigating to a new file always starts at the top, and
            //   • back/forward navigation restores the previous scroll position.
            egui::ScrollArea::vertical()
                .id_salt(&current_path_label)
                .show(ui, |ui| {
                    // Scale content text styles locally so the toolbar is unaffected.
                    if (font_scale - 1.0).abs() > 0.005 {
                        use egui::{FontFamily, FontId, TextStyle};
                        let s = ui.style_mut();
                        let base_body =
                            s.text_styles.get(&TextStyle::Body).map_or(14.0, |f| f.size);
                        let base_mono = s
                            .text_styles
                            .get(&TextStyle::Monospace)
                            .map_or(12.0, |f| f.size);
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
                            FontId::new(base_body * font_scale, FontFamily::Proportional),
                        );
                        s.text_styles.insert(
                            TextStyle::Monospace,
                            FontId::new(base_mono * font_scale, FontFamily::Monospace),
                        );
                        s.text_styles.insert(
                            TextStyle::Heading,
                            FontId::new(base_heading * font_scale, FontFamily::Proportional),
                        );
                        s.text_styles.insert(
                            TextStyle::Small,
                            FontId::new(base_small * font_scale, FontFamily::Proportional),
                        );
                    }
                    CommonMarkViewer::new()
                        .syntax_theme_dark("base16-ocean.dark")
                        .syntax_theme_light("InspiredGitHub")
                        .enable_scroll_to_heading(true)
                        .show(ui, &mut self.cache, &self.content);
                });
        });

        // ── Intercept link clicks from egui_commonmark ────────────────────────────────────
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

        // ── Execute navigation ────────────────────────────────────────────────────────────
        let navigated = if let Some(url) = clicked_url {
            if url.starts_with("http://") || url.starts_with("https://") {
                // Re-queue external links for the platform to open in the browser.
                ui.ctx().open_url(egui::output::OpenUrl::new_tab(url));
                false
            } else {
                self.handle_link_click(&url)
            }
        } else if open_file_requested {
            let start_dir = self
                .current_file_path
                .parent()
                .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
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
        } else if refresh_requested {
            self.reload_file()
            // No history change on refresh.
        } else {
            match nav_action {
                NavAction::Back => self.go_back(),
                NavAction::Forward => self.go_forward(),
                NavAction::None => false,
            }
        };

        // ── Search navigation (applied after all panels are drawn) ────────────────────────
        if search_nav != 0 && match_count > 0 {
            if search_nav > 0 {
                self.search_active = (self.search_active + 1) % match_count;
            } else {
                self.search_active = if self.search_active == 0 {
                    match_count - 1
                } else {
                    self.search_active - 1
                };
            }
            self.scroll_to_active_match();
        }

        // ── Commit state changes ──────────────────────────────────────────────────────────
        self.font_scale = new_font_scale;
        self.show_toc = new_show_toc;
        self.search_open = new_search_open;
        self.show_help = new_show_help;

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

        // ── Help window (floats above everything else) ────────────────────────────────────
        if self.show_help {
            let mut open = true;
            egui::Window::new("Help")
                .resizable(true)
                .default_size([620.0, 480.0])
                .collapsible(false)
                .open(&mut open)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        CommonMarkViewer::new().show(ui, &mut self.help_cache, HELP_TEXT);
                    });
                });
            if !open {
                self.show_help = false;
            }
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

        let FastSvgState {
            pass_index,
            cache,
            options,
        } = &mut *(self.state.lock().unwrap());

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
