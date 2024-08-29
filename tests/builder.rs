#[cfg(test)]
mod tests {

    use cargo_toml::Manifest;
    use thag_rs::builder::{build, generate, run};
    use thag_rs::cmd_args::Cli;
    use thag_rs::{code_utils, escape_path_for_windows, execute, TMPDIR};
    use thag_rs::{BuildState, ProcFlags};
    // use sequential_test::sequential;
    use std::env::current_dir;
    use std::fs::{self, OpenOptions};
    use std::io::Write;

    // Set environment variables before running tests
    fn set_up() {
        std::env::set_var("TEST_ENV", "1");
        std::env::set_var("VISUAL", "cat");
        std::env::set_var("EDITOR", "cat");
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
        let current_dir = current_dir().expect("Could not get current dir");
        let working_dir_path = current_dir.clone();
        let cargo_home = home::cargo_home().expect("Could not get Cargo home");
        let target_dir_path = TMPDIR.join("thag_rs").join(source_stem);
        fs::create_dir_all(target_dir_path.clone()).expect("Failed to create script directory");
        let target_path = target_dir_path
            .clone()
            .join("target/debug")
            .join(source_stem);
        let cargo_toml_path = target_dir_path.clone().join("Cargo.toml");
        let source_dir_path = current_dir.clone().join("tests/assets");
        let source_path = current_dir.clone().join("tests/assets").join(source_name);
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
        }
    }

    #[test]
    // #[sequential]
    fn test_execute_dynamic_script() {
        set_up();
        let mut args = create_sample_cli(Some(
            "tests/assets/determine_if_known_type_trait_t.rs".to_string(),
        ));
        args.force = true;
        let result = execute(&mut args);
        assert!(result.is_ok());
    }

    // Any test of the REPL is problematic because reedline will panic
    // with a message that the current cursor position can't be found.
    // #[test]
    // fn test_execute_repl_script() {
    // let mut args = create_sample_cli(None);
    // args.repl = true;
    //     let result = execute(args);
    //     assert!(result.is_ok());
    // }

    #[test]
    fn test_generate_source_file() {
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
        let cargo_manifest =
            Manifest::from_str(&cargo_toml).expect("Could not parse manifest string");
        build_state.cargo_manifest = Some(cargo_manifest);

        let rs_source = code_utils::read_file_contents(&build_state.source_path)
            .expect("Error reading script contents");
        let proc_flags = ProcFlags::empty();
        let result = generate(&build_state, &rs_source, &proc_flags);
        assert!(result.is_ok());
        assert!(build_state.target_dir_path.join(script_name).exists());
        assert!(build_state.cargo_toml_path.exists());
    }

    #[test]
    // #[sequential]
    fn test_build_cargo_project() {
        set_up();
        let source_name = "bitflags_t.rs";
        let source_stem: &str = source_name
            .strip_suffix(thag_rs::RS_SUFFIX)
            .expect("Problem stripping Rust suffix");

        let current_dir = current_dir().expect("Could not get current dir");
        let source_path = current_dir.join("tests/assets").join(source_name);
        let cargo_home = home::cargo_home().expect("Could not get Cargo home");
        let target_dir_path = TMPDIR.join("thag_rs").join(source_stem);
        fs::create_dir_all(target_dir_path.clone()).expect("Failed to create script directory");
        let cargo_toml_path = target_dir_path.clone().join("Cargo.toml");
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

        let target_rs_path = target_dir_path.clone().join(source_name);

        let rs_source =
            code_utils::read_file_contents(&source_path).expect("Error reading script contents");
        let _source_file = code_utils::write_source(&target_rs_path, &rs_source)
            .expect("Problem writing source to target path");
        // println!("source_file={source_file:#?}");

        let build_state = BuildState {
            working_dir_path: current_dir.clone(),
            source_stem: source_stem.into(),
            source_name: source_name.into(),
            source_dir_path: current_dir.join("tests/assets"),
            source_path,
            cargo_home,
            target_path: target_dir_path
                .clone()
                .join("target/debug")
                .join(source_stem),
            cargo_toml_path,
            target_dir_path,
            rs_manifest: None,
            cargo_manifest: None,
            must_gen: true,
            must_build: true,
        };
        dbg!(&build_state);
        let proc_flags = ProcFlags::empty();
        let result = build(&proc_flags, &build_state);
        assert!(result.is_ok());
    }

    #[test]
    // #[sequential]
    fn test_run_script() {
        set_up();
        let source_name = "fib_fac_dashu_t.rs";
        let source_stem: &str = source_name
            .strip_suffix(thag_rs::RS_SUFFIX)
            .expect("Problem stripping Rust suffix");
        let target_dir_path = TMPDIR
            .join("thag_rs")
            .join(source_stem)
            .join("target/debug");
        let target_path = if cfg!(windows) {
            target_dir_path.join(source_stem.to_string() + ".exe")
        } else {
            target_dir_path.join(source_stem)
        };

        // Remove executable if it exists, and check
        let result = fs::remove_file(&target_path);
        eprintln!("Result of fs::remove_file({target_path:?}): {result:?}");
        assert!(!target_path.exists());

        // Generate and build executable, and check it exists.
        let mut args = create_sample_cli(Some("tests/assets/fib_fac_dashu_t.rs".to_string()));
        args.generate = true;
        args.build = true;
        let result = execute(&mut args);
        assert!(result.is_ok());
        println!("target_path={target_path:#?}");
        assert!(target_path.exists());

        // Finally, run it
        let cli = create_sample_cli(Some(format!("tests/assets/{source_name}")));
        let build_state = create_sample_build_state(source_name);
        dbg!(&build_state);
        let proc_flags = ProcFlags::empty();
        let result = run(&proc_flags, &cli.args, &build_state);
        assert!(result.is_ok());
    }
}
