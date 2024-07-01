use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    // Get the OUT_DIR environment variable
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("generated_tests.rs");
    let mut file = fs::File::create(dest_path).expect("Failed to create generated_tests.rs");

    let demo_dir = Path::new("demo");
    assert!(
        demo_dir.exists() && demo_dir.is_dir(),
        "demo directory does not exist"
    );

    let skip_files_on_windows = [
        "factorial_main_rug.rs",
        "fib_big_clap_rug.rs",
        "fib_fac_rug.rs",
    ];

    let multimain = ["flume_async.rs", "flume_select.rs"];

    for entry in fs::read_dir(demo_dir).expect("Failed to read demo directory") {
        let entry = entry.expect("Failed to get directory entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let file_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .expect("Failed to get file name");

            // Skip files on Windows
            if cfg!(target_os = "windows") && skip_files_on_windows.contains(&file_name) {
                continue;
            }

            let test_name = file_name.replace(".", "_");

            writeln!(
                file,
                r#"
#[test]
fn test_{test_name}() {{
    {{
        use std::process::Command;
        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("-bgnq{more_options}")
            .arg("{file_path}")
            .output()
            .expect("Failed to execute command");

        if !output.status.success() {{
            panic!(
                "Failed to build file: {file_name}\nstdout: {{}}\nstderr: {{}}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }}
    }}
}}
"#,
                test_name = test_name,
                file_name = file_name,
                file_path = path.to_str().expect("Failed to get file path"),
                more_options = if multimain.contains(&file_name) {
                    "m"
                } else {
                    ""
                }
            )
            .expect("Failed to write test function");
        }
    }
}
