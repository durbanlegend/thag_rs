/*[toml]
[dependencies]
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["config", "simplelog"] }
*/

/// Test script to validate that proc macro examples work correctly.
/// This script runs a selection of proc macro examples to ensure they compile and execute properly.
//# Purpose: Test proc macro examples to ensure they work correctly
//# Categories: testing, proc_macros, tools
use std::{
    env,
    process::{Command, Stdio},
};
use thag_rs::{auto_help, cvprtln, help_system::check_help_and_exit, Role, V};

struct TestCase {
    name: &'static str,
    file: &'static str,
    description: &'static str,
}

const TEST_CASES: &[TestCase] = &[
    TestCase {
        name: "AnsiCodeDerive",
        file: "demo/proc_macro_ansi_code_derive.rs",
        description: "Derive macro for ANSI color enums",
    },
    TestCase {
        name: "DeriveBasic",
        file: "demo/proc_macro_derive_basic.rs",
        description: "Basic derive macro with constructor generation",
    },
    TestCase {
        name: "AttributeBasic",
        file: "demo/proc_macro_attribute_basic.rs",
        description: "Basic attribute macro demonstration",
    },
    TestCase {
        name: "ConstDemo",
        file: "demo/proc_macro_const_demo.rs",
        description: "Compile-time constant generation",
    },
    TestCase {
        name: "FunctionLikeBasic",
        file: "demo/proc_macro_functionlike_basic.rs",
        description: "Basic function-like macro",
    },
    TestCase {
        name: "RepeatDash",
        file: "demo/proc_macro_repeat_dash.rs",
        description: "Function-like macro for text generation",
    },
    TestCase {
        name: "StringConcat",
        file: "demo/proc_macro_string_concat.rs",
        description: "Compile-time string concatenation",
    },
    TestCase {
        name: "HostPortConst",
        file: "demo/proc_macro_host_port_const.rs",
        description: "Derive macro for network constants",
    },
    TestCase {
        name: "DeriveCustomModel",
        file: "demo/proc_macro_derive_custom_model.rs",
        description: "Advanced custom model derive macro",
    },
    TestCase {
        name: "LoadStaticMap",
        file: "demo/proc_macro_load_static_map.rs",
        description: "Embed directory contents as static maps",
    },
];

fn run_test_case(test_case: &TestCase, thag_dev_path: &str) -> Result<bool, String> {
    cvprtln!(
        Role::INFO,
        V::N,
        "Testing {}: {}",
        test_case.name,
        test_case.description
    );

    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--bin", "thag", "--", test_case.file])
        .env("THAG_DEV_PATH", thag_dev_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                cvprtln!(Role::SUCC, V::N, "✓ {} passed", test_case.name);
                Ok(true)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                cvprtln!(Role::ERR, V::N, "✗ {} failed", test_case.name);
                cvprtln!(Role::ERR, V::N, "Error: {}", stderr);
                Ok(false)
            }
        }
        Err(e) => {
            cvprtln!(
                Role::ERR,
                V::N,
                "✗ {} failed to execute: {}",
                test_case.name,
                e
            );
            Err(e.to_string())
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for help first
    let help = auto_help!("test_proc_macro_examples");
    check_help_and_exit(&help);

    // Get the current directory as THAG_DEV_PATH
    let current_dir = env::current_dir()?;
    let thag_dev_path = current_dir.to_string_lossy();

    cvprtln!(
        Role::INFO,
        V::N,
        "Testing proc macro examples with THAG_DEV_PATH={}",
        thag_dev_path
    );

    // Check if we're in the right directory
    if !current_dir.join("demo").exists() {
        cvprtln!(
            Role::ERR,
            V::N,
            "Error: demo directory not found. Please run from the thag_rs root directory."
        );
        std::process::exit(1);
    }

    if !current_dir.join("demo/proc_macros").exists() {
        cvprtln!(
            Role::ERR,
            V::N,
            "Error: demo/proc_macros directory not found."
        );
        std::process::exit(1);
    }

    cvprtln!(
        Role::INFO,
        V::N,
        "Running {} test cases...\n",
        TEST_CASES.len()
    );

    let mut passed = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for test_case in TEST_CASES {
        match run_test_case(test_case, &thag_dev_path) {
            Ok(true) => passed += 1,
            Ok(false) => failed += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("{}: {}", test_case.name, e));
            }
        }
    }

    cvprtln!(Role::INFO, V::N, "\n=== Test Results ===");
    cvprtln!(Role::SUCC, V::N, "Passed: {}", passed);

    if failed > 0 {
        cvprtln!(Role::ERR, V::N, "Failed: {}", failed);

        if !errors.is_empty() {
            cvprtln!(Role::ERR, V::N, "\nExecution errors:");
            for error in &errors {
                cvprtln!(Role::ERR, V::N, "  {}", error);
            }
        }
    }

    cvprtln!(
        Role::INFO,
        V::N,
        "Total: {} ({:.1}% success rate)",
        TEST_CASES.len(),
        (passed as f64 / TEST_CASES.len() as f64) * 100.0
    );

    if failed > 0 {
        cvprtln!(
            Role::WARN,
            V::N,
            "\nSome tests failed. This might be due to:"
        );
        cvprtln!(Role::WARN, V::N, "  - Missing dependencies in examples");
        cvprtln!(Role::WARN, V::N, "  - Compilation errors in proc macros");
        cvprtln!(Role::WARN, V::N, "  - Environment setup issues");
        cvprtln!(
            Role::WARN,
            V::N,
            "  - Run individual tests for more details"
        );
        std::process::exit(1);
    }

    cvprtln!(Role::SUCC, V::N, "\nAll tests passed! 🎉");
    Ok(())
}
