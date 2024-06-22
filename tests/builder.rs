#[cfg(test)]
mod tests {

    use rs_script::builder::{build, generate, run};
    use rs_script::cmd_args::Cli;
    use rs_script::{code_utils, execute, TMPDIR};
    use rs_script::{BuildState, CargoManifest, ProcFlags};
    // use sequential_test::sequential;
    use std::env::current_dir;
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::str::FromStr;

    // Helper function to create a sample Cli structure
    fn create_sample_cli(script: Option<String>) -> Cli {
        Cli {
            script,
            args: Vec::new(),
            expression: None,
            ..Default::default()
        }
    }

    // Helper function to create a sample VuildState structure.
    // Requires the sample script to be in tests/assets.
    fn create_sample_build_state(script_name: &str) -> BuildState {
        let script_stem: &str = script_name
            .strip_suffix(rs_script::RS_SUFFIX)
            .expect("Problem stripping Rust suffix");
        let current_dir = current_dir().expect("Could not get current dir");
        let cargo_home = home::cargo_home().expect("Could not get Cargo home");
        let target_dir_path = TMPDIR.join("rs-script").join(script_stem);
        fs::create_dir_all(target_dir_path.clone()).expect("Failed to create script directory");
        BuildState {
            working_dir_path: current_dir.clone(),
            source_stem: "test_run_script".into(),
            source_name: "test_run_script.rs".into(),
            source_dir_path: current_dir.join("tests/assets"),
            source_path: current_dir.join("tests/assets/test_run_script.rs"),
            cargo_home,
            target_dir_path,
            target_path: TMPDIR.join("rs-script/test_run_script/target/debug/test_run_script"),
            cargo_toml_path: TMPDIR.join("rs-script/test_run_script/Cargo.toml"),
            rs_manifest: None,
            cargo_manifest: None,
            must_gen: true,
            must_build: true,
        }
    }

    #[test]
    // #[sequential]
    fn test_execute_dynamic_script() {
        let mut args = create_sample_cli(Some(
            "tests/assets/determine_if_known_type_trait.rs".to_string(),
        ));
        args.force = true;
        let result = execute(args);
        assert!(result.is_ok());
    }

    // Any test of the REPL is roblematic because reedline will panic
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
        let script_name = "fib_fac_lite.rs";
        let mut build_state = create_sample_build_state(script_name);
        build_state.must_gen = true;
        build_state.must_build = true;
        build_state.cargo_toml_path = build_state.target_dir_path.clone().join("Cargo.toml");
        let cargo_toml = format!(
            r#"[package]
        name = "fib_fac_lite"
        version = "0.0.1"
        edition = "2021"

        [dependencies]
        itertools = "0.13.0"

        [features]

        [workspace]

        [[bin]]
        path = "{}/rs-script/fib_fac_lite/fib_fac_lite.rs"
        name = "fib_fac_lite"
"#,
            TMPDIR.display()
        );
        let cargo_manifest =
            CargoManifest::from_str(&cargo_toml).expect("Could not parse manifest string");
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
        let current_dir = current_dir().expect("Could not get current dir");
        let cargo_home = home::cargo_home().expect("Could not get Cargo home");
        let target_dir_path = TMPDIR.join("rs-script/fizz_buzz");
        fs::create_dir_all(target_dir_path.clone()).expect("Failed to create script directory");
        let cargo_toml_path = target_dir_path.clone().join("Cargo.toml");
        let cargo_toml = format!(
            r#"[package]
name = "fizz_buzz"
version = "0.0.1"
edition = "2021"

[dependencies]

[features]

[workspace]

[[bin]]
path = "{}/rs-script/fizz_buzz/fizz_buzz.rs"
name = "fizz_buzz"
"#,
            TMPDIR.display()
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

        let build_state = BuildState {
            working_dir_path: current_dir.clone(),
            source_stem: "fizz_buzz".into(),
            source_name: "fizz_buzz.rs".into(),
            source_dir_path: current_dir.join("tests/assets"),
            source_path: current_dir.join("tests/assets/fizz_buzz.rs"),
            cargo_home,
            target_path: target_dir_path.clone().join("target/debug/fizz_buzz"),
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
        let mut cli = create_sample_cli(Some("tests/assets/test_run_script.rs".to_string()));
        cli.run = true;
        let script_name = "test_run_script.rs";
        let build_state = create_sample_build_state(script_name);
        dbg!(&build_state);
        let proc_flags = ProcFlags::empty();
        // let result = execute(args);
        let result = run(&proc_flags, &cli.args, &build_state);
        assert!(result.is_ok());
    }
}
