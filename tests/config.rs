#[cfg(test)]
mod tests {
    use std::{env::current_dir, path::PathBuf, sync::Arc};
    use tempfile::TempDir;
    use thag_rs::{
        config::{
            self, validate_config_format, Config, Dependencies, FeatureOverride, MockContext,
            RealContext,
        },
        load,
        logging::Verbosity,
        ColorSupport, Context, TermBgLuma, ThagResult,
    };

    #[cfg(feature = "simplelog")]
    use simplelog::{
        ColorChoice, CombinedLogger, LevelFilter, TermLogger, TerminalMode, WriteLogger,
    };

    #[cfg(feature = "simplelog")]
    use std::fs::File;

    #[cfg(feature = "simplelog")]
    use std::sync::OnceLock;

    #[cfg(feature = "simplelog")]
    use thag_rs::debug_log;

    #[cfg(feature = "simplelog")]
    static LOGGER: OnceLock<()> = OnceLock::new();

    fn init_logger() {
        // Choose between simplelog and env_logger based on compile feature
        #[cfg(feature = "simplelog")]
        LOGGER.get_or_init(|| {
            CombinedLogger::init(vec![
                TermLogger::new(
                    LevelFilter::Debug,
                    simplelog::Config::default(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(
                    LevelFilter::Debug,
                    simplelog::Config::default(),
                    File::create("app.log").unwrap(),
                ),
            ])
            .unwrap();
            debug_log!("Initialized simplelog");
        });

        #[cfg(not(feature = "simplelog"))] // This will use env_logger if simplelog is not active
        {
            let _ = env_logger::builder().is_test(true).try_init();
        }
    }

    // Set environment variables before running tests
    fn set_up() {
        static INIT: Once = Once::new();
        INIT.call_once(|| unsafe {
            std::env::set_var("TEST_ENV", "1");
            std::env::set_var("VISUAL", "cat");
            std::env::set_var("EDITOR", "cat");
        });
    }

    #[test]
    fn test_config_load_config_success() {
        set_up();
        init_logger();
        let current_dir = current_dir().unwrap();
        let config_path = current_dir.join("tests").join("assets").join("config.toml");

        let get_context = || -> Arc<dyn Context> {
            let context: Arc<dyn Context> = if std::env::var("TEST_ENV").is_ok() {
                let mut mock_context = MockContext::default();
                mock_context
                    .expect_get_config_path()
                    .return_const(config_path.clone());
                mock_context.expect_is_real().return_const(false);
                Arc::new(mock_context)
            } else {
                Arc::new(RealContext::new())
            };
            context
        };

        let config = load(&get_context())
            .expect("Failed to load config")
            .unwrap();

        assert_eq!(config.logging.default_verbosity, Verbosity::Normal);
        assert_eq!(config.styling.color_support, ColorSupport::default());
        assert_eq!(config.styling.term_bg_luma, TermBgLuma::default());
    }

    #[test]
    fn test_config_load_config_file_not_found() {
        set_up();
        init_logger();

        let get_context = || -> Arc<dyn Context> {
            let context: Arc<dyn Context> = if std::env::var("TEST_ENV").is_ok() {
                let mut mock_context = MockContext::default();
                mock_context
                    .expect_get_config_path()
                    .return_const(PathBuf::from("/non/existent/path/config.toml"));
                mock_context.expect_is_real().return_const(false);
                Arc::new(mock_context)
            } else {
                Arc::new(RealContext::new())
            };
            context
        };

        let config = load(&get_context()).expect("Failed to load config");

        assert!(
            config.is_some(),
            "Expected to load default config when config file is not found"
        );
    }

    #[test]
    fn test_config_load_config_invalid_format() {
        set_up();
        init_logger();
        let config_content = r#"invalid = toml"#;
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, config_content).expect("Failed to write to temp config file");

        let get_context = || -> Arc<dyn Context> {
            let context: Arc<dyn Context> = if std::env::var("TEST_ENV").is_ok() {
                let mut mock_context = MockContext::default();
                mock_context
                    .expect_get_config_path()
                    .return_const(config_path.clone());
                mock_context.expect_is_real().return_const(false);
                Arc::new(mock_context)
            } else {
                Arc::new(RealContext::new())
            };
            context
        };

        let config = load(&get_context());
        // It's expected to fall back to a partial config now.
        assert!(config.is_ok());
        eprintln!("config={config:#?}");
    }

    // #[ignore = "Opens file and expects human interaction"]
    #[test]
    fn test_config_edit_creates_config_file_if_not_exists() {
        set_up();
        init_logger();
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.toml");

        let mut mock_context = MockContext::default();
        mock_context
            .expect_get_config_path()
            .return_const(config_path.clone());
        mock_context.expect_is_real().return_const(false);

        let result = config::open(&mock_context).expect("Failed to edit config");

        assert!(config_path.exists(), "Config file should be created");
        let config_content =
            std::fs::read_to_string(&config_path).expect("Failed to read config file");
        // eprintln!("config_content={config_content}");
        #[cfg(target_os = "windows")]
        assert!(
            config_content.contains("[dependencies.feature_overrides.syn]"),
            "Config file should contain the expected `syn` crate overrides"
        );
        #[cfg(target_os = "windows")]
        assert!(
            config_content.contains("[dependencies.feature_overrides.syn]"),
            "Config file should contain the expected `syn` crate overrides"
        );
        #[cfg(target_os = "windows")]
        assert!(
            config_content.contains("visit-mut"),
            "Config file should contain the expected `syn` crate overrides"
        );
        #[cfg(not(target_os = "windows"))]
        assert!(
            config_content.contains(
                r#"[dependencies.feature_overrides.syn]
required_features = [
    "extra-traits",
    "fold",
    "full",
    "parsing",
    "visit",
    "visit-mut",
]
default_features = false"#
            ),
            "Config file should contain the expected `syn` crate overrides"
        );
        assert_eq!(result, Some(String::from("End of edit")));
    }

    fn create_test_config() -> Dependencies {
        set_up();
        init_logger();
        let mut config = Dependencies::default();
        config.exclude_unstable_features = true;
        config.exclude_std_feature = true;
        config.global_excluded_features = vec!["default".to_string(), "sqlite".to_string()];
        config.always_include_features = vec!["derive".to_string()];

        let rustyline_override = FeatureOverride {
            excluded_features: Some(vec!["with-sqlite-history".to_string()]),
            required_features: Some(vec!["with-file-history".to_string()]),
            default_features: Some(true),
            // alternative_features: vec![],
        };

        config
            .feature_overrides
            .insert("rustyline".to_string(), rustyline_override);
        config
    }

    #[test]
    fn test_config_filter_features_global_exclusions() {
        set_up();
        init_logger();
        let config = create_test_config();
        let features = &[
            "default".to_string(),
            "derive".to_string(),
            "std".to_string(),
        ];
        let filtered = config.filter_maximal_features("some_crate", features).0;
        assert!(!filtered.contains(&"default".to_string()));
        assert!(filtered.contains(&"derive".to_string())); // Always included
        assert!(!filtered.contains(&"std".to_string()));
        eprintln!("config={}", toml::to_string_pretty(&config).unwrap());
    }

    #[test]
    fn test_config_filter_features_crate_specific() {
        set_up();
        init_logger();
        let config = create_test_config();
        let features = &[
            "with-sqlite-history".to_string(),
            "derive".to_string(),
            "with-fuzzy".to_string(),
        ];
        let filtered = config.filter_maximal_features("rustyline", features).0;
        assert!(!filtered.contains(&"with-sqlite-history".to_string()));
        assert!(filtered.contains(&"with-file-history".to_string())); // Required
        assert!(filtered.contains(&"derive".to_string()));
        assert!(filtered.contains(&"with-fuzzy".to_string()));
    }

    #[test]
    fn test_config_should_include_feature() {
        set_up();
        init_logger();
        let config = create_test_config();
        assert!(!config.should_include_feature("default", "some_crate"));
        assert!(config.should_include_feature("derive", "some_crate"));
        assert!(!config.should_include_feature("with-sqlite-history", "rustyline"));
        assert!(config.should_include_feature("with-file-history", "rustyline"));
    }

    #[test]
    fn test_config_validation() {
        // Test valid config
        let config = r#"
            [dependencies]
            inference_level = "custom"
            exclude_unstable_features = true

            [dependencies.feature_overrides.clap]
            required_features = ["derive"]
            excluded_features = ["unstable"]
            default_features = true
        "#;

        assert!(validate_config_format(config).is_ok());

        // Test invalid config
        let invalid_config = r#"
            [dependencies]
            inference_level = "Custom"  # Wrong case

            [dependencies.feature_overrides.tokio]
            required_features = ["rt"]
            excluded_features = ["rt"]  # Conflict
        "#;

        assert!(validate_config_format(invalid_config).is_err());
    }

    #[test]
    fn test_config_load_or_create_default_when_config_doesnt_exist() -> ThagResult<()> {
        // Create a temporary directory that will be automatically cleaned up when the test ends
        let temp_dir = TempDir::new()?;
        let mut mock_context = MockContext::new();

        // Set up the config path inside the temporary directory
        let config_path = temp_dir.path().join("thag_rs").join("config.toml");

        mock_context
            .expect_get_config_path()
            .return_const(config_path.clone());

        mock_context.expect_is_real().return_const(false);

        let maybe_config = Config::load_or_create_default(&mock_context);

        assert!(
            maybe_config.is_ok(),
            "Expected Ok result, got {:?}",
            maybe_config
        );
        assert!(config_path.exists(), "Config file was not created");

        // TempDir will automatically clean up when it goes out of scope
        Ok(())
    }
}
