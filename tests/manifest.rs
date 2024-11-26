#[cfg(test)]
mod tests {
    use cargo_toml::{Dependency, Edition, Manifest};
    use thag_rs::manifest::{capture_dep, cargo_search, configure_default, merge};
    use thag_rs::BuildState;

    // Set environment variables before running tests
    fn set_up() {
        std::env::set_var("TEST_ENV", "1");
        std::env::set_var("VISUAL", "cat");
        std::env::set_var("EDITOR", "cat");
    }

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_manifest_cargo_search_success() {
        set_up();
        let option = cargo_search("serde");
        assert!(option.is_some());
        let (name, version) = option.unwrap();
        assert_eq!(name, "serde");
        assert!(version.starts_with("1.0."));
        assert!(version.as_str() >= "1.0.215");
    }

    #[test]
    fn test_manifest_capture_dep_valid() {
        set_up();
        let line = r#"serde = "1.0.104""#;
        let result = capture_dep(line);
        assert!(result.is_ok());
        let (name, version) = result.unwrap();
        assert_eq!(name, "serde");
        assert_eq!(version, "1.0.104");
    }

    #[test]
    fn test_manifest_capture_dep_invalid() {
        set_up();
        let line = r#"invalid format"#;
        let result = capture_dep(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_default_manifest() {
        set_up();
        let build_state = BuildState {
            source_stem: "example".to_string(),
            source_name: "example.rs".to_string(),
            target_dir_path: std::path::PathBuf::from("/tmp"),
            cargo_manifest: None,
            rs_manifest: None,
            ..Default::default()
        };

        let manifest = configure_default(&build_state).unwrap();
        let package = manifest.package.expect("Problem unwrapping package");
        assert_eq!(package.name, "example");
        assert_eq!(package.version.get().unwrap(), &"0.0.1".to_string());
        assert!(matches!(package.edition.get().unwrap(), Edition::E2021));
    }

    #[test]
    fn test_manifest_merge_manifest() -> Result<(), Box<dyn std::error::Error>> {
        set_up();
        init_logger();

        let rs_toml_str = r##"[package]
    name = "toml_block_name"
    version = "0.0.1"
    edition = "2021"

    [dependencies]
    toml_block_dep = "1.0"

    [features]
    default = ["toml_block_default_feature"]

    [patch.crates-io]
    toml_block_foo = { git = 'https://github.com/example/foo.git' }
    toml_block_bar = { path = 'my/local/bar' }

    [workspace]

    [[bin]]
    name = "toml_block_bin_name"
    path = "toml_block_bin_path"
    "##;
        let rs_manifest = Some(Manifest::from_str(rs_toml_str).unwrap());
        let mut build_state = BuildState {
            source_stem: "example".to_string(),
            source_name: "example.rs".to_string(),
            target_dir_path: std::path::PathBuf::from("/tmp"),
            cargo_manifest: None,
            rs_manifest: rs_manifest.clone(),
            ..Default::default()
        };

        let rs_source = r#"
        #[macro_use]
        extern crate serde_derive;
        "#;

        merge(&mut build_state, rs_source)?;

        eprintln!("merged manifest={:#?}", build_state.cargo_manifest);

        if let Some(ref manifest) = build_state.cargo_manifest {
            assert_eq!(manifest.package().name(), "toml_block_name");
            assert_eq!(manifest.package().edition(), Edition::E2021);

            assert!(manifest.dependencies.contains_key("serde_derive"));
            assert_eq!(
                manifest.dependencies["toml_block_dep"],
                Dependency::Simple("1.0".to_string())
            );

            assert!(manifest.features.contains_key("default"));
            assert_eq!(
                manifest.features["default"],
                vec!["toml_block_default_feature"]
            );

            // Pattern match to handle `Dependency::Detailed`
            if let Some(Dependency::Detailed(dep)) =
                manifest.patch["crates-io"].get("toml_block_bar")
            {
                assert_eq!(dep.path.as_deref(), Some("my/local/bar"));
            } else {
                panic!("Expected toml_block_bar to be a Detailed dependency");
            }

            if let Some(Dependency::Detailed(dep)) =
                manifest.patch["crates-io"].get("toml_block_foo")
            {
                assert_eq!(
                    dep.git.as_deref(),
                    Some("https://github.com/example/foo.git")
                );
            } else {
                panic!("Expected toml_block_foo to be a Detailed dependency");
            }
        }

        Ok(())
    }

    #[test]
    fn test_manifest_cargo_search_valid_crate() {
        let result = cargo_search("serde");
        assert!(result.is_some());
        let (name, version) = result.unwrap();
        assert_eq!(name, "serde");
        assert!(version.starts_with("1.")); // Version check is less strict
    }

    #[test]
    fn test_manifest_cargo_search_nonexistent_crate() {
        let result = cargo_search("definitely_not_a_real_crate_name");
        assert!(result.is_none());
    }

    #[test]
    fn test_manifest_cargo_search_hyphenated() {
        let result = cargo_search("nu_ansi_term");
        assert!(result.is_some());
        let (name, version) = result.unwrap();
        assert_eq!(name, "nu-ansi-term");
        assert!(version.starts_with("0.")); // Version check is less strict
        assert!(version.as_str() >= "0.50.1");
    }
}
