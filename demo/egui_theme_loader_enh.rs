/*[toml]
[package]
name = "egui_theme_loader_enh"
version = "0.1.0"
edition = "2021"

[dependencies]
# eframe = "0.36.2"
# serde_yaml = "0.9"
# default-fancy = pure-Rust regex backend (no C/onig build needed).
# If you already depend on syntect via egui_commonmark, match that version/features.
syntect = { version = "5", default-features = false, features = ["default-fancy"] }
*/
/// Demo of enhanced RYO `egui` theming using a `syntect` `.tmTheme` file.
///
/// E.g.: `thag demo/egui_theme_loader_enh.rs -- /path/to/file.tmTheme`
///
//# Purpose: demo RYO `egui` theming from popular `syntect` themes.
//# Categories: crates, demo, styling, technique
//# Argument: PATH: Path to a `syntect` `.thTheme` file or a `Base16` `.y[a]ml` file.
use eframe::egui::{self, RichText};
use markdown::MarkdownColours;
use theme::Roles;

struct Demo {
    md: MarkdownColours,
    text: String,
    checked: bool,
    slider: f32,
}

impl eframe::App for Demo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Theme preview");
            ui.label("Normal text");
            ui.weak("Weak text");
            ui.hyperlink("https://example.com");
            ui.code("inline_code()");
            ui.colored_label(ui.visuals().warn_fg_color, "warning");
            ui.colored_label(ui.visuals().error_fg_color, "error");
            ui.separator();
            ui.text_edit_singleline(&mut self.text);
            ui.checkbox(&mut self.checked, "A checkbox");
            ui.add(egui::Slider::new(&mut self.slider, 0.0..=1.0));
            let _ = ui.button("A button");

            ui.separator();
            ui.label("Markdown-style text, drawn the way the viewer does it:");

            // In your app, replace the contents of this closure with:
            //     CommonMarkViewer::new().show(ui, &mut cache, markdown_text);
            self.md.scope(ui, |ui| {
                ui.label(RichText::new("A heading").heading().strong());
                ui.horizontal_wrapped(|ui| {
                    ui.label("Text with ");
                    ui.label(RichText::new("bold").strong());
                    ui.label(", ");
                    ui.label(RichText::new("italic").italics());
                    ui.label(" and ");
                    ui.label(RichText::new("inline code").code());
                    ui.label(".");
                });
                ui.label(RichText::new("A block quote").weak());
                ui.hyperlink("https://example.com");
            });
        });
    }
}

/// Pick a loader by file extension: .yaml/.yml = base16, .tmTheme = syntect.
fn load(path: &str) -> Result<(String, Roles), Box<dyn std::error::Error>> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match ext.as_deref() {
        Some("yaml" | "yml") => base16::from_file(path),
        Some("tmtheme") => tmtheme::from_file(path),
        _ => Err("expected a .yaml/.yml (base16) or .tmTheme file".into()),
    }
}

fn main() -> eframe::Result<()> {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: base16-egui <theme.yaml | theme.tmTheme>");
        std::process::exit(2);
    });
    let (name, roles) = load(&path).unwrap_or_else(|e| {
        eprintln!("failed to load {path}: {e}");
        std::process::exit(1);
    });
    println!(
        "Loaded `{name}` ({})",
        if roles.is_dark { "dark" } else { "light" }
    );
    eprintln!("Markdown colours: {:#?}", roles.md);

    eframe::run_native(
        "egui_theme_loader_enh",
        eframe::NativeOptions::default(),
        Box::new(move |cc| {
            roles.apply(&cc.egui_ctx);
            Ok(Box::new(Demo {
                md: roles.md,
                text: "Edit me".into(),
                checked: true,
                slider: 0.4_f32,
            }))
        }),
    )
}

/// Load a base16 YAML scheme into UI colour roles.
///
/// Supports both the current "tinted-theming" layout (colours nested under
/// `palette:`) and the older flat layout (`base00: "..."` at the top level).
mod base16 {
    use crate::markdown::MarkdownColours;
    use crate::theme::{Roles, luminance};
    use eframe::egui::Color32;
    use std::{error::Error, fs, path::Path};

    pub fn from_file(path: impl AsRef<Path>) -> Result<(String, Roles), Box<dyn Error>> {
        from_yaml(&fs::read_to_string(path)?)
    }

    pub fn from_yaml(src: &str) -> Result<(String, Roles), Box<dyn Error>> {
        let root: serde_yaml::Value = serde_yaml::from_str(src)?;
        let palette = root.get("palette").unwrap_or(&root);

        let mut c = [Color32::PLACEHOLDER; 16];
        for (i, slot) in c.iter_mut().enumerate() {
            let key = format!("base{i:02X}");
            let hex = palette
                .get(&key)
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("missing colour `{key}`"))?;
            *slot = parse_hex(hex)?;
        }

        let name = root
            .get("name")
            .or_else(|| root.get("scheme"))
            .and_then(|v| v.as_str())
            .unwrap_or("base16")
            .to_owned();

        // Use the `variant` field if present, otherwise guess from base00's luminance.
        let is_dark = match root.get("variant").and_then(|v| v.as_str()) {
            Some("dark") => true,
            Some("light") => false,
            _ => luminance(c[0]) < 0.5_f32,
        };

        let roles = Roles {
            is_dark,
            bg: c[0x0],
            bg_alt: c[0x1],
            bg_sel: c[0x2],
            border: c[0x3],
            muted: c[0x4],
            fg: c[0x5],
            fg_hi: c[0x6],
            fg_max: c[0x7],
            accent: c[0xD], // blue
            sel_bg: c[0xD],
            sel_fg: c[0x0],
            error: c[0x8], // red
            warn: c[0x9],  // orange
            // base16 styling guide: headings 0D, bold 0A, italic 0E, inline code 0B,
            // quotes 0C, lists 08, link URLs 09.
            md: MarkdownColours {
                heading: c[0xD],
                bold: c[0xA],
                italic: c[0xE],
                code: c[0xB],
                quote: c[0xC],
                list: c[0x8],
                link: c[0x9],
            },
        };
        Ok((name, roles))
    }

    fn parse_hex(s: &str) -> Result<Color32, Box<dyn Error>> {
        let h = s.trim().trim_start_matches('#');
        if h.len() != 6 {
            return Err(format!("expected 6 hex digits, got `{s}`").into());
        }
        let p = |i: usize| u8::from_str_radix(&h[i..i + 2], 16);
        Ok(Color32::from_rgb(p(0)?, p(2)?, p(4)?))
    }
}

/// Derive UI colour roles from a syntect `.tmTheme` file.
///
/// A tmTheme only defines an editor background/foreground plus scope colours,
/// so the in-between greys (borders, hover fills, ...) are blended from
/// background and foreground, and accents are looked up from scopes.
mod tmtheme {
    use crate::markdown::MarkdownColours;
    use crate::theme::{Roles, luminance, mix};
    use eframe::egui::Color32;
    use std::{error::Error, path::Path};
    use syntect::highlighting::{Color, Highlighter, ThemeSet};
    use syntect::parsing::Scope;

    #[allow(clippy::similar_names)]
    pub fn from_file(path: impl AsRef<Path>) -> Result<(String, Roles), Box<dyn Error>> {
        let theme = ThemeSet::get_theme(path.as_ref())?;
        let s = &theme.settings;

        let rgb = |c: Color| Color32::from_rgb(c.r, c.g, c.b);

        let bg = s.background.map_or(Color32::WHITE, rgb);
        // Deliberately ignores alpha: some themes (e.g. gruvbox Light) store the
        // foreground as #RRGGBB80 for editor reasons; flattening it would wash out all text.
        let fg = s.foreground.map_or(Color32::BLACK, rgb);
        let is_dark = luminance(bg) < 0.5;
        let pure = if is_dark {
            Color32::WHITE
        } else {
            Color32::BLACK
        };

        // Colours in tmThemes may carry alpha; flatten them onto a background
        // so egui never has to deal with translucent fills.
        let flatten = |c: Color, over: Color32| mix(over, rgb(c), f32::from(c.a) / 255.0);

        // Greys derived from bg/fg, unless the theme supplies something better.
        let bg_alt = s
            .line_highlight
            .map_or_else(|| mix(bg, fg, 0.06), |c| flatten(c, bg));
        // Idle widgets sit on `bg_alt`, so hover/pressed must step away from *that*,
        // not from `bg`, or hovering would barely change anything.
        let bg_sel = mix(bg_alt, fg, 0.12);
        let border = mix(bg_alt, fg, 0.25);
        let muted = mix(bg, fg, 0.50);

        // Colour of the first scope in `names` that the theme actually styles.
        let hl = Highlighter::new(&theme);
        let scope_fg = |names: &[&str]| -> Option<Color32> {
            names.iter().find_map(|n| {
                let scope = Scope::new(n).ok()?;
                let c = hl.style_mod_for_stack(&[scope]).foreground?;
                Some(flatten(c, bg))
            })
        };

        let accent = scope_fg(&[
            "markup.underline.link",
            "string.other.link",
            "entity.name.function",
            "keyword",
        ])
        .unwrap_or(fg);
        // For "alarm" scopes the background is usually the real signal colour: many
        // themes style `invalid` as dark/white text *on* red, so its foreground
        // would make error text look like normal text.
        let scope_signal = |names: &[&str]| -> Option<Color32> {
            names.iter().find_map(|n| {
                let m = hl.style_mod_for_stack(&[Scope::new(n).ok()?]);
                m.background.or(m.foreground).map(|c| flatten(c, bg))
            })
        };
        let error = scope_signal(&["invalid.illegal", "invalid", "markup.deleted"])
            .unwrap_or(Color32::from_rgb(200, 50, 50));
        let warn = scope_fg(&["markup.changed", "constant.numeric", "constant"])
            .unwrap_or(Color32::from_rgb(220, 140, 30));

        // Selection: use the theme's own colour, but only if it is clearly visible
        // against BOTH the panel background and `bg_alt` (text edits, code blocks).
        // Editors often use a subtle selection that equals the line highlight, which
        // would be invisible in egui. Otherwise use a tint of the accent colour.
        let far = |a: Color32, b: Color32| {
            let d = |x: u8, y: u8| (i32::from(x) - i32::from(y)).abs();
            d(a.r(), b.r()) + d(a.g(), b.g()) + d(a.b(), b.b()) >= 80
        };
        let themed = s
            .selection
            .map(|c| (flatten(c, bg), s.selection_foreground.map_or(fg, rgb)));
        let (sel_bg, sel_fg) = match themed {
            Some((b, f)) if far(b, bg) && far(b, bg_alt) => (b, f),
            _ => (mix(bg, accent, 0.35), fg),
        };

        // Markdown element colours. Anything the theme doesn't style falls back to
        // plain text colour, i.e. no visible change for that element.
        let fg_hi = mix(fg, pure, 0.35);
        let md = MarkdownColours {
            heading: scope_fg(&["markup.heading"]).unwrap_or(fg_hi),
            bold: scope_fg(&["markup.bold"]).unwrap_or(fg_hi),
            italic: scope_fg(&["markup.italic"]).unwrap_or(fg),
            code: scope_fg(&["markup.raw.inline", "markup.raw"]).unwrap_or(fg),
            quote: scope_fg(&["markup.quote"]).unwrap_or_else(|| mix(fg, bg, 0.4)),
            list: scope_fg(&["markup.list"]).unwrap_or(fg),
            link: accent,
        };

        let roles = Roles {
            is_dark,
            bg,
            bg_alt,
            bg_sel,
            border,
            muted,
            fg,
            fg_hi,
            fg_max: mix(fg, pure, 0.70),
            accent,
            sel_bg,
            sel_fg,
            error,
            warn,
            md,
        };

        let name = theme.name.clone().unwrap_or_else(|| "tmTheme".to_owned());
        Ok((name, roles))
    }
}

/// Source-agnostic UI colour roles and their mapping onto `egui::Visuals`.
/// Both the base16 and the .tmTheme loaders just fill in a `Roles`.
mod theme {
    use crate::markdown::MarkdownColours;
    use eframe::egui::{Color32, Stroke, Theme, Visuals};

    pub struct Roles {
        pub is_dark: bool,
        pub bg: Color32,     // panels, windows
        pub bg_alt: Color32, // text edits, idle buttons, code bg, striped rows
        pub bg_sel: Color32, // hovered widgets, separators
        pub border: Color32, // window border, pressed widgets
        pub muted: Color32,  // pressed-widget outline
        pub fg: Color32,     // normal text
        pub fg_hi: Color32,  // hovered text
        pub fg_max: Color32, // pressed text
        pub accent: Color32, // hyperlinks (and selection when no better source)
        pub sel_bg: Color32, // text selection background
        pub sel_fg: Color32, // text colour on top of the selection
        pub error: Color32,
        pub warn: Color32,
        pub md: MarkdownColours, // per-element Markdown colours (see markdown.rs)
    }

    impl Roles {
        pub fn visuals(&self) -> Visuals {
            // Start from egui's own defaults so every field we don't touch stays sane.
            let mut v = if self.is_dark {
                Visuals::dark()
            } else {
                Visuals::light()
            };

            v.panel_fill = self.bg;
            v.window_fill = self.bg;
            v.window_stroke = Stroke::new(1.0_f32, self.border);
            v.extreme_bg_color = self.bg_alt; // text edits, scroll areas
            v.faint_bg_color = self.bg_alt; // striped table rows
            v.code_bg_color = self.bg_alt; // inline code (markdown)

            v.hyperlink_color = self.accent;
            v.warn_fg_color = self.warn;
            v.error_fg_color = self.error;
            v.selection.bg_fill = self.sel_bg;
            v.selection.stroke = Stroke::new(1.0_f32, self.sel_fg);

            // Text colour comes from each state's `fg_stroke`.
            let w = &mut v.widgets;

            w.noninteractive.bg_fill = self.bg;
            w.noninteractive.weak_bg_fill = self.bg;
            w.noninteractive.bg_stroke = Stroke::new(1.0_f32, self.bg_sel);
            w.noninteractive.fg_stroke = Stroke::new(1.0_f32, self.fg);

            w.inactive.bg_fill = self.bg_alt;
            w.inactive.weak_bg_fill = self.bg_alt;
            w.inactive.bg_stroke = Stroke::NONE;
            w.inactive.fg_stroke = Stroke::new(1.0_f32, self.fg);

            w.hovered.bg_fill = self.bg_sel;
            w.hovered.weak_bg_fill = self.bg_sel;
            w.hovered.bg_stroke = Stroke::new(1.0_f32, self.border);
            w.hovered.fg_stroke = Stroke::new(1.5_f32, self.fg_hi);

            w.active.bg_fill = self.border;
            w.active.weak_bg_fill = self.border;
            w.active.bg_stroke = Stroke::new(1.0_f32, self.muted);
            w.active.fg_stroke = Stroke::new(2.0_f32, self.fg_max);

            w.open.bg_fill = self.bg_alt;
            w.open.weak_bg_fill = self.bg_alt;
            w.open.bg_stroke = Stroke::new(1.0_f32, self.border);
            w.open.fg_stroke = Stroke::new(1.0_f32, self.fg_hi);

            v
        }

        /// egui keeps separate Light/Dark styles and follows the OS theme by default,
        /// so write to the matching slot and pin the theme preference.
        pub fn apply(&self, ctx: &eframe::egui::Context) {
            let theme = if self.is_dark {
                Theme::Dark
            } else {
                Theme::Light
            };
            ctx.set_visuals_of(theme, self.visuals());
            ctx.set_theme(theme);
        }
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    /// Linear blend in sRGB space: t = 0 gives `a`, t = 1 gives `b`.
    pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
        let ch = |x: u8, y: u8| {
            (f32::from(y) - f32::from(x))
                .mul_add(t, f32::from(x))
                .round() as u8
        };
        Color32::from_rgb(ch(a.r(), b.r()), ch(a.g(), b.g()), ch(a.b(), b.b()))
    }

    pub fn luminance(c: Color32) -> f32 {
        0.0722f32.mul_add(
            f32::from(c.b()),
            0.7152f32.mul_add(f32::from(c.g()), 0.2126 * f32::from(c.r())),
        ) / 255.0
    }
}

/// Per-element Markdown colours, and the part of them that egui can express
/// without patching `egui_commonmark`.
///
/// `CommonMarkViewer` has no colour options. It builds `RichText` and lets egui
/// pick the colour, so the only levers are the style fields egui consults:
///
/// | Markdown element     | How the viewer draws it              | Style field we can set          |
/// |----------------------|--------------------------------------|---------------------------------|
/// | Headings             | `RichText::strong()` + larger size   | `widgets.active.fg_stroke`      |
/// | **Bold**             | `RichText::strong()`                 | same field (shared with headings) |
/// | > Block quote + bar  | `RichText::weak()`, bar = weak colour| `weak_text_color`               |
/// | [Links](...)         | egui hyperlink                       | `hyperlink_color`               |
/// | *Italic*             | `RichText::italics()` (no colour)    | none                            |
/// | `Inline code`        | `RichText::code()` (bg only)         | none (background: `code_bg_color`) |
/// | List bullets/numbers | default text colour                  | none                            |
///
/// Headings and bold share one colour because both go through `strong()`.
/// To colour italic / code / lists / headings-vs-bold independently you need a
/// small patch to the viewer; see the note at the bottom of this file.
mod markdown {

    use eframe::egui::{Color32, InnerResponse, Ui};

    #[allow(dead_code)]
    #[derive(Clone, Copy, Debug)]
    pub struct MarkdownColours {
        pub heading: Color32,
        pub bold: Color32,
        pub italic: Color32,
        pub code: Color32,
        pub quote: Color32,
        pub list: Color32,
        pub link: Color32,
    }

    impl MarkdownColours {
        /// Run `add_contents` (e.g. your `CommonMarkViewer::show`) in a child `Ui`
        /// whose style applies every colour egui lets us control. The override is
        /// local to that child `Ui`; the rest of your app is unaffected.
        ///
        /// ```ignore
        /// md.scope(ui, |ui| CommonMarkViewer::new().show(ui, &mut cache, text));
        /// ```
        pub fn scope<R>(
            &self,
            ui: &mut Ui,
            add_contents: impl FnOnce(&mut Ui) -> R,
        ) -> InnerResponse<R> {
            ui.scope(|ui| {
                let v = &mut ui.style_mut().visuals;
                // `strong()` text uses `widgets.active.fg_stroke.color`. Headings and
                // bold both use it. Swap to `self.bold` if you prefer bold's colour.
                v.widgets.active.fg_stroke.color = self.heading;
                v.weak_text_color = Some(self.quote); // quote text and its side bar
                v.hyperlink_color = self.link;
                add_contents(ui)
            })
        }
    }

    // ---------------------------------------------------------------------------
    // Going further: colouring italic, inline code, lists and headings-vs-bold
    // ---------------------------------------------------------------------------
    // These need a change inside the viewer's `Style::to_richtext` (in
    // `egui_commonmark_backend`, `misc.rs`), which is where flags like `heading`,
    // `strong`, `emphasis`, `code` and `quote` become a `RichText`. Vendor or fork
    // that crate (MIT/Apache), then at the end of `to_richtext` do something like:
    //
    //     if let Some(c) = ui.ctx().data(|d| d.get_temp::<MdColours>(egui::Id::NULL)) {
    //         if self.heading.is_some()   { text = text.color(c.heading); }
    //         else if self.strong         { text = text.color(c.bold);    }
    //         else if self.emphasis       { text = text.color(c.italic);  }
    //         else if self.code           { text = text.color(c.code);    }
    //     }
    //
    // and store the colours once with `ctx.data_mut(|d| d.insert_temp(Id::NULL, colours))`.
    // (`MdColours` must be a type the patched crate can see, so define it there.)
    // List markers are drawn by separate helpers (`bullet_point`, `number_point`)
    // and would need the same treatment.
}
