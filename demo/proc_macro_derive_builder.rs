#![allow(clippy::uninlined_format_args)]
/// Demo of the `DeriveBuilder` proc macro that generates builder pattern implementations.
///
/// This macro demonstrates advanced derive macro techniques by generating a complete
/// builder pattern implementation including:
/// - A separate builder struct with optional fields
/// - Fluent API with method chaining
/// - Build-time validation with comprehensive error handling
/// - Default trait implementation
/// - Documentation generation
//# Purpose: Demonstrate builder pattern generation with validation
//# Categories: technique, proc_macros, derive_macros, builder_pattern
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::DeriveBuilder;

#[derive(Debug, DeriveBuilder)]
#[expand_macro]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_ms: u64,
    pub max_connections: u32,
    pub enable_ssl: bool,
}

#[derive(Debug, DeriveBuilder)]
#[expand_macro]
pub struct DatabaseConfig {
    pub url: String,
    pub username: String,
    pub password: String,
    pub pool_size: u32,
}

#[derive(Debug, DeriveBuilder)]
#[expand_macro]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub is_admin: bool,
}

fn main() -> Result<(), String> {
    println!("🏗️  Builder Pattern Demo");
    println!("======================\n");

    // Example 1: Complete server configuration
    println!("1. Building a complete ServerConfig:");
    let server_config = ServerConfig::builder()
        .host("localhost".to_string())
        .port(8080)
        .timeout_ms(5000)
        .max_connections(100)
        .enable_ssl(false)
        .build()?;

    println!("   {:?}", server_config);

    // Example 2: Database configuration with method chaining
    println!("\n2. Building DatabaseConfig with method chaining:");
    let db_config = DatabaseConfig::builder()
        .url("postgresql://localhost:5432/mydb".to_string())
        .username("admin".to_string())
        .password("secret123".to_string())
        .pool_size(10)
        .build()?;

    println!("   {:?}", db_config);

    // Example 3: User configuration
    println!("\n3. Building User object:");
    let user = User::builder()
        .id(12345)
        .name("Alice Johnson".to_string())
        .email("alice@example.com".to_string())
        .is_admin(true)
        .build()?;

    println!("   {:?}", user);

    // Example 4: Demonstrating error handling - missing field
    println!("\n4. Demonstrating validation - missing required field:");
    let incomplete_config = ServerConfig::builder()
        .host("localhost".to_string())
        .port(8080)
        // Missing timeout_ms, max_connections, enable_ssl
        .build();

    match incomplete_config {
        Ok(_) => println!("   Unexpected success!"),
        Err(e) => println!("   ✅ Caught expected error: {}", e),
    }

    // Example 5: Building with default builder
    println!("\n5. Using default builder constructor:");
    let config_from_default = ServerConfigBuilder::default()
        .host("0.0.0.0".to_string())
        .port(443)
        .timeout_ms(10000)
        .max_connections(200)
        .enable_ssl(true)
        .build()?;

    println!("   {:?}", config_from_default);

    // Example 6: Partial building and continuing
    println!("\n6. Partial building and continuing:");
    let partial_builder = User::builder().id(67890).name("Bob Smith".to_string());

    let complete_user = partial_builder
        .email("bob@example.com".to_string())
        .is_admin(false)
        .build()?;

    println!("   {:?}", complete_user);

    // Example 7: Demonstrating fluent API flexibility
    println!("\n7. Different ordering of method calls:");
    let flexible_config = DatabaseConfig::builder()
        .pool_size(5)
        .password("another_secret".to_string())
        .url("mysql://localhost:3306/testdb".to_string())
        .username("test_user".to_string())
        .build()?;

    println!("   {:?}", flexible_config);

    println!("\n🎉 Builder pattern demo completed successfully!");
    println!("\nGenerated features demonstrated:");
    println!("  - Fluent API with method chaining");
    println!("  - Build-time validation with error messages");
    println!("  - Default trait implementation");
    println!("  - Flexible field ordering");
    println!("  - Type safety and compile-time checking");

    Ok(())
}
