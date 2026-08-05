/*[toml]
[dependencies]
egui_commonmark = { git = "https://github.com/durbanlegend/egui_commonmark", branch = "feat/scrollable-extras", features = ["better_syntax_highlighting", "svg", "fetch"] }
# egui_commonmark = { path = "/Users/donf/projects/egui_commonmark/egui_commonmark" }
# env_logger = "0.11"

[features]
default = ["eframe/wgpu", "egui_commonmark/better_syntax_highlighting","egui_commonmark/svg","egui_commonmark/fetch"]

# Make sure the result runs fast
[profile.dev]
opt-level = 3     # Apply maximum performance optimizations
debug = false
*/
use std::env;

use eframe::egui;
use egui::Ui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

struct App {
    cache: CommonMarkCache,
    content: String,
    viewport_cache: bool,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        ui.set_min_height(512.0);

        egui::CentralPanel::default().show(ui, |ui| {
            // ── Style the scroll bar to show the them position clearly for the demo.
            // This Ui gets passed along to and used by show_scrollable.
            {
                let scroll = &mut ui.style_mut().spacing.scroll;
                scroll.floating = true;
                scroll.floating_width = 7.0;
                scroll.content_margin = egui::Margin::same(10);
                scroll.bar_width = 10.0;
                scroll.dormant_handle_opacity = 0.40;
                scroll.interact_handle_opacity = 0.55;
                scroll.active_handle_opacity = 0.80;
            }

            let (
                scroll_line_up,
                scroll_line_down,
                scroll_page_up,
                scroll_page_down,
                scroll_doc_top,
                scroll_doc_bottom,
            ) = ui.ctx().input(|i| {
                use egui::Key;
                (
                    // Scroll keys — only plain (non-Cmd) arrow keys for line scroll.
                    !i.modifiers.command && i.key_pressed(Key::ArrowUp),
                    !i.modifiers.command && i.key_pressed(Key::ArrowDown),
                    i.key_pressed(Key::PageUp),
                    i.key_pressed(Key::PageDown),
                    // Home / End: physical key OR Cmd+Arrow (standard macOS navigation).
                    i.key_pressed(Key::Home)
                        || (i.modifiers.command && i.key_pressed(Key::ArrowUp)),
                    i.key_pressed(Key::End)
                        || (i.modifiers.command && i.key_pressed(Key::ArrowDown)),
                )
            });

            // Act on shortcuts only when text field does not have focus.
            let wants_text = ui.ctx().egui_wants_keyboard_input();

            // ── Keyboard scrolling ───────────────────────────────────────────
            // Deltas are threaded through CommonMarkCache so show_scrollable
            // can apply them inside its own internal ScrollArea.
            if !wants_text {
                let line_h = ui.text_style_height(&egui::TextStyle::Body);
                let page_h = ui.available_height();
                // eprintln!("line_h={line_h}, page_h={page_h}");
                if scroll_line_up {
                    self.cache.set_scroll_delta(egui::vec2(0.0, line_h));
                } else if scroll_line_down {
                    self.cache.set_scroll_delta(egui::vec2(0.0, -line_h));
                } else if scroll_page_up {
                    self.cache.set_scroll_delta(egui::vec2(0.0, page_h));
                } else if scroll_page_down {
                    self.cache.set_scroll_delta(egui::vec2(0.0, -page_h));
                } else if scroll_doc_top {
                    self.cache.set_scroll_delta(egui::vec2(0.0, f32::MAX / 2.0));
                } else if scroll_doc_bottom {
                    self.cache
                        .set_scroll_delta(egui::vec2(0.0, -f32::MAX / 2.0));
                }
            }

            CommonMarkViewer::new()
                .max_image_width(Some(512))
                .viewport_cache(self.viewport_cache)
                .show_scrollable("Generated content", ui, &mut self.cache, &self.content);
        });
    }
}

fn main() {
    let mut args = env::args();
    args.next();

    let viewport_cache = env::var("CACHE")
        .map(|v| v.to_lowercase() != "false" && v != "0")
        .unwrap_or(true);

    let text = build_document();

    // Creates or overwrites "output.txt" with the string data
    std::fs::write(concat!(env!("TMPDIR"), "/scroll.md"), &text);

    eprintln!("Document size is {} bytes", text.len());

    eframe::run_native(
        "Markdown viewer",
        eframe::NativeOptions::default(),
        Box::new(move |cc| {
            if let Some(theme) = args.next() {
                if theme == "light" {
                    cc.egui_ctx.set_theme(egui::Theme::Light);
                } else if theme == "dark" {
                    cc.egui_ctx.set_theme(egui::Theme::Dark);
                }
            }

            Ok(Box::new(App {
                cache: CommonMarkCache::default(),
                content: text,
                viewport_cache,
            }))
        }),
    )
    .unwrap();
}

fn build_document() -> String {
    let mut text = r"# Commonmark Viewer Example
    This is a fairly large markdown file showcasing scroll.

    After the first rendering pass it should be responsive.
    But it will need to re-render each time the app is resized
    or if the content gets modified for any reason.

    To experience uncached performance for comparison, run
    with the environment variable `CACHE=false`.

    The scrollbar has deliberately been made conspicuous
    for the demonstration.

    Try using the scrolling shortcuts:
        Home:           Fn-left arrow
        End:            Fn-right arrow
        Up   1 line:    Up-arrow
        Down 1 line:    Down-arrow
        Up   1 page:    Fn-up arrow
        Down 1 page:    Fn-up arrow
                "
    .to_string();

    let repeating = r"
This section will be repeated

```rs
let mut vec = Vec::new();
vec.push(5);
```

# Plans
* Make a sandwich
* Bake a cake
* Conquer the world
* Meet Ferris

![Ferris the Rust mascot](egui_commonmark/examples/cuddlyferris.png)
    ";
    text += &repeating.repeat(1024);
    text
}
