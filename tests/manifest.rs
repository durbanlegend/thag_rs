#[cfg(test)]
mod tests {
    use cargo_toml::{Dependency, Edition, Manifest};
    use semver::Version;
    #[cfg(feature = "simplelog")]
    use simplelog::{
        ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
    };
    use std::path::PathBuf;
    use std::time::Instant;
    #[cfg(feature = "simplelog")]
    use std::{fs::File, sync::OnceLock};
    use thag_rs::{
        code_utils::{self, to_ast},
        config::{Dependencies, FeatureOverride},
        debug_log,
        manifest::{capture_dep, cargo_lookup, configure_default, merge},
        shared::{find_crates, find_metadata},
        BuildState,
    };

    // Set environment variables before running tests
    fn set_up() {
        init_logger();
        std::env::set_var("TEST_ENV", "1");
        std::env::set_var("VISUAL", "cat");
        std::env::set_var("EDITOR", "cat");
    }

    #[cfg(feature = "simplelog")]
    static LOGGER: OnceLock<()> = OnceLock::new();

    fn init_logger() {
        // Choose between simplelog and env_logger based on compile feature
        #[cfg(feature = "simplelog")]
        LOGGER.get_or_init(|| {
            CombinedLogger::init(vec![
                TermLogger::new(
                    LevelFilter::Debug,
                    Config::default(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(
                    LevelFilter::Debug,
                    Config::default(),
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

    #[test]
    fn test_manifest_cargo_lookup_success() {
        set_up();
        let option = cargo_lookup("serde");
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

        // eprintln!("merged manifest={:#?}", build_state.cargo_manifest);

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
    fn test_manifest_search_valid_crate() {
        set_up();
        let result = cargo_lookup("serde");
        assert!(result.is_some());
        let (name, version) = result.unwrap();
        assert_eq!(name, "serde");
        assert!(Version::parse(&version).unwrap().pre.is_empty());
    }

    #[test]
    fn test_manifest_cargo_lookup_hyphenated() {
        set_up();
        let result = cargo_lookup("nu_ansi_term");
        assert!(result.is_some());
        let (name, version) = result.unwrap();
        assert_eq!(name, "nu-ansi-term");
        assert!(Version::parse(&version).unwrap().pre.is_empty());
    }

    #[test]
    fn test_manifest_cargo_lookup_nonexistent_crate() {
        set_up();
        let result = cargo_lookup("definitely_not_a_real_crate_name");
        assert!(result.is_none());
    }

    fn setup_build_state(source: &str) -> BuildState {
        set_up();
        let mut build_state = BuildState {
            source_path: PathBuf::from("dummy_test.rs"),
            source_stem: String::from("dummy_test"),
            ast: None,
            crates_finder: None,
            metadata_finder: None,
            cargo_manifest: None,
            rs_manifest: None,
            build_from_orig_source: false,
            ..Default::default()
        };

        let source_path_string = build_state.source_path.to_string_lossy();

        if build_state.ast.is_none() {
            build_state.ast = to_ast(&source_path_string, source);
        }

        if let Some(ref ast) = build_state.ast {
            build_state.crates_finder = Some(find_crates(ast));
            build_state.metadata_finder = Some(find_metadata(ast));
        }

        let rs_manifest: Manifest =
            { code_utils::extract_manifest(&source, Instant::now()) }.unwrap();

        // debug_log!("rs_manifest={rs_manifest:#?}");

        // eprintln!("rs_source={source}");
        if build_state.rs_manifest.is_none() {
            build_state.rs_manifest = Some(rs_manifest);
        }

        build_state
    }

    #[test]
    fn test_manifest_analyze_type_annotations() {
        set_up();
        let source = r#"
            struct MyStruct {
                client: reqwest::Client,
                pool: sqlx::PgPool,
            }

            fn process(data: serde_json::Value) -> anyhow::Result<()> {
                let cache: redis::Client = redis::Client::new();
                Ok(())
            }
        "#;

        let mut build_state = setup_build_state(source);

        //         eprintln!(
        //             r#"In test_manifest_analyze_type_annotations: build_state.crates_finder ={:#?}
        // build_state.metadata_finder={:#?}"#,
        //             build_state.crates_finder, build_state.metadata_finder
        //         );
        merge(&mut build_state, source).unwrap();

        let manifest = build_state.cargo_manifest.unwrap();
        // eprintln!(
        //     "In test_manifest_analyze_type_annotations: source={source}\ndeps={:#?}",
        //     manifest.dependencies
        // );
        assert!(manifest.dependencies.contains_key("reqwest"));
        assert!(manifest.dependencies.contains_key("sqlx"));
        assert!(manifest.dependencies.contains_key("serde_json"));
        assert!(manifest.dependencies.contains_key("anyhow"));
        assert!(manifest.dependencies.contains_key("redis"));
    }

    #[test]
    fn test_manifest_analyze_expr_paths() {
        set_up();
        let source = r#"
            fn main() {
                // Should detect
                let client = reqwest::Client::new();
                let json = serde_json::json!({});

                // Should not detect (single segment)
                let response = client.get();
                let data = json.to_string();
            }
        "#;

        let mut build_state = setup_build_state(source);
        //         eprintln!(
        //             r#"In test_manifest_analyze_expr_paths: build_state.crates_finder ={:#?}
        // build_state.metadata_finder={:#?}"#,
        //             build_state.crates_finder, build_state.metadata_finder
        //         );
        merge(&mut build_state, source).unwrap();
        let manifest = build_state.cargo_manifest.unwrap();
        // eprintln!(
        //     "In test_manifest_analyze_expr_paths: source={source}\ndeps={:#?}",
        //     manifest.dependencies
        // );

        assert!(manifest.dependencies.contains_key("reqwest"));
        assert!(manifest.dependencies.contains_key("serde_json"));
        assert_eq!(manifest.dependencies.len(), 2);
    }

    #[test]
    fn test_manifest_analyze_complex_paths() {
        set_up();
        let source = r#"
            use tokio;

            async fn process() -> Result<(), Box<dyn std::error::Error>> {
                // Multi-segment type annotation
                let handle: tokio::task::JoinHandle<()> = tokio::spawn(async {
                    // Multi-segment function call
                    let time = chrono::Utc::now();
                    println!("Time: {}", time);
                });

                // Single segment variable (should not detect 'handle')
                handle.await?;
                Ok(())
            }
        "#;

        let mut build_state = setup_build_state(source);
        //         eprintln!(
        //             r#"In test_manifest_analyze_complex_paths: build_state.crates_finder = {:#?}
        // build_state.metadata_finder={:#?}"#,
        //             build_state.crates_finder, build_state.metadata_finder
        //         );
        merge(&mut build_state, source).unwrap();
        let manifest = build_state.cargo_manifest.unwrap();
        // eprintln!(
        //     "In test_manifest_analyze_complex_paths: source={source}\ndeps={:?}",
        //     manifest.dependencies
        // );

        assert!(manifest.dependencies.contains_key("tokio"));
        assert!(manifest.dependencies.contains_key("chrono"));
        assert!(!manifest.dependencies.contains_key("handle"));
    }

    #[test]
    fn test_manifest_analyze_macros() {
        set_up();
        let source = r#"
            fn main() {
                let json = serde_json::json!({ "key": "value" });
                let query = sqlx::query!("SELECT * FROM users");
                let sql = diesel::sql_query("SELECT 1");
            }
        "#;

        let mut build_state = setup_build_state(source);
        merge(&mut build_state, source).unwrap();

        let manifest = build_state.cargo_manifest.unwrap();
        assert!(manifest.dependencies.contains_key("serde_json"));
        assert!(manifest.dependencies.contains_key("sqlx"));
        assert!(manifest.dependencies.contains_key("diesel"));
    }

    #[test]
    fn test_manifest_analyze_traits_and_types() {
        set_up();
        let source = r#"
            use tokio;

            struct MyStream;

            impl tokio::io::AsyncRead for MyStream {
                type Error = diesel::result::Error;

                async fn read(&mut self) -> Result<(), Self::Error> {
                    Ok(())
                }
            }

            fn process<T: serde::de::DeserializeOwned>(data: T) {
                // ...
            }
        "#;

        let mut build_state = setup_build_state(source);
        merge(&mut build_state, source).unwrap();

        let manifest = build_state.cargo_manifest.unwrap();
        assert!(manifest.dependencies.contains_key("tokio"));
        assert!(manifest.dependencies.contains_key("diesel"));
        assert!(manifest.dependencies.contains_key("serde"));
    }

    #[test]
    fn test_manifest_infer_deps_case() {
        set_up();
        let source = r#"
            use clap::{CommandFactory, Parser};
            use console::style;
            use rustyline::DefaultEditor;
            // use shlex;
            use std::error::Error;
            use std::str::FromStr;
            use strum::{EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

            #[derive(Debug, Parser, EnumIter, EnumString, IntoStaticStr)]
            #[command(name = "", disable_help_flag = true, disable_help_subcommand = true)] // Disable automatic help subcommand and flag
            #[strum(serialize_all = "kebab-case")]
            enum LoopCommand {
                /// Evaluate an expression. Enclose complex expressions in braces {}.
                Eval,
                /// Enter, paste or modify your code
                Edit,
                /// Enter, paste or modify the generated Cargo.toml file your code
                Toml,
                /// List generated files
                List,
                /// Delete generated files
                Delete,
                /// Exit REPL
                Quit,
                /// Show help information
                Help,
            }

            impl LoopCommand {
                fn print_help() {
                    let mut command = LoopCommand::command();
                    let help_message = command.render_help();
                    println!("{}", help_message);
                }
            }

            fn main() -> Result<(), Box<dyn Error>> {
                let mut rl = DefaultEditor::new().unwrap();

                let cmd_vec = LoopCommand::iter()
                    .map(<LoopCommand as Into<&'static str>>::into)
                    .map(String::from)
                    .collect::<Vec<String>>();
                let cmd_list =
                    "Enter full or partial match of one of the following: ".to_owned() + &cmd_vec.join(", ");

                println!("{cmd_list}");
                loop {
                    let line = match rl.readline(">> ") {
                        Ok(x) => x,
                        Err(e) => match e {
                            rustyline::error::ReadlineError::Eof
                            | rustyline::error::ReadlineError::Interrupted => break,
                            rustyline::error::ReadlineError::WindowResized => continue,
                            _ => panic!("Error in read line: {e:?}"),
                        },
                    };
                    if line.trim().is_empty() {
                        continue;
                    }
                    _ = rl.add_history_entry(line.as_str());
                    let command = match shlex::split(&line) {
                        Some(split) => {
                            // eprintln!("split={split:?}");
                            let mut matches = 0;
                            let first_word = split[0].as_str();
                            let mut cmd = String::new();
                            for key in cmd_vec.iter() {
                                if key.starts_with(first_word) {
                                    matches += 1;
                                    // Selects last match
                                    if matches == 1 {
                                        cmd = key.to_string();
                                    }
                                    // eprintln!("key={key}, split[0]={}", split[0]);
                                }
                            }
                            if matches == 1 {
                                cmd
                            } else {
                                println!("No single matching key found");
                                continue;
                            }
                        }
                        None => {
                            println!(
                                "{} input was not valid and could not be processed",
                                style("error:").red().bold()
                            );
                            LoopCommand::print_help();
                            continue;
                        }
                    };
                    println!(
                        "command={command}, matching variant={:#?}",
                        LoopCommand::from_str(&command)?
                    );
                    if command == "help" {
                        println!();
                        LoopCommand::print_help();
                        continue;
                    }
                }
                Ok(())
            }
        "#;

        let mut build_state = setup_build_state(source);

        // Set up config similar to your actual config
        let mut config = Dependencies::default();
        config.exclude_unstable_features = true;
        config.exclude_std_feature = true;
        config.global_excluded_features = vec![
            "default".to_string(),
            "sqlite".to_string(),
            "unstable".to_string(),
        ];
        config.always_include_features = vec!["derive".to_string()];

        // Add rustyline override
        let rustyline_override = FeatureOverride {
            excluded_features: vec!["with-sqlite-history".to_string()],
            required_features: vec!["with-file-history".to_string()],
            alternative_features: vec![],
        };
        config
            .feature_overrides
            .insert("rustyline".to_string(), rustyline_override);

        // Mock the config for the test
        // (You'll need to show me how your config system works for this part)

        merge(&mut build_state, source).unwrap();

        let manifest = build_state.cargo_manifest.unwrap();

        // Check clap features
        let clap_deps = manifest.dependencies.get("clap").unwrap();
        if let Dependency::Detailed(detail) = clap_deps {
            let features = &detail.features;
            assert!(!features.contains(&"default".to_string()));
            assert!(!features.contains(&"unstable-derive-ui-tests".to_string()));
            assert!(features.contains(&"derive".to_string()));
        }

        // Check rustyline features
        let rustyline_deps = manifest.dependencies.get("rustyline").unwrap();
        if let Dependency::Detailed(detail) = rustyline_deps {
            let features = &detail.features;
            assert!(!features.contains(&"with-sqlite-history".to_string()));
            assert!(features.contains(&"with-file-history".to_string()));
        }

        // Log the complete manifest for debugging
        debug_log!("Final manifest: {:#?}", manifest);
    }
}
