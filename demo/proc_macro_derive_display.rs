/*[toml]
[dependencies]
thag_demo_proc_macros = { version = "0.1, thag-auto" }
*/

/// Demo of the DeriveDisplay proc macro that generates Display trait implementations.
///
/// This macro demonstrates advanced trait implementation generation by automatically
/// creating Display implementations for various types:
/// - Structs with named fields
/// - Tuple structs
/// - Unit structs
/// - Enums with all variant types
/// - Proper formatting with separators and type-aware output
//# Purpose: Demonstrate automatic Display trait implementation generation
//# Categories: technique, proc_macros, derive_macros, trait_implementation
use thag_demo_proc_macros::DeriveDisplay;

#[derive(Debug, DeriveDisplay)]
#[expand_macro]
pub struct Person {
    pub name: String,
    pub age: u32,
    pub email: String,
    pub active: bool,
}

#[derive(Debug, DeriveDisplay)]
#[expand_macro]
pub struct Point(f64, f64, f64);

#[derive(Debug, DeriveDisplay)]
#[expand_macro]
pub struct UnitStruct;

#[derive(Debug, DeriveDisplay)]
#[expand_macro]
pub enum Status {
    Pending,
    InProgress { task_id: u32, progress: f32 },
    Completed(String),
    Failed { error_code: i32, message: String },
}

#[derive(Debug, DeriveDisplay)]
#[expand_macro]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub timeout: u64,
}

#[derive(Debug, DeriveDisplay)]
#[expand_macro]
pub enum Color {
    Red,
    Green,
    Blue,
    Rgb(u8, u8, u8),
    Hsl {
        hue: f32,
        saturation: f32,
        lightness: f32,
    },
}

fn main() {
    println!("🖨️  Display Trait Demo");
    println!("=====================\n");

    // Example 1: Struct with named fields
    println!("1. Struct with named fields:");
    let person = Person {
        name: "Alice Johnson".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
        active: true,
    };
    println!("   Debug:   {:?}", person);
    println!("   Display: {}", person);

    // Example 2: Tuple struct
    println!("\n2. Tuple struct:");
    let point = Point(1.5, 2.7, 3.9);
    println!("   Debug:   {:?}", point);
    println!("   Display: {}", point);

    // Example 3: Unit struct
    println!("\n3. Unit struct:");
    let unit = UnitStruct;
    println!("   Debug:   {:?}", unit);
    println!("   Display: {}", unit);

    // Example 4: Enum variants
    println!("\n4. Enum variants:");

    // Unit variant
    let status1 = Status::Pending;
    println!("   Unit variant:");
    println!("     Debug:   {:?}", status1);
    println!("     Display: {}", status1);

    // Struct variant
    let status2 = Status::InProgress {
        task_id: 12345,
        progress: 0.75,
    };
    println!("   Struct variant:");
    println!("     Debug:   {:?}", status2);
    println!("     Display: {}", status2);

    // Tuple variant
    let status3 = Status::Completed("Data processing finished".to_string());
    println!("   Tuple variant:");
    println!("     Debug:   {:?}", status3);
    println!("     Display: {}", status3);

    // Struct variant with multiple fields
    let status4 = Status::Failed {
        error_code: 404,
        message: "Resource not found".to_string(),
    };
    println!("   Complex struct variant:");
    println!("     Debug:   {:?}", status4);
    println!("     Display: {}", status4);

    // Example 5: Configuration object
    println!("\n5. Configuration object:");
    let config = Config {
        host: "localhost".to_string(),
        port: 8080,
        timeout: 5000,
    };
    println!("   Debug:   {:?}", config);
    println!("   Display: {}", config);

    // Example 6: Color enum with different variants
    println!("\n6. Color enum with various types:");

    let colors = vec![
        Color::Red,
        Color::Blue,
        Color::Rgb(255, 128, 0),
        Color::Hsl {
            hue: 240.0,
            saturation: 1.0,
            lightness: 0.5,
        },
    ];

    for (i, color) in colors.iter().enumerate() {
        println!("   Color {}:", i + 1);
        println!("     Debug:   {:?}", color);
        println!("     Display: {}", color);
    }

    // Example 7: Using in string formatting
    println!("\n7. Using in string interpolation:");
    println!("   Welcome, {}!", person);
    println!("   Server running at {}", config);
    println!("   Current status: {}", status2);
    println!("   Point coordinates: {}", point);

    // Example 8: Demonstrating difference from Debug
    println!("\n8. Comparing Display vs Debug output:");
    let complex_status = Status::InProgress {
        task_id: 99999,
        progress: 0.333333,
    };

    println!("   Debug format (developer-focused):");
    println!("     {:?}", complex_status);
    println!("   Display format (user-friendly):");
    println!("     {}", complex_status);

    println!("\n🎉 Display trait demo completed successfully!");
    println!("\nGenerated features demonstrated:");
    println!("  - Named field formatting with proper separators");
    println!("  - Tuple struct formatting");
    println!("  - Unit struct and variant handling");
    println!("  - Enum variant pattern matching");
    println!("  - Clean, readable output format");
    println!("  - Proper std::fmt::Display trait implementation");
    println!("  - String interpolation compatibility");
}
