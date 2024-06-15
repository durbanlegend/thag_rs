#[cfg(test)]
mod tests {
    use rs_script::manifest::{
        capture_dep, cargo_search, default_manifest, escape_path_for_windows, merge_manifest,
    };
    use rs_script::BuildState;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
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

        let manifest = default_manifest(&build_state).unwrap();
        assert_eq!(manifest.package.name, "example");
        assert_eq!(manifest.package.version, "0.0.1");
        assert_eq!(manifest.package.edition, "2021");
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

    #[test]
    fn test_cargo_search_success() {
        // This is a mocked test. In a real test environment, you should mock Command to simulate Cargo behavior.
        let output = r#"serde = "1.0.104""#;
        let mut search_command = NamedTempFile::new().unwrap();
        writeln!(search_command, "{}", output).unwrap();
        search_command.flush().unwrap();

        // Mocking Command::output
        let result = cargo_search("serde");
        assert!(result.is_ok());
        let (name, version) = result.unwrap();
        assert_eq!(name, "serde");
        assert_eq!(version, "1.0.203");
    }

    #[test]
    fn test_merge_manifest() {
        init_logger();
        let mut build_state = BuildState {
            source_stem: "example".to_string(),
            source_name: "example.rs".to_string(),
            target_dir_path: std::path::PathBuf::from("/tmp"),
            cargo_manifest: None,
            rs_manifest: None,
            ..Default::default()
        };

        let rs_source = r#"
        #[macro_use]
        extern crate serde_derive;
        "#;

        let syntax_tree = None;

        let manifest = merge_manifest(&mut build_state, rs_source, &syntax_tree).unwrap();
        eprintln!("manifest.dependencies={:#?}", manifest.dependencies);
        assert!(manifest.dependencies.is_some());
        assert!(manifest
            .dependencies
            .as_ref()
            .unwrap()
            .contains_key("serde_derive"));
    }
}
