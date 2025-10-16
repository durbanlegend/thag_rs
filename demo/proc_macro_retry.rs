/// Demo of the retry attribute macro that adds automatic retry logic to functions.
///
/// This macro demonstrates attribute macro parameter parsing and error handling
/// patterns by wrapping functions with retry logic. It automatically retries
/// failed function calls with configurable attempts and backoff delays.
//# Purpose: Demonstrate automatic retry logic with configurable parameters
//# Categories: technique, proc_macros, attribute_macros, error_handling, resilience

// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::retry;

/// Unreliable network operation that fails randomly
#[retry(expand)]
fn unreliable_network_call() -> Result<String, String> {
    println!("    Attempting network call...");

    // 70% chance of failure
    if rand::random::<f32>() < 0.7 {
        Err("Network timeout".to_string())
    } else {
        Ok("Data received successfully".to_string())
    }
}

/// Custom retry count - try 5 times
#[retry(5)]
fn flaky_database_query() -> Result<Vec<String>, String> {
    println!("    Querying database...");

    // 60% chance of failure
    if rand::random::<f32>() < 0.6 {
        Err("Database connection failed".to_string())
    } else {
        Ok(vec!["record1".to_string(), "record2".to_string()])
    }
}

/// File operation that might fail due to permissions
#[retry(3)]
fn read_config_file() -> Result<String, String> {
    println!("    Reading configuration file...");

    // 50% chance of failure
    if rand::random::<f32>() < 0.5 {
        Err("Permission denied".to_string())
    } else {
        Ok("config_data=value".to_string())
    }
}

/// API call with authentication that might fail
#[retry(4)]
fn authenticate_user(username: &str) -> Result<String, String> {
    println!("    Authenticating user: {}", username);

    // 40% chance of failure
    if rand::random::<f32>() < 0.4 {
        Err("Authentication failed".to_string())
    } else {
        Ok(format!("Token for {}", username))
    }
}

/// Resource allocation that might fail under load
#[retry(2)]
fn allocate_resource() -> Result<u32, String> {
    println!("    Allocating system resource...");

    // 80% chance of failure (resource contention)
    if rand::random::<f32>() < 0.8 {
        Err("Resource busy".to_string())
    } else {
        let resource_id = rand::random::<u32>() % 1000;
        Ok(resource_id)
    }
}

/// Service health check with retry
#[retry(6)]
fn health_check_service(service_name: &str) -> Result<String, String> {
    println!("    Checking health of service: {}", service_name);

    // Different failure rates for different services
    let failure_rate = match service_name {
        "critical-service" => 0.3, // More reliable
        "flaky-service" => 0.8,    // Very unreliable
        _ => 0.5,                  // Moderately reliable
    };

    if rand::random::<f32>() < failure_rate {
        Err(format!("Service {} is unhealthy", service_name))
    } else {
        Ok(format!("Service {} is healthy", service_name))
    }
}

fn demonstrate_operation<F, T>(operation_name: &str, operation: F) -> Option<T>
where
    F: FnOnce() -> Result<T, String>,
{
    println!("\n🔄 Testing: {}", operation_name);
    println!("   {}", "=".repeat(operation_name.len() + 11));

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)) {
        Ok(result) => match result {
            Ok(value) => {
                println!("   ✅ Operation succeeded!");
                Some(value)
            }
            Err(e) => {
                println!("   ❌ Operation failed after all retries: {}", e);
                None
            }
        },
        Err(_) => {
            println!("   💥 Operation panicked after all retries");
            None
        }
    }
}

fn main() {
    println!("🔁 Retry Attribute Macro Demo");
    println!("=============================\n");

    println!("This demo shows functions with retry logic that will automatically");
    println!("retry failed operations. Each function has different failure rates");
    println!("and retry counts to demonstrate various scenarios.\n");

    // Example 1: Default retry (3 attempts)
    let _result1 = demonstrate_operation("Unreliable Network Call (default 3 retries)", || {
        unreliable_network_call()
    });

    // Example 2: Custom retry count
    let _result2 = demonstrate_operation("Flaky Database Query (5 retries)", || {
        flaky_database_query()
    });

    // Example 3: File operations
    let _result3 =
        demonstrate_operation("Configuration File Read (3 retries)", || read_config_file());

    // Example 4: Authentication with parameters
    let _result4 = demonstrate_operation("User Authentication (4 retries)", || {
        authenticate_user("alice")
    });

    // Example 5: Resource allocation under contention
    let _result5 = demonstrate_operation("Resource Allocation (2 retries)", || allocate_resource());

    // Example 6: Multiple service health checks
    let services = vec!["critical-service", "flaky-service", "normal-service"];

    for service in services {
        let operation_name = format!("Health Check: {} (6 retries)", service);
        let _result = demonstrate_operation(&operation_name, || health_check_service(service));
    }

    // Example 7: Demonstrating retry patterns
    println!("\n📊 Retry Patterns Summary");
    println!("=========================");
    println!("   • Default retry count: 3 attempts");
    println!("   • Custom retry counts: 2-6 attempts demonstrated");
    println!("   • Automatic backoff: 100ms * attempt_number");
    println!("   • Progress reporting: Shows which attempt failed/succeeded");
    println!("   • Panic handling: Catches and retries panicked functions");
    println!("   • Final failure: Reports after all attempts exhausted");

    println!("\n🎯 Retry Strategy Details");
    println!("=========================");
    println!("   • Each retry has an increasing delay (100ms, 200ms, 300ms, ...)");
    println!("   • The macro catches both Result::Err and panics");
    println!("   • Progress is reported for each attempt");
    println!("   • Success is reported if retry succeeds before max attempts");
    println!("   • Functions with parameters work normally");

    println!("\n🎉 Retry attribute macro demo completed successfully!");
    println!("\nKey features demonstrated:");
    println!("  - Configurable retry counts with #[retry(N)] syntax");
    println!("  - Automatic backoff delays between attempts");
    println!("  - Panic catching and retry logic");
    println!("  - Progress reporting for retry attempts");
    println!("  - Works with any function signature");
    println!("  - Preserves original function behavior when successful");

    println!("\nUse cases for #[retry]:");
    println!("  - Network operations and API calls");
    println!("  - Database connection and query operations");
    println!("  - File I/O operations");
    println!("  - Resource allocation under contention");
    println!("  - External service integrations");
    println!("  - Microservice communication");
    println!("  - Cloud infrastructure operations");
}
