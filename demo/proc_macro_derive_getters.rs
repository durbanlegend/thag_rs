/// Demo of the DeriveGetters proc macro that automatically generates getter methods.
///
/// This macro generates getter methods for all fields in a struct, returning references
/// to avoid unnecessary moves. It's a simpler but still useful teaching example that
/// demonstrates:
/// - Derive macro syntax and parsing
/// - Field iteration and type analysis
/// - Method generation with documentation
/// - Error handling for unsupported types
//# Purpose: Demonstrate automatic getter generation
//# Categories: technique, proc_macros, derive_macros

// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::DeriveGetters;

#[derive(Debug, DeriveGetters)]
pub struct Person {
    pub name: String,
    pub age: u32,
    pub email: Option<String>,
    pub active: bool,
}

#[derive(Debug, DeriveGetters)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub timeout_ms: u64,
    pub features: Vec<String>,
}

fn main() {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        email: Some("alice@example.com".to_string()),
        active: true,
    };

    // Use the generated getter methods
    println!("Name: {}", person.name());
    println!("Age: {}", person.age());
    println!("Email: {:?}", person.email());
    println!("Active: {}", person.active());

    let config = Config {
        host: "localhost".to_string(),
        port: 8080,
        timeout_ms: 5000,
        features: vec!["auth".to_string(), "logging".to_string()],
    };

    println!("\nConfig:");
    println!("Host: {}", config.host());
    println!("Port: {}", config.port());
    println!("Timeout: {}ms", config.timeout_ms());
    println!("Features: {:?}", config.features());
}
