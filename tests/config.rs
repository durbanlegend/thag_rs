#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use thag_rs::{
        colors::{ColorSupport, TermTheme},
        config::{self, MockContext},
        load,
        logging::Verbosity,
    };

    #[test]
    fn test_config_load_config_success() {
        let config_content = r#"
            [logging]
            default_verbosity = "verbose"

            [colors]
            color_support = "ansi16"
            #term_theme = "dark"
        "#;
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, config_content).expect("Failed to write to temp config file");

        let mut mock_context = MockContext::default();
        mock_context
            .expect_get_config_path()
            .return_const(config_path.clone());
        mock_context.expect_is_real().return_const(false);

        let config = load(&mock_context).expect("Failed to load config");

        assert_eq!(config.logging.default_verbosity, Verbosity::Verbose);
        assert_eq!(config.colors.color_support, ColorSupport::Ansi16);
        assert_eq!(config.colors.term_theme, TermTheme::None);
    }

    #[test]
    fn test_config_load_config_file_not_found() {
        let mut mock_context = MockContext::default();
        mock_context
            .expect_get_config_path()
            .return_const(PathBuf::from("/non/existent/path/config.toml"));
        mock_context.expect_is_real().return_const(false);

        let config = load(&mock_context);
        assert!(
            config.is_none(),
            "Expected None when config file is not found"
        );
    }

    #[test]
    fn test_config_load_config_invalid_format() {
        let config_content = r#"invalid = toml"#;
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, config_content).expect("Failed to write to temp config file");

        let mut mock_context = MockContext::default();
        mock_context
            .expect_get_config_path()
            .return_const(config_path.clone());
        mock_context.expect_is_real().return_const(false);

        let config = load(&mock_context);
        assert!(
            config.is_none(),
            "Expected None when config file format is invalid"
        );
    }

    // #[ignore = "Opens file and expects human interaction"]
    #[test]
    fn test_config_edit_creates_config_file_if_not_exists() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.toml");

        let mut mock_context = MockContext::default();
        mock_context
            .expect_get_config_path()
            .return_const(config_path.clone());
        mock_context.expect_is_real().return_const(false);

        let result = config::edit(&mock_context).expect("Failed to edit config");

        assert!(config_path.exists(), "Config file should be created");
        let config_content =
            std::fs::read_to_string(&config_path).expect("Failed to read config file");
        assert!(
            config_content.contains("Please set up the config file as follows"),
            "Config file should contain the template text"
        );
        assert_eq!(result, Some(String::from("End of edit")));
    }
}
