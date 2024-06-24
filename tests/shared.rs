use quote::ToTokens;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use rs_script::cmd_args::{Cli, ProcFlags};
use rs_script::shared::{
    debug_timings, display_timings, escape_path_for_windows, Ast, BuildState, CargoManifest,
    Dependency, Feature, Package, Product, ScriptState, Workspace,
};

#[test]
fn test_ast_to_tokens() {
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
fn test_cargo_manifest_from_str() {
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

    let manifest: CargoManifest = toml::from_str(toml_str).unwrap();

    assert_eq!(manifest.package.name, "example");
    assert_eq!(manifest.package.version, "0.1.0");
    assert_eq!(manifest.package.edition, "2021");
    assert!(manifest.dependencies.is_some());
    assert_eq!(
        manifest.dependencies.unwrap().get("serde").unwrap(),
        &Dependency::Simple("1.0".to_string())
    );
    assert!(manifest.features.is_some());
    assert_eq!(
        manifest.features.unwrap().get("default").unwrap(),
        &vec![Feature::Simple("serde".to_string())]
    );
}

#[test]
fn test_cargo_manifest_display() {
    let manifest = CargoManifest {
        package: Package {
            name: "example".to_string(),
            version: "0.1.0".to_string(),
            edition: "2021".to_string(),
        },
        dependencies: Some(BTreeMap::from([(
            "serde".to_string(),
            Dependency::Simple("1.0".to_string()),
        )])),
        features: Some(BTreeMap::from([(
            "default".to_string(),
            vec![Feature::Simple("serde".to_string())],
        )])),
        workspace: Workspace::default(),
        bin: Vec::new(),
        lib: Some(Product {
            path: None,
            name: None,
            required_features: None,
            crate_type: vec!["cdylib".to_string()].into(),
        }),
    };

    let toml_str = manifest.to_string();
    let expected_toml_str = r#"
        [package]
        name = "example"
        version = "0.1.0"
        edition = "2021"

        [dependencies]
        serde = "1.0"

        [features]
        default = ["serde"]

        [workspace]

        [lib]
        crate-type=["cdylib"]
    "#;

    assert_eq!(
        toml_str.replace(" ", "").replace("\n", ""),
        expected_toml_str.replace(" ", "").replace("\n", "")
    );
}

#[test]
fn test_build_state_pre_configure() {
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
fn test_script_state_getters() {
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
fn test_debug_timings() {
    let start = Instant::now();
    debug_timings(&start, "test_process");
    // No direct assertion, this just ensures the function runs without panic
}

#[test]
fn test_display_timings() {
    let start = Instant::now();
    let proc_flags = ProcFlags::empty();
    display_timings(&start, "test_process", &proc_flags);
    // No direct assertion, this just ensures the function runs without panic
}

#[test]
fn test_escape_path_for_windows() {
    #[cfg(windows)]
    {
        let path = r"C:\path\to\file";
        let escaped_path = escape_path_for_windows(path);
        assert_eq!(escaped_path, r"C:\\path\\to\\file");
    }

    #[cfg(not(windows))]
    {
        let path = "/path/to/file";
        let escaped_path = escape_path_for_windows(path);
        assert_eq!(escaped_path, path);
    }
}
