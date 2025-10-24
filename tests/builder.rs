#[cfg(test)]
use cargo_toml::Manifest;
use quote::ToTokens;
use std::sync::Once;
use std::{
    env::current_dir,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::Instant,
};
use thag_proc_macros::{safe_eprintln, safe_println};
use thag_rs::ast::Ast;
use thag_rs::builder::{build, display_timings, generate, run, BuildState, ScriptState};
use thag_rs::cmd_args::Cli;
use thag_rs::code_utils::{self};
use thag_rs::config::DependencyInference;
#[cfg(debug_assertions)]
use thag_rs::debug_timings;
use thag_rs::{escape_path_for_windows, execute, ProcFlags, EXECUTABLE_CACHE_SUBDIR, TMPDIR};

// Set environment variables before running tests
fn set_up() {
    static INIT: Once = Once::new();
    INIT.call_once(|| unsafe {
        std::env::set_var("TEST_ENV", "1");
        std::env::set_var("VISUAL", "cat");
        std::env::set_var("EDITOR", "cat");
    });
}

// Helper function to create a sample Cli structure
fn create_sample_cli(script: Option<String>) -> Cli {
    Cli {
        script,
        args: Vec::new(),
        expression: None,
        ..Default::default()
    }
}

// Helper function to create a sample BuildState structure.
// Requires the sample script to be in tests/assets.
fn create_sample_build_state(source_name: &str) -> BuildState {
    set_up();
    let source_stem: &str = source_name
        .strip_suffix(thag_rs::RS_SUFFIX)
        .expect("Problem stripping Rust suffix");
    let current_dir = std::env::current_dir().expect("Could not get current dir");
    let working_dir_path = current_dir.clone();
    let cargo_home_string = std::env::var("CARGO_HOME").unwrap_or_else(|_| ".".into());
    let cargo_home = PathBuf::from(cargo_home_string);
    let target_dir_path = TMPDIR.join("thag_rs").join(source_stem);
    fs::create_dir_all(target_dir_path.clone()).expect("Failed to create script directory");
    // Target path points to executable cache with the new shared target implementation
    let target_path = if cfg!(windows) {
        TMPDIR
            .join(EXECUTABLE_CACHE_SUBDIR)
            .join(format!("{}.exe", source_stem))
    } else {
        TMPDIR.join(EXECUTABLE_CACHE_SUBDIR).join(source_stem)
    };
    let cargo_toml_path = target_dir_path.join("Cargo.toml");
    let source_dir_path = current_dir.join("tests/assets");
    let source_path = current_dir.join("tests/assets").join(source_name);
    BuildState {
        working_dir_path,
        source_stem: source_stem.into(),
        source_name: source_name.into(),
        source_dir_path,
        source_path,
        cargo_home,
        target_dir_path,
        target_path,
        cargo_toml_path,
        rs_manifest: None,
        cargo_manifest: None,
        must_gen: true,
        must_build: true,
        build_from_orig_source: false,
        ast: None,
        crates_finder: None,
        metadata_finder: None,
        infer: DependencyInference::None,
        args: vec![],
        features: None,
        thag_auto_processed: false,
    }
}

#[test]
fn test_builder_execute_dynamic_script() {
    set_up();
    let mut cli = create_sample_cli(Some(
        "tests/assets/determine_if_known_type_trait_t.rs".to_string(),
    ));
    cli.force = true;
    let result = execute(&mut cli);
    assert!(result.is_ok());
}

// Any test of the iterator is problematic because reedline will panic
// with a message that the current cursor position can't be found.
// #[test]
// fn test_builder_execute_repl_script() {
// let mut cli = create_sample_cli(None);
// cli.iter = true;
//     let result = execute(cli);
//     assert!(result.is_ok());
// }

#[test]
fn test_builder_generate_source_file() {
    set_up();
    let script_name = "fib_fac_lite_t.rs";
    let mut build_state = create_sample_build_state(script_name);
    build_state.must_gen = true;
    build_state.must_build = true;
    build_state.cargo_toml_path = build_state.target_dir_path.clone().join("Cargo.toml");
    let cargo_toml = format!(
        r#"[package]
        name = "fib_fac_lite_t"
        version = "0.0.1"
        edition = "2021"

        [dependencies]
        itertools = "0.13.0"

        [features]

        [patch]

        [workspace]

        [[bin]]
        path = "{}/thag_rs/fib_fac_lite_t/fib_fac_lite_t.rs"
        name = "fib_fac_lite_t"
"#,
        escape_path_for_windows(TMPDIR.display().to_string().as_str())
    );
    let cargo_manifest = Manifest::from_str(&cargo_toml).expect("Could not parse manifest string");
    build_state.cargo_manifest = Some(cargo_manifest);

    let rs_source = code_utils::read_file_contents(&build_state.source_path)
        .expect("Error reading script contents");
    let proc_flags = ProcFlags::empty();
    let result = generate(&build_state, Some(&rs_source), &proc_flags);
    assert!(result.is_ok());
    assert!(build_state.target_dir_path.join(script_name).exists());
    assert!(build_state.cargo_toml_path.exists());
}

#[test]
fn test_builder_build_cargo_project() {
    set_up();
    let source_name = "bitflags_t.rs";
    let source_stem: &str = source_name
        .strip_suffix(thag_rs::RS_SUFFIX)
        .expect("Problem stripping Rust suffix");

    let current_dir = current_dir().expect("Could not get current dir");
    let source_path = current_dir.join("tests/assets").join(source_name);
    let cargo_home_str = std::env::var("CARGO_HOME").unwrap_or_else(|_| ".".into());
    let cargo_home = PathBuf::from(cargo_home_str);
    let target_dir_path = TMPDIR.join("thag_rs").join(source_stem);
    fs::create_dir_all(target_dir_path.clone()).expect("Failed to create script directory");
    let cargo_toml_path = target_dir_path.join("Cargo.toml");
    let cargo_toml = format!(
        r#"[package]
name = "bitflags_t"
version = "0.0.1"
edition = "2021"

[dependencies]
bitflags = "2.5.0"

[features]

[patch]

[workspace]

[[bin]]
path = "{}/thag_rs/bitflags_t/bitflags_t.rs"
name = "bitflags_t"
"#,
        escape_path_for_windows(TMPDIR.display().to_string().as_str())
    );

    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(cargo_toml_path.clone())
        .expect("Error creating Cargo.toml");

    let mut cargo_toml_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(cargo_toml_path.clone())
        .expect("Error opening Cargo.toml");

    cargo_toml_file
        .write_all(cargo_toml.as_bytes())
        .expect("error writing Cargo.toml");

    let target_rs_path = target_dir_path.join(source_name);

    let rs_source =
        code_utils::read_file_contents(&source_path).expect("Error reading script contents");
    let _source_file = code_utils::write_source(&target_rs_path, &rs_source)
        .expect("Problem writing source to target path");
    // safe_println!("source_file={source_file:#?}");

    let build_state = BuildState {
        working_dir_path: current_dir.clone(),
        source_stem: source_stem.into(),
        source_name: source_name.into(),
        source_dir_path: current_dir.join("tests/assets"),
        source_path,
        cargo_home,
        // With shared target implementation, target_path points to executable cache
        target_path: if cfg!(windows) {
            TMPDIR
                .join(EXECUTABLE_CACHE_SUBDIR)
                .join(format!("{}.exe", source_stem))
        } else {
            TMPDIR.join(EXECUTABLE_CACHE_SUBDIR).join(source_stem)
        },
        cargo_toml_path,
        target_dir_path,
        rs_manifest: None,
        cargo_manifest: None,
        must_gen: true,
        must_build: true,
        build_from_orig_source: false,
        ast: None,
        crates_finder: None,
        metadata_finder: None,
        infer: DependencyInference::None,
        args: vec![],
        features: None,
        thag_auto_processed: false,
    };
    dbg!(&build_state);
    let proc_flags = ProcFlags::empty();
    let result = build(&proc_flags, &build_state);
    assert!(result.is_ok());
}

#[test]
fn test_builder_run_script() {
    set_up();
    let source_name = "fib_fac_dashu_t.rs";
    let source_stem: &str = source_name
        .strip_suffix(thag_rs::RS_SUFFIX)
        .expect("Problem stripping Rust suffix");
    // With shared target implementation, executables are cached in EXECUTABLE_CACHE_SUBDIR
    let target_path = if cfg!(windows) {
        TMPDIR
            .join(EXECUTABLE_CACHE_SUBDIR)
            .join(format!("{}.exe", source_stem))
    } else {
        TMPDIR.join(EXECUTABLE_CACHE_SUBDIR).join(source_stem)
    };

    // Remove executable if it exists, and check
    let result = fs::remove_file(&target_path);
    safe_eprintln!("Result of fs::remove_file({target_path:?}): {result:?}");
    assert!(!target_path.exists());

    // Generate and build executable, and check it exists.
    let mut cli = create_sample_cli(Some("tests/assets/fib_fac_dashu_t.rs".to_string()));
    cli.generate = true;
    cli.build = true;
    let result = execute(&mut cli);
    assert!(result.is_ok());
    safe_println!("target_path={target_path:#?}");
    assert!(target_path.exists());

    // Finally, run it
    let cli = create_sample_cli(Some(format!("tests/assets/{source_name}")));
    let build_state = create_sample_build_state(source_name);
    dbg!(&build_state);
    let proc_flags = ProcFlags::empty();
    let result = run(&proc_flags, &cli.args, &build_state);
    assert!(result.is_ok());
}

#[test]
#[cfg(debug_assertions)]
fn test_builder_debug_timings() {
    set_up();
    let start = Instant::now();
    debug_timings(&start, "test_process");
    // No direct assertion, this just ensures the function runs without panic
}

#[test]
fn test_builder_display_timings() {
    set_up();
    let start = Instant::now();
    let proc_flags = ProcFlags::empty();
    display_timings(&start, "test_process", &proc_flags);
    // No direct assertion, this just ensures the function runs without panic
}

#[test]
fn test_builder_build_state_pre_configure() {
    set_up();
    let _ = env_logger::try_init();

    let proc_flags = ProcFlags::empty();
    let cli = Cli::default();
    let script = "tests/assets/fizz_buzz_t.rs";
    let script_state = ScriptState::Named {
        script: script.to_string(),
        script_dir_path: PathBuf::from(script),
    };

    let build_state = BuildState::pre_configure(&proc_flags, &cli, &script_state).unwrap();

    assert_eq!(build_state.source_stem, "fizz_buzz_t");
    assert_eq!(build_state.source_name, "fizz_buzz_t.rs");
    assert_eq!(
        build_state.source_dir_path,
        PathBuf::from(script)
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap()
    );
    assert_eq!(
        build_state.cargo_home,
        PathBuf::from(std::env::var("CARGO_HOME").unwrap())
    );
}

#[test]
fn test_builder_script_state_getters() {
    set_up();
    let anonymous_state = ScriptState::Anonymous;
    assert!(anonymous_state.get_script().is_none());
    assert!(anonymous_state.get_script_dir_path().is_none());

    let named_empty_state = ScriptState::NamedEmpty {
        script: "test_script".to_string(),
        script_dir_path: PathBuf::from("/path/to/scripts"),
    };
    assert_eq!(
        named_empty_state.get_script(),
        Some("test_script".to_string())
    );
    assert_eq!(
        named_empty_state.get_script_dir_path(),
        Some(PathBuf::from("/path/to/scripts"))
    );

    let named_state = ScriptState::Named {
        script: "test_script".to_string(),
        script_dir_path: PathBuf::from("/path/to/scripts"),
    };
    assert_eq!(named_state.get_script(), Some("test_script".to_string()));
    assert_eq!(
        named_state.get_script_dir_path(),
        Some(PathBuf::from("/path/to/scripts"))
    );
}

#[test]
fn test_builder_ast_to_tokens() {
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::parse_quote;

    set_up();

    let file: syn::File = parse_quote! {
        fn main() {
            safe_println!("Hello, world!");
        }
    };
    let expr: syn::Expr = parse_quote! {
        safe_println!("Hello, world!")
    };

    let ast_file = Ast::File(file);
    let ast_expr = Ast::Expr(expr);

    let mut tokens_file = TokenStream::new();
    ast_file.to_tokens(&mut tokens_file);
    let expected_file: TokenStream = quote! {
        fn main() {
            safe_println!("Hello, world!");
        }
    };
    assert_eq!(tokens_file.to_string(), expected_file.to_string());

    let mut tokens_expr = TokenStream::new();
    ast_expr.to_tokens(&mut tokens_expr);
    let expected_expr: TokenStream = quote! {
        safe_println!("Hello, world!")
    };
    assert_eq!(tokens_expr.to_string(), expected_expr.to_string());
}
