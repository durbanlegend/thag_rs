use std::fs;
use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;

use rust_fontconfig::{FcFontCache, FcPattern, FontId};

// let mut fonts = egui::FontDefinitions::default();
// let mut loaded_fonts: HashSet<String> = HashSet::new();

let start = Instant::now();
let cache = FcFontCache::build();
let build_time = start.elapsed();
println!("Font cache built with {} fonts\n", cache.list().len());

let disp = |pattern: &FcPattern, font_id: &FontId| {
    println!("pattern: {pattern:?}, font_id:{font_id}");
};

cache.for_each_pattern(disp);

cache.list().iter().for_each(|(pattern, font_id)| eprintln!("pattern: {pattern:?},
    font_id:{font_id}, font_bytes={:?}", cache.get_font_bytes(&font_id)));

// if loaded_fonts.is_empty() {
//     eprintln!("No system fonts loaded. Unicode characters may show as red triangles.");
//     eprintln!("Install noto-fonts and noto-fonts-cjk for full Unicode support.");
// } else {
//     eprintln!(
//         "Loaded {} font fallbacks for Unicode support",
//         loaded_fonts.len()
//     );
// }
