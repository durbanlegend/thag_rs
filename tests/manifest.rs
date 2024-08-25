#[cfg(test)]
mod tests {
    use cargo_toml::{Edition, Manifest};
    use mockall::predicate::*;
    use serde_merge::omerge;
    use std::process::Output;
    use thag_rs::code_utils::infer_deps_from_ast;
    use thag_rs::manifest::{
        self, capture_dep, cargo_search, default_manifest_from_build_state, merge, search_deps,
        MockCommandRunner,
    };
    use thag_rs::{Ast, BuildState};

    // Set environment variables before running tests
    fn set_up() {
        std::env::set_var("TEST_ENV", "1");
        std::env::set_var("VISUAL", "cat");
        std::env::set_var("EDITOR", "cat");
    }

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn successful_exit_status() -> std::process::ExitStatus {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            std::process::ExitStatus::from_raw(0)
        }

        #[cfg(windows)]
        {
            use std::os::windows::process::ExitStatusExt;
            std::process::ExitStatus::from_raw(0)
        }
    }

    #[test]
    fn test_cargo_search_success() {
        set_up();
        let output = Output {
            status: successful_exit_status(),
            stdout: b"serde = \"1.0.203\"".to_vec(),
            stderr: Vec::new(),
        };

        let mut mock_runner = MockCommandRunner::new();
        let args: Vec<String> = vec![
            "search".to_string(),
            "serde".to_string(),
            "--limit".to_string(),
            "1".to_string(),
        ];

        mock_runner
            .expect_run_command()
            .with(eq("cargo"), eq(args))
            .returning(move |_, _| Ok(output.clone()));

        let result = cargo_search(&mock_runner, "serde");
        assert!(result.is_ok());
        let (name, version) = result.unwrap();
        assert_eq!(name, "serde");
        assert_eq!(version, "1.0.203");
    }

    #[test]
    fn test_capture_dep_valid() {
        set_up();
        let line = r#"serde = "1.0.104""#;
        let result = capture_dep(line);
        assert!(result.is_ok());
        let (name, version) = result.unwrap();
        assert_eq!(name, "serde");
        assert_eq!(version, "1.0.104");
    }

    #[test]
    fn test_capture_dep_invalid() {
        set_up();
        let line = r#"invalid format"#;
        let result = capture_dep(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_manifest() {
        set_up();
        let build_state = BuildState {
            source_stem: "example".to_string(),
            source_name: "example.rs".to_string(),
            target_dir_path: std::path::PathBuf::from("/tmp"),
            cargo_manifest: None,
            rs_manifest: None,
            ..Default::default()
        };

        let manifest = default_manifest_from_build_state(&build_state).unwrap();
        let package = manifest.package.expect("Problem unwrapping package");
        assert_eq!(package.name, "example");
        assert_eq!(package.version.get().unwrap(), &"0.0.1".to_string());
        assert!(matches!(package.edition.get().unwrap(), Edition::E2021));
    }

    // #[test]
    // fn test_cargo_search_success() {
    //     // This is a mocked test. In a real test environment, you should mock Command to simulate Cargo behavior.
    //     let output = r#"serde = "1.0.203""#;
    //     let mut search_command = NamedTempFile::new().unwrap();
    //     writeln!(search_command, "{}", output).unwrap();
    //     search_command.flush().unwrap();

    //     // Mocking Command::output
    //     let result = cargo_search("serde");
    //     assert!(result.is_ok());
    //     let (name, version) = result.unwrap();
    //     assert_eq!(name, "serde");
    //     assert_eq!(version, "1.0.203");
    // }

    #[test]
    fn test_merge_manifest() -> Result<(), Box<dyn std::error::Error>> {
        set_up();
        init_logger();

        let rs_toml_str = r##"[package]
name = "example"
version = "0.0.1"
edition = "2021"

[dependencies]
serde = "1.0"

[features]
default = ["serde"]

[patch.crates-io]
foo = { git = 'https://github.com/example/foo.git' }
bar = { path = 'my/local/bar' }

[workspace]

[[bin]]
name = "bin_name"
path = "bin_path"
"##;
        let rs_manifest = Some(Manifest::from_str(rs_toml_str).unwrap());
        // let alt_rs_manifest = Some(toml::from_str::<Manifest>(rs_toml_str).unwrap());
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

        let syntax_tree = None;
        let manifest = merge(&mut build_state, rs_source, &syntax_tree).unwrap();
        eprintln!("manifest.dependencies={:#?}", manifest.dependencies);
        assert!(manifest.dependencies.contains_key("serde_derive"));
        eprintln!("manifest.features={:#?}", manifest.features);
        assert!(manifest.features.contains_key("default"));

        // Temp compare old and new techniques

        let file = syn::parse_file(
            r#"
            extern crate foo;
            use bar::baz;
            use std::fmt;
            "#,
        )
        .unwrap();
        let ast = Ast::File(file);

        let default_manifest = manifest::default("example", "path/to/script").unwrap();
        // default_manifest.lib = None;

        let mut build_state = BuildState {
            source_stem: "example".to_string(),
            source_name: "example.rs".to_string(),
            target_dir_path: std::path::PathBuf::from("/tmp"),
            cargo_manifest: Some(default_manifest.clone()),
            rs_manifest: rs_manifest.clone(),
            ..Default::default()
        };

        let default_cargo_manifest = default_manifest_from_build_state(&build_state)?;

        let mut rs_manifest = if let Some(rs_manifest) = build_state.rs_manifest.as_mut() {
            rs_manifest.clone()
        } else {
            default_cargo_manifest.clone()
        };

        let rs_dep_map = &mut rs_manifest.dependencies;

        let rs_inferred_deps = infer_deps_from_ast(&ast);
        if !rs_inferred_deps.is_empty() {
            search_deps(rs_inferred_deps, rs_dep_map);
        }

        let trad_manifest = merge(&mut build_state, rs_source, &Some(ast))?;

        let rs_manifest = rs_manifest.clone();
        let build_state = BuildState {
            source_stem: "example".to_string(),
            source_name: "example.rs".to_string(),
            target_dir_path: std::path::PathBuf::from("/tmp"),
            cargo_manifest: Some(default_manifest.clone()),
            rs_manifest: Some(rs_manifest.clone()),
            ..Default::default()
        };

        // Merge the manifests
        let mut merged_manifest: Manifest =
            omerge(build_state.rs_manifest, build_state.cargo_manifest)?;

        // Ensure all `[[bin]]` sections have the edition set to E2021
        let bins = &mut merged_manifest.bin;
        for bin in bins {
            // eprintln!("Found bin.edition={:#?}", bin.edition);
            // Don't accept the default of E2015. This is the only way I can think of
            // to stop it defaulting to E2015 and then overriding the template value.
            if matches!(bin.edition, Edition::E2015) {
                bin.edition = cargo_toml::Edition::E2021;
            }
        }

        // eprintln!(
        //     "type_of merged_manifest.lib={}",
        //     type_of(&merged_manifest.lib.clone().unwrap())
        // );
        eprintln!("default_manifest={}", toml::to_string(&default_manifest)?);
        eprintln!("rs_manifest={}", toml::to_string(&rs_manifest)?);
        // eprintln!("alt_rs_manifest={}", toml::to_string(&alt_rs_manifest)?);
        eprintln!("merged_manifest={}", toml::to_string(&merged_manifest)?);
        eprintln!("trad_manifest={}", toml::to_string(&trad_manifest)?);
        assert_eq!(merged_manifest.package(), trad_manifest.package());
        assert_eq!(merged_manifest.workspace, trad_manifest.workspace);
        assert_eq!(merged_manifest.dependencies, trad_manifest.dependencies);
        assert_eq!(merged_manifest.features, trad_manifest.features);
        eprintln!(
            "merged_manifest.patch={:#?}",
            toml::to_string(&merged_manifest.patch)?
        );
        eprintln!(
            "trad_manifest.patch={:#?}",
            toml::to_string(&trad_manifest.patch)?
        );
        assert_eq!(merged_manifest.bin, trad_manifest.bin);
        assert_eq!(merged_manifest, trad_manifest);

        Ok(())
    }
}
