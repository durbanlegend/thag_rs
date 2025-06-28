/*[toml]
[dependencies]
egui = "0.27.0"
eframe = { version = "0.27.0", default-features = false, features = [
    "accesskit",     # Make egui comptaible with screen readers. NOTE: adds a lot of dependencies.
    "default_fonts", # Embed the default egui fonts.
    "glow",          # Use the glow rendering backend. Alternative: "wgpu".
    "persistence",   # Enable restoring app state when restarting the app.
] }

# You only need serde if you want app persistence:
serde = { version = "1", features = ["derive"] }
egui_extras = "0.27.2"

[features]
default = ["syntect"]

## Enable better syntax highlighting using [`syntect`](https://docs.rs/syntect).
syntect = ["egui_extras/syntect"]
*/

#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe;
use env_logger;

/// A prototype GUI editor with saved state and syntax highlighting.
//# Purpose: Prototype a native-mode editor using the `egui` crate.
//# Categories: crates, prototype
// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct CodeEditor {
    language: String,
    code: String,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            language: "rs".into(),
            code: r#"// A very simple example
fn main() {
	println!("Hello world!");
}
"#
            .into(),
        }
    }
}

impl CodeEditor {
    /// Called once before the first frame.
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let ctx = &cc.egui_ctx;
        ctx.set_zoom_factor(1.2);
        // ctx.set_visuals(egui::Visuals {
        //     extreme_bg_color: Color32::LIGHT_GRAY,
        //     ..Default::default()
        // });

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        CodeEditor::default()
    }
}

impl eframe::App for CodeEditor {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let Self { language, code } = self;

            let mut theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
            ui.collapsing("Theme", |ui| {
                ui.group(|ui| {
                    theme.ui(ui);
                    theme.clone().store_in_memory(ui.ctx());
                });
            });

            let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                let mut layout_job =
                    egui_extras::syntax_highlighting::highlight(ui.ctx(), &theme, string, language);
                layout_job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(layout_job))
            };

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(code)
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .code_editor()
                        .desired_rows(25)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });
        });
    }
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        // .with_icon(
        //     // NOTE: Adding an icon is optional
        //     eframe::icon_data::from_png_bytes(
        //         &include_bytes!("path_to_icon/icon-256.png")[..],
        //     )
        //     .unwrap(),
        // ),
        ..Default::default()
    };
    eframe::run_native(
        "⌨️ Code Editor",
        native_options,
        Box::new(|cc| Box::new(CodeEditor::new(cc))),
    )
}
