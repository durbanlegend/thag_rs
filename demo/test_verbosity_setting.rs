/*[toml]
[dependencies]
thag_rs = { version = "0.2, thag-auto" }
*/

//: Test the new verbosity setting logic
//# Purpose: Demonstrate and test the improved verbosity setting API
//# Categories: debugging, testing

use thag_rs::{get_verbosity, init_verbosity, set_global_verbosity, set_verbosity, vprtln, V};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Verbosity Setting Logic ===\n");

    // Test 1: Original way (still works)
    println!("1. Testing original set_global_verbosity function:");
    set_global_verbosity(V::V);
    println!("   Set to verbose using set_global_verbosity(V::V)");
    println!("   Current verbosity: {:?}", get_verbosity());
    vprtln!(V::V, "   This verbose message should appear");

    // Test 2: New init_verbosity function
    println!("\n2. Testing new init_verbosity function:");
    init_verbosity(V::D)?;
    println!("   Set to debug using init_verbosity(V::D)");
    println!("   Current verbosity: {:?}", get_verbosity());
    vprtln!(V::V, "   This verbose message should appear");
    vprtln!(V::D, "   This debug message should appear");

    // Test 3: New set_verbosity! macro with string patterns
    println!("\n3. Testing set_verbosity! macro with keywords:");

    // Test quiet
    set_verbosity!(quiet);
    println!("   Set to quiet using set_verbosity!(quiet)");
    println!("   Current verbosity: {:?}", get_verbosity());
    vprtln!(V::Q, "   This quiet message should appear");
    vprtln!(V::N, "   This normal message should NOT appear");

    // Test normal
    set_verbosity!(normal);
    println!("   Set to normal using set_verbosity!(normal)");
    println!("   Current verbosity: {:?}", get_verbosity());
    vprtln!(V::N, "   This normal message should appear");

    // Test verbose
    set_verbosity!(verbose);
    println!("   Set to verbose using set_verbosity!(verbose)");
    println!("   Current verbosity: {:?}", get_verbosity());
    vprtln!(V::V, "   This verbose message should appear");

    // Test debug
    set_verbosity!(debug);
    println!("   Set to debug using set_verbosity!(debug)");
    println!("   Current verbosity: {:?}", get_verbosity());
    vprtln!(V::D, "   This debug message should appear");

    // Test 4: Macro with V constants
    println!("\n4. Testing set_verbosity! macro with V constants:");
    set_verbosity!(V::N);
    println!("   Set to normal using set_verbosity!(V::N)");
    println!("   Current verbosity: {:?}", get_verbosity());

    println!("\n=== All tests completed successfully! ===");
    println!("\n=== Usage Summary ===");
    println!("Old way:  set_global_verbosity(V::V);");
    println!("New ways:");
    println!("  - init_verbosity(V::V)?;");
    println!("  - set_verbosity!(verbose);");
    println!("  - set_verbosity!(debug);");
    println!("  - set_verbosity!(quiet);");
    println!("  - set_verbosity!(normal);");
    println!("  - set_verbosity!(V::V);");

    Ok(())
}
