/*[toml]
[dependencies]
thag_demo_proc_macros = { version = "0.1, thag-auto" }
*/

#![allow(dead_code)]
/// Demo of the compile_time_assert function-like macro for compile-time validation.
///
/// This macro demonstrates function-like macro parsing with multiple parameters
/// and compile-time validation techniques. It generates assertions that are
/// checked at compile time, causing compilation to fail if conditions are not met.
//# Purpose: Demonstrate compile-time assertions and validation
//# Categories: technique, proc_macros, function_like_macros, compile_time, validation
use thag_demo_proc_macros::compile_time_assert;

// Example 1: Basic compile-time assertions that should pass
compile_time_assert!(1 + 1 == 2, "Basic arithmetic must work");
compile_time_assert!(true, "True should always be true");
compile_time_assert!(std::mem::size_of::<u8>() == 1, "u8 should be 1 byte");
compile_time_assert!(std::mem::size_of::<u32>() == 4, "u32 should be 4 bytes");

// Example 2: Type size assertions
compile_time_assert!(
    std::mem::size_of::<usize>() >= 4,
    "usize should be at least 4 bytes"
);
compile_time_assert!(
    std::mem::size_of::<*const u8>() == std::mem::size_of::<usize>(),
    "Pointer size should match usize"
);

// Example 3: Mathematical constants and expressions
compile_time_assert!(2_u32.pow(3) == 8, "2^3 should equal 8");
compile_time_assert!(10 % 3 == 1, "10 mod 3 should be 1");
compile_time_assert!(core::u8::MAX as u16 + 1 == 256, "u8::MAX + 1 should be 256");

// Example 4: Array and slice properties
const ARRAY_SIZE: usize = 10;
const TEST_ARRAY: [i32; ARRAY_SIZE] = [0; ARRAY_SIZE];
compile_time_assert!(
    TEST_ARRAY.len() == ARRAY_SIZE,
    "Array length should match constant"
);
compile_time_assert!(ARRAY_SIZE > 5, "Array should be large enough");

// Example 5: Configuration validation
const CONFIG_MAX_USERS: usize = 1000;
const CONFIG_BUFFER_SIZE: usize = 4096;
compile_time_assert!(CONFIG_MAX_USERS > 0, "Maximum users must be positive");
compile_time_assert!(
    CONFIG_BUFFER_SIZE.is_power_of_two(),
    "Buffer size should be power of 2"
);

// Example 6: Platform-specific assertions
#[cfg(target_pointer_width = "64")]
compile_time_assert!(
    std::mem::size_of::<usize>() == 8,
    "64-bit platforms should have 8-byte usize"
);

#[cfg(target_pointer_width = "32")]
compile_time_assert!(
    std::mem::size_of::<usize>() == 4,
    "32-bit platforms should have 4-byte usize"
);

// Example 7: Custom struct size validation
#[repr(C)]
struct Point {
    x: f32,
    y: f32,
}

compile_time_assert!(
    std::mem::size_of::<Point>() == 8,
    "Point should be 8 bytes (2 f32s)"
);
compile_time_assert!(
    std::mem::align_of::<Point>() == 4,
    "Point should be 4-byte aligned"
);

// Example 8: Enum discriminant validation
#[repr(u8)]
enum Status {
    Pending = 0,
    InProgress = 1,
    Completed = 2,
}

compile_time_assert!(
    std::mem::size_of::<Status>() == 1,
    "Status enum should be 1 byte"
);

// Example 9: Generic struct validation
struct Container<T> {
    data: T,
    count: usize,
}

type IntContainer = Container<i32>;
compile_time_assert!(
    std::mem::size_of::<IntContainer>()
        >= std::mem::size_of::<i32>() + std::mem::size_of::<usize>(),
    "Container size should be at least the sum of its fields"
);

fn main() {
    println!("⚡ Compile-time Assert Macro Demo");
    println!("=================================\n");

    println!("🎉 All compile-time assertions passed!");
    println!("If you're seeing this message, it means all the compile-time");
    println!("assertions in this file were successful.\n");

    println!("Successful assertions demonstrated:");
    println!("  ✅ Basic arithmetic: 1 + 1 == 2");
    println!("  ✅ Type sizes: sizeof(u8) == 1, sizeof(u32) == 4");
    println!("  ✅ Platform validation: pointer size checks");
    println!("  ✅ Mathematical expressions: 2^3 == 8, 10 % 3 == 1");
    println!("  ✅ Array properties: length validation");
    println!("  ✅ Configuration validation: positive values, power of 2");
    println!("  ✅ Custom struct sizes: Point struct = 8 bytes");
    println!("  ✅ Enum representation: Status enum = 1 byte");
    println!("  ✅ Generic struct validation: Container size calculation");

    println!("\n📋 Key Features Demonstrated:");
    println!("  • Compile-time evaluation of boolean expressions");
    println!("  • Custom error messages for failed assertions");
    println!("  • Zero runtime overhead (all checks at compile time)");
    println!("  • Type size and alignment validation");
    println!("  • Platform-specific conditional assertions");
    println!("  • Mathematical constant validation");
    println!("  • Configuration parameter validation");

    println!("\n🔧 Use Cases for compile_time_assert!:");
    println!("  • API contract validation");
    println!("  • Platform compatibility checks");
    println!("  • Memory layout validation");
    println!("  • Configuration parameter validation");
    println!("  • Mathematical constant verification");
    println!("  • Type system constraints");
    println!("  • Safety-critical system validation");

    // Runtime demonstrations
    println!("\n🧪 Runtime verification of compile-time checks:");
    println!("   Size of u8: {} bytes", std::mem::size_of::<u8>());
    println!("   Size of u32: {} bytes", std::mem::size_of::<u32>());
    println!("   Size of usize: {} bytes", std::mem::size_of::<usize>());
    println!("   Size of Point: {} bytes", std::mem::size_of::<Point>());
    println!("   Size of Status: {} bytes", std::mem::size_of::<Status>());
    println!("   Test array length: {}", TEST_ARRAY.len());
    println!(
        "   Buffer size is power of 2: {}",
        CONFIG_BUFFER_SIZE.is_power_of_two()
    );

    println!("\n✨ All assertions verified at both compile and runtime!");

    // Example of what would happen with a failing assertion:
    println!("\n❌ Example of failing assertion (commented out):");
    println!("   // compile_time_assert!(1 + 1 == 3, \"This would fail compilation\");");
    println!("   // If uncommented, this would prevent the program from compiling");
    println!("   // with the error message: \"This would fail compilation\"");
}
