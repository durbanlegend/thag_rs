/// Demo of the env_or_default function-like macro for compile-time environment variable access.
///
/// This macro demonstrates compile-time environment variable processing with fallback
/// defaults. It reads environment variables during compilation and generates string
/// literals, providing a zero-overhead configuration management pattern.
//# Purpose: Demonstrate compile-time environment variable access with defaults
//# Categories: technique, proc_macros, function_like_macros, configuration, environment

// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "./demo/proc_macros".
use thag_demo_proc_macros::env_or_default;

// Example 1: Database configuration with defaults
const DATABASE_URL: &str = env_or_default!("DATABASE_URL", "postgresql://localhost:5432/myapp");
const DATABASE_POOL_SIZE: &str = env_or_default!("DB_POOL_SIZE", "10");

// Example 2: Application configuration
const APP_NAME: &str = env_or_default!("APP_NAME", "Demo Application");
const APP_VERSION: &str = env_or_default!("APP_VERSION", "1.0.0");
const APP_ENVIRONMENT: &str = env_or_default!("APP_ENV", "development");

// Example 3: Server configuration
const SERVER_HOST: &str = env_or_default!("SERVER_HOST", "127.0.0.1");
const SERVER_PORT: &str = env_or_default!("PORT", "8080");
const SERVER_WORKERS: &str = env_or_default!("WORKERS", "4");

// Example 4: Feature flags and debugging
const DEBUG_MODE: &str = env_or_default!("DEBUG", "false");
const LOG_LEVEL: &str = env_or_default!("LOG_LEVEL", "info");
const ENABLE_METRICS: &str = env_or_default!("ENABLE_METRICS", "true");

// Example 5: External service configuration
const REDIS_URL: &str = env_or_default!("REDIS_URL", "redis://localhost:6379");
const API_BASE_URL: &str = env_or_default!("API_BASE_URL", "https://api.example.com");
const API_TIMEOUT: &str = env_or_default!("API_TIMEOUT_SECONDS", "30");

// Example 6: Build and deployment configuration
const BUILD_TARGET: &str = env_or_default!("BUILD_TARGET", "release");
const DEPLOY_ENVIRONMENT: &str = env_or_default!("DEPLOY_ENV", "staging");

fn main() {
    println!("Environment Variable Configuration Demo");
    println!("======================================\n");

    println!("This demo shows how env_or_default! resolves environment variables");
    println!("at compile time with fallback defaults when variables are not set.\n");

    // Database configuration
    println!("Database Configuration:");
    println!("  DATABASE_URL: {}", DATABASE_URL);
    println!("  DB_POOL_SIZE: {}", DATABASE_POOL_SIZE);

    // Application configuration
    println!("\nApplication Configuration:");
    println!("  APP_NAME: {}", APP_NAME);
    println!("  APP_VERSION: {}", APP_VERSION);
    println!("  APP_ENVIRONMENT: {}", APP_ENVIRONMENT);

    // Server configuration
    println!("\nServer Configuration:");
    println!("  SERVER_HOST: {}", SERVER_HOST);
    println!("  SERVER_PORT: {}", SERVER_PORT);
    println!("  SERVER_WORKERS: {}", SERVER_WORKERS);

    // Feature flags and debugging
    println!("\nFeature Flags & Debugging:");
    println!("  DEBUG_MODE: {}", DEBUG_MODE);
    println!("  LOG_LEVEL: {}", LOG_LEVEL);
    println!("  ENABLE_METRICS: {}", ENABLE_METRICS);

    // External services
    println!("\nExternal Service Configuration:");
    println!("  REDIS_URL: {}", REDIS_URL);
    println!("  API_BASE_URL: {}", API_BASE_URL);
    println!("  API_TIMEOUT: {}", API_TIMEOUT);

    // Build and deployment
    println!("\nBuild & Deployment Configuration:");
    println!("  BUILD_TARGET: {}", BUILD_TARGET);
    println!("  DEPLOY_ENVIRONMENT: {}", DEPLOY_ENVIRONMENT);

    // Demonstrate type conversion patterns
    println!("\nType Conversion Examples:");

    // Convert string values to appropriate types
    let db_pool_size: u32 = DATABASE_POOL_SIZE.parse().unwrap_or(10);
    let server_port: u16 = SERVER_PORT.parse().unwrap_or(8080);
    let debug_enabled: bool = DEBUG_MODE.parse().unwrap_or(false);
    let api_timeout: u64 = API_TIMEOUT.parse().unwrap_or(30);

    println!("  Parsed DB pool size: {} (type: u32)", db_pool_size);
    println!("  Parsed server port: {} (type: u16)", server_port);
    println!("  Parsed debug mode: {} (type: bool)", debug_enabled);
    println!("  Parsed API timeout: {} seconds (type: u64)", api_timeout);

    // Environment variable status
    println!("\nEnvironment Variable Status:");
    println!("  Values shown above were resolved at compile time");
    println!("  If environment variables were set during compilation,");
    println!("  those values were used; otherwise defaults were applied");

    println!("\nTo test with different values, set environment variables and recompile:");
    println!("  export DATABASE_URL=postgresql://prod-server:5432/app");
    println!("  export APP_ENV=production");
    println!("  export DEBUG=true");
    println!("  cargo run -- demo/proc_macro_env_or_default.rs");

    println!("\nKey features demonstrated:");
    println!("  • Compile-time environment variable resolution");
    println!("  • Automatic fallback to default values");
    println!("  • Zero runtime overhead");
    println!("  • Configuration management patterns");
    println!("  • Type conversion from string literals");

    println!("\nUse cases for env_or_default!:");
    println!("  • Application configuration");
    println!("  • Database connection strings");
    println!("  • API endpoints and timeouts");
    println!("  • Feature flags and debugging");
    println!("  • Build and deployment settings");
    println!("  • External service configuration");
}
