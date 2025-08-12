    use colors_transform::{Rgb, Color};

    fn main() {
        // Create an RGB color
        let original_color = Rgb::from(100.0, 150.0, 200.0); // Example: a shade of blue

        // Lighten the color by a certain percentage (e.g., 20%)
        let lighter_color = original_color.lighten(20.0);

        // Print the original and lighter colors
        println!("Original Color: {:?}", original_color);
        println!("Lighter Color: {:?}", lighter_color);

/*
        // You can also convert to HSL and manipulate the lightness component directly
        let hsl_color = original_color.to_hsl();
        let even_lighter_hsl = hsl_color.with_lightness(hsl_color.get_lightness() + 15.0); // Increase lightness by 15 units
        let even_lighter_rgb = even_lighter_hsl.to_rgb();
        println!("Even Lighter (via HSL): {:?}", even_lighter_rgb);
*/
    }
