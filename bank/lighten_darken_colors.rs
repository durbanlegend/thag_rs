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

// fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
//     let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
//     let h_prime = h / 60.0;
//     let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());

//     let (r1, g1, b1) = match h_prime as u32 {
//         0 => (c, x, 0.0),
//         1 => (x, c, 0.0),
//         2 => (0.0, c, x),
//         3 => (0.0, x, c),
//         4 => (x, 0.0, c),
//         _ => (c, 0.0, x),
//     };

//     let m = l - c / 2.0;
//     let (r, g, b) = (r1 + m, g1 + m, b1 + m);

//     (
//         (r * 255.0).round() as u8,
//         (g * 255.0).round() as u8,
//         (b * 255.0).round() as u8,
//     )
// }

// // Helper: RGB -> HSL
// fn rgb_to_hsl(rgb: [u8; 3]) -> (f32, f32, f32) {
//     let r = rgb[0] as f32 / 255.0;
//     let g = rgb[1] as f32 / 255.0;
//     let b = rgb[2] as f32 / 255.0;

//     let max = r.max(g.max(b));
//     let min = r.min(g.min(b));
//     let l = (max + min) / 2.0;

//     if (max - min).abs() < f32::EPSILON {
//         (0.0, 0.0, l) // achromatic
//     } else {
//         let d = max - min;
//         let s = if l > 0.5 {
//             d / (2.0 - max - min)
//         } else {
//             d / (max + min)
//         };

//         let h = if (max - r).abs() < f32::EPSILON {
//             (g - b) / d + if g < b { 6.0 } else { 0.0 }
//         } else if (max - g).abs() < f32::EPSILON {
//             (b - r) / d + 2.0
//         } else {
//             (r - g) / d + 4.0
//         } / 6.0;

//         (h, s, l)
//     }
// }

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

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let l = (max + min) / 2.0;
    let s;
    let mut h;

    if delta == 0.0 {
        h = 0.0;
        s = 0.0;
    } else {
        s = if l > 0.5 {
            delta / (2.0 - max - min)
        } else {
            delta / (max + min)
        };

        h = if max == r {
            ((g - b) / delta) % 6.0
        } else if max == g {
            ((b - r) / delta) + 2.0
        } else {
            ((r - g) / delta) + 4.0
        } * 60.0;

        // Ensure hue is positive
        if h < 0.0 {
            h += 360.0;
        }
    }

    (h, s, l)
}

fn main() {
    // let hsl = (210.0, 0.5, 0.4); // hue, saturation, lightness
    let rgb = [96, 144, 216];
    let hsl = rgb_to_hsl(rgb[0], rgb[1], rgb[2]);

    println!(
        "{}",
        Style::new().with_rgb(rgb).paint(format!(
            "starter: {:?} => #{:02X}{:02X}{:02X}",
            hsl, rgb[0], rgb[1], rgb[2]
        ))
    );

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
