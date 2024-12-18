use cargo_toml::{Dependency, Edition, Manifest, Product};
use quote::ToTokens;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;
use thag::manifest;

use thag::cmd_args::{Cli, ProcFlags};
#[cfg(debug_assertions)]
use thag::shared::debug_timings;
use thag::shared::{
    display_timings, escape_path_for_windows, should_filter_dependency, Ast, BuildState,
    ScriptState,
};

// Set environment variables before running tests
fn set_up() {
    std::env::set_var("TEST_ENV", "1");
    std::env::set_var("VISUAL", "cat");
    std::env::set_var("EDITOR", "cat");
}

#[test]
fn test_shared_ast_to_tokens() {
    set_up();
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::parse_quote;

    let file: syn::File = parse_quote! {
        fn main() {
            println!("Hello, world!");
        }
    };
    let expr: syn::Expr = parse_quote! {
        println!("Hello, world!")
    };

    let ast_file = Ast::File(file);
    let ast_expr = Ast::Expr(expr);

    let mut tokens_file = TokenStream::new();
    ast_file.to_tokens(&mut tokens_file);
    let expected_file: TokenStream = quote! {
        fn main() {
            println!("Hello, world!");
        }
    };
    assert_eq!(tokens_file.to_string(), expected_file.to_string());

    let mut tokens_expr = TokenStream::new();
    ast_expr.to_tokens(&mut tokens_expr);
    let expected_expr: TokenStream = quote! {
        println!("Hello, world!")
    };
    assert_eq!(tokens_expr.to_string(), expected_expr.to_string());
}

#[test]
fn test_shared_cargo_manifest_from_str() {
    set_up();
    let toml_str = r#"
        [package]
        name = "example"
        version = "0.1.0"
        edition = "2021"

        [dependencies]
        serde = "1.0"

        [features]
        default = ["serde"]
    "#;

    let manifest: Manifest = Manifest::from_str(toml_str).unwrap();

    let package = manifest.package.expect("Problem unwrapping package");
    assert_eq!(package.name, "example");
    assert_eq!(package.version.get().unwrap(), &"0.1.0".to_string());
    assert!(matches!(package.edition.get().unwrap(), Edition::E2021));
    assert_eq!(
        manifest.dependencies.get("serde").unwrap(),
        &Dependency::Simple("1.0".to_string())
    );

    println!(
        r#"manifest.features.get("default").unwrap()={:#?}"#,
        manifest.features.get("default").unwrap()
    );
    // assert_eq!(
    //     manifest.features.get("default").unwrap(),
    //     &vec![Feature::Simple("serde".to_string())]
    // );
}

#[test]
fn test_shared_cargo_manifest_display() {
    set_up();
    let mut manifest = manifest::default("example", "path/to/script").unwrap();

    manifest
        .dependencies
        .insert("serde".to_string(), Dependency::Simple("1.0".to_string()));
    manifest
        .features
        .insert("default".to_string(), vec!["serde".to_string()]);
    manifest.patch.insert(
        "a".to_string(),
        [("b".to_string(), Dependency::Simple("1.0".to_string()))]
            .iter()
            .cloned()
            .collect::<BTreeMap<String, Dependency>>(),
    );
    // manifest.workspace.insert(Workspace::<Value>::default());
    manifest.workspace = None;
    manifest.lib = Some(Product {
        path: None,
        name: None,
        test: true,
        doctest: true,
        bench: true,
        doc: true,
        plugin: false,
        proc_macro: false,
        harness: true,
        edition: Some(Edition::E2021),
        crate_type: vec!["cdylib".to_string()],
        required_features: Vec::<String>::new(),
    });
    println!("manifest={manifest:#?}");

    let toml_str = toml::to_string(&manifest).unwrap();
    let expected_toml_str = r#"[package]
name = "example"
version = "0.0.1"
edition = "2021"

[dependencies]
serde = "1.0"

[features]
default = ["serde"]

[patch.a]
b = "1.0"

[lib]
edition = "2021"
crate-type = ["cdylib"]
required-features = []

[[bin]]
path = "path/to/script"
name = "example"
edition = "2021"
required-features = []
"#;

    println!("toml_str={toml_str}");
    assert_eq!(
        toml_str.replace(" ", "").replace("\n", ""),
        expected_toml_str.replace(" ", "").replace("\n", "")
    );
}

#[test]
fn test_shared_build_state_pre_configure() {
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
fn test_shared_script_state_getters() {
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
#[cfg(debug_assertions)]
fn test_shared_debug_timings() {
    set_up();
    let start = Instant::now();
    debug_timings(&start, "test_process");
    // No direct assertion, this just ensures the function runs without panic
}

#[test]
fn test_shared_display_timings() {
    set_up();
    let start = Instant::now();
    let proc_flags = ProcFlags::empty();
    display_timings(&start, "test_process", &proc_flags);
    // No direct assertion, this just ensures the function runs without panic
}

#[test]
fn test_shared_escape_path_for_windows() {
    set_up();
    #[cfg(windows)]
    {
        let path = r"C:\path\to\file";
        let escaped_path = escape_path_for_windows(path);
        assert_eq!(escaped_path, r"C:/path/to/file");
    }

    #[cfg(not(windows))]
    {
        let path = "/path/to/file";
        let escaped_path = escape_path_for_windows(path);
        assert_eq!(escaped_path, path);
    }
}

#[test]
fn test_shared_dep_filter_numeric_primitives() {
    assert!(should_filter_dependency("f32"));
    assert!(should_filter_dependency("i64"));
    assert!(should_filter_dependency("usize"));
}

#[test]
fn test_shared_dep_filter_core_types() {
    assert!(should_filter_dependency("bool"));
    assert!(should_filter_dependency("str"));
}

#[test]
fn test_shared_dep_filter_keywords() {
    assert!(should_filter_dependency("self"));
    assert!(should_filter_dependency("super"));
    assert!(should_filter_dependency("crate"));
}

#[test]
fn test_shared_dep_filter_real_crates_not_filtered() {
    assert!(!should_filter_dependency("serde"));
    assert!(!should_filter_dependency("tokio"));
    assert!(!should_filter_dependency("rand"));
}

#[test]
fn test_shared_dep_filter_capitalized_names() {
    assert!(should_filter_dependency("String"));
    assert!(should_filter_dependency("Result"));
    assert!(should_filter_dependency("Option"));
}
