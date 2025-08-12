/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["basic"] }
*/
/// Demo of generating a `thag_styling` theme from an image.
//# Purpose: Lighten and darken colors
//# Categories: color, styling, technique
use thag_styling::Style;

fn lighten(h: f32, s: f32, l: f32, amount: f32) -> (f32, f32, f32) {
    let new_l = (l + amount).clamp(0.0, 1.0);
    (h, s, new_l)
}

fn darken(h: f32, s: f32, l: f32, amount: f32) -> (f32, f32, f32) {
    let new_l = (l - amount).clamp(0.0, 1.0);
    (h, s, new_l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());

    let (r1, g1, b1) = match h_prime as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let m = l - c / 2.0;
    let (r, g, b) = (r1 + m, g1 + m, b1 + m);

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

fn main() {
    let hsl = (210.0, 0.5, 0.4); // hue, saturation, lightness

    let lighter_hsl = lighten(hsl.0, hsl.1, hsl.2, 0.2);
    let darker_hsl = darken(hsl.0, hsl.1, hsl.2, 0.2);

    let orig_rgb = hsl_to_rgb(hsl.0, hsl.1, hsl.2);
    let lighter_rgb = hsl_to_rgb(lighter_hsl.0, lighter_hsl.1, lighter_hsl.2);
    let darker_rgb = hsl_to_rgb(darker_hsl.0, darker_hsl.1, darker_hsl.2);

    println!(
        "{}",
        Style::new()
            .with_rgb([orig_rgb.0, orig_rgb.1, orig_rgb.2])
            .paint(format!(
                "original: {:?} => #{:02X}{:02X}{:02X}",
                hsl, orig_rgb.0, orig_rgb.1, orig_rgb.2
            ))
    );
    println!(
        "{}",
        Style::new()
            .with_rgb([lighter_rgb.0, lighter_rgb.1, lighter_rgb.2])
            .paint(format!(
                "lighter: {:?} => #{:02X}{:02X}{:02X}",
                hsl, lighter_rgb.0, lighter_rgb.1, lighter_rgb.2
            ))
    );
    println!(
        "{}",
        Style::new()
            .with_rgb([darker_rgb.0, darker_rgb.1, darker_rgb.2])
            .paint(format!(
                "darker: {:?} => #{:02X}{:02X}{:02X}",
                hsl, darker_rgb.0, darker_rgb.1, darker_rgb.2
            ))
    );
}
