#![allow(dead_code)]
/// Demo of the enhanced DeriveDocComment proc macro that extracts documentation from multiple types.
///
/// This macro demonstrates advanced derive macro techniques by extracting documentation
/// comments from various Rust items and making them available at runtime:
/// - Enum variants with their documentation
/// - Struct fields with their documentation
/// - The items themselves (struct/enum level docs)
/// - Different struct types (named fields, tuple, unit)
//# Purpose: Demonstrate comprehensive documentation extraction across item types
//# Categories: technique, proc_macros, derive_macros, documentation, attribute_parsing

// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::DeriveDocComment;

/// Represents the current status of a task or operation
#[derive(Debug, DeriveDocComment)]
pub enum TaskStatus {
    /// The task is waiting to be started
    Pending,
    /// The task is currently being processed
    InProgress,
    /// The task completed successfully
    Completed,
    /// The task failed with an error
    Failed,
    /// The task was cancelled by the user
    Cancelled,
}

/// A comprehensive user configuration structure
///
/// This struct holds all the necessary configuration
/// for connecting to and managing a server.
#[derive(Debug, DeriveDocComment)]
pub struct ServerConfig {
    /// The hostname or IP address of the server
    pub host: String,
    /// The port number to connect to (typically 80, 443, 8080, etc.)
    pub port: u16,
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum number of concurrent connections allowed
    pub max_connections: u32,
    /// Whether SSL/TLS encryption is enabled
    pub enable_ssl: bool,
    /// Optional API key for authentication
    pub api_key: Option<String>,
}

/// A simple point in 3D space
#[derive(Debug, DeriveDocComment)]
pub struct Point3D(
    /// X coordinate
    f64,
    /// Y coordinate
    f64,
    /// Z coordinate
    f64,
);

/// A marker struct indicating successful initialization
#[derive(Debug, DeriveDocComment)]
pub struct InitComplete;

/// Different types of network protocols
#[derive(Debug, DeriveDocComment)]
pub enum Protocol {
    /// Hypertext Transfer Protocol
    Http,
    /// Secure HTTP over TLS/SSL
    Https,
    /// File Transfer Protocol
    Ftp {
        /// Whether to use passive mode
        passive: bool,
        /// Custom port if not using standard port 21
        custom_port: Option<u16>,
    },
    /// Simple Mail Transfer Protocol
    Smtp(
        /// SMTP server hostname
        String,
        /// SMTP port (usually 25, 465, or 587)
        u16,
    ),
}

fn main() {
    println!("📚 Enhanced Documentation Extraction Demo");
    println!("=========================================\n");

    // Example 1: Enum documentation
    println!("1. Enum variant documentation:");
    let status = TaskStatus::InProgress;
    println!("   Current status: {:?}", status);
    println!("   Documentation: \"{}\"", status.doc_comment());

    println!("\n   All available status docs:");
    for (variant, doc) in TaskStatus::all_docs() {
        println!("     {}: \"{}\"", variant, doc);
    }

    // Example 2: Struct field documentation
    println!("\n2. Struct field documentation:");
    let config = ServerConfig {
        host: "api.example.com".to_string(),
        port: 443,
        timeout_ms: 5000,
        max_connections: 100,
        enable_ssl: true,
        api_key: Some("secret_key_123".to_string()),
    };

    println!("   Config: {:?}", config);
    println!("\n   Field documentation lookup:");

    // Test individual field lookups
    let fields_to_check = ["host", "port", "timeout_ms", "enable_ssl", "nonexistent"];
    for field in &fields_to_check {
        match ServerConfig::field_doc(field) {
            Some(doc) => println!("     {}: \"{}\"", field, doc),
            None => println!("     {}: No documentation found", field),
        }
    }

    println!("\n   All field documentation:");
    for (name, type_name, doc) in ServerConfig::all_field_docs() {
        println!("     {} ({}): \"{}\"", name, type_name, doc);
    }

    println!("\n   Struct-level documentation:");
    println!("     \"{}\"", ServerConfig::struct_doc());

    // Example 3: Tuple struct documentation
    println!("\n3. Tuple struct documentation:");
    let point = Point3D(1.5, 2.7, 3.9);
    println!("   Point: {:?}", point);
    println!("   Documentation: \"{}\"", Point3D::struct_doc());

    // Example 4: Unit struct documentation
    println!("\n4. Unit struct documentation:");
    let init = InitComplete;
    println!("   Init marker: {:?}", init);
    println!("   Documentation: \"{}\"", InitComplete::struct_doc());

    // Example 5: Complex enum with different variant types
    println!("\n5. Complex enum documentation:");
    let protocols = vec![
        Protocol::Http,
        Protocol::Https,
        Protocol::Ftp {
            passive: true,
            custom_port: Some(2121),
        },
        Protocol::Smtp("mail.example.com".to_string(), 587),
    ];

    for (i, protocol) in protocols.iter().enumerate() {
        println!("   Protocol {}: {:?}", i + 1, protocol);
        println!("     Documentation: \"{}\"", protocol.doc_comment());
    }

    println!("\n   All protocol documentation:");
    for (variant, doc) in Protocol::all_docs() {
        println!("     {}: \"{}\"", variant, doc);
    }

    // Example 6: Demonstrating error handling
    println!("\n6. Error handling demonstration:");
    println!("   Looking up invalid field 'invalid_field':");
    match ServerConfig::field_doc("invalid_field") {
        Some(doc) => println!("     Found: \"{}\"", doc),
        None => println!("     ✅ Correctly returned None for invalid field"),
    }

    // Example 7: Runtime documentation access
    println!("\n7. Runtime documentation access:");
    println!("   This demonstrates how compile-time documentation");
    println!("   becomes available at runtime for:");
    println!("   - Help systems");
    println!("   - Configuration validators");
    println!("   - Auto-generated documentation");
    println!("   - API introspection");
    println!("   - Development tools");

    println!("\n🎉 Enhanced documentation extraction demo completed successfully!");
    println!("\nGenerated features demonstrated:");
    println!("  - Enum variant documentation extraction");
    println!("  - Struct field documentation with type information");
    println!("  - Struct-level documentation access");
    println!("  - Tuple and unit struct support");
    println!("  - Complex enum variant handling");
    println!("  - Runtime documentation lookup by name");
    println!("  - Proper error handling for missing docs");
    println!("  - Multi-line documentation support");
}
