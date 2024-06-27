#[cfg(test)]
mod tests {
    use cargo_toml::{Edition, Manifest};
    use mockall::predicate::*;
    use rs_script::manifest::{
        capture_dep, cargo_search, default_manifest_from_build_state, merge_manifest,
        MockCommandRunner,
    };
    use rs_script::BuildState;
    use std::process::Output;

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
        let line = r#"serde = "1.0.104""#;
        let result = capture_dep(line);
        assert!(result.is_ok());
        let (name, version) = result.unwrap();
        assert_eq!(name, "serde");
        assert_eq!(version, "1.0.104");
    }

    #[test]
    fn test_capture_dep_invalid() {
        let line = r#"invalid format"#;
        let result = capture_dep(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_manifest() {
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
    fn test_merge_manifest() {
        init_logger();

        let rs_manifest = Some(
            Manifest::from_str(
                r##"[package]
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

[lib]
crate_type = ["cdylib"]
"##,
            )
            .unwrap(),
        );

        let mut build_state = BuildState {
            source_stem: "example".to_string(),
            source_name: "example.rs".to_string(),
            target_dir_path: std::path::PathBuf::from("/tmp"),
            cargo_manifest: None,
            rs_manifest,
            ..Default::default()
        };

        let rs_source = r#"
        #[macro_use]
        extern crate serde_derive;
        "#;

        let syntax_tree = None;

        let manifest = merge_manifest(&mut build_state, rs_source, &syntax_tree).unwrap();
        eprintln!("manifest.dependencies={:#?}", manifest.dependencies);
        assert!(manifest.dependencies.contains_key("serde_derive"));
    }
}
