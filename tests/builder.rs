#[cfg(test)]
mod tests {
    
    use rs_script::builder::{build, generate, run};
    use rs_script::cmd_args::Cli;
    use rs_script::{execute, TMPDIR};
    use rs_script::{BuildState, ProcFlags};
    use sequential_test::sequential;
    use std::env::current_dir;
    use std::path::PathBuf;

    // Helper function to create a sample Cli structure
    fn create_sample_cli(script: Option<String>) -> Cli {
        // let cli = Cli::default();
        Cli {
            script,
            args: Vec::new(),
            expression: None,
            ..Default::default()
        }
    }

    #[test]
    #[sequential]
    fn test_execute_dynamic_script() {
        let mut args = create_sample_cli(Some(
            "tests/assets/determine_if_known_type_trait.rs".to_string(),
        ));
        args.force = true;
        let result = execute(args);
        assert!(result.is_ok());
    }

    // #[test]
    // fn test_execute_repl_script() {
    // let mut args = create_sample_cli(None);
    // args.repl = true;
    //     let result = execute(args);
    //     assert!(result.is_ok());
    // }

    #[ignore = "TODO get working"]
    #[test]
    fn test_generate_source_file() {
        let build_state = BuildState {
            source_path: PathBuf::from("test.rs"),
            target_dir_path: TMPDIR.to_path_buf(),
            cargo_toml_path: TMPDIR.join("Cargo.toml"),
            must_gen: true,
            must_build: true,
            ..Default::default()
        };
        let rs_source = "fn main() { println!(\"Hello, world!\"); }";
        let proc_flags = ProcFlags::empty();
        let result = generate(&build_state, rs_source, &proc_flags);
        assert!(result.is_ok());
        assert!(build_state.target_dir_path.join("test.rs").exists());
        assert!(build_state.cargo_toml_path.exists());
    }

    #[test]
    #[sequential]
    fn test_build_cargo_project() {
        let build_state = BuildState {
            working_dir_path: "/Users/donf/projects/rs-script".into(),
            source_stem: "determine_if_known_type_trait".into(),
                    source_name: "determine_if_known_type_trait.rs".into(),
                    source_dir_path: "/Users/donf/projects/rs-script/tests/assets".into(),
                    source_path: "/Users/donf/projects/rs-script/tests/assets/determine_if_known_type_trait.rs".into(),
                    cargo_home: "/Users/donf/.cargo".into(),
                    target_dir_path: TMPDIR.join("rs-script/determine_if_known_type_trait"),
                    target_path: TMPDIR.join("rs-script/determine_if_known_type_trait/target/debug/determine_if_known_type_trait"),
                    cargo_toml_path: TMPDIR.join("rs-script/determine_if_known_type_trait/Cargo.toml"),
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
        let current_dir = current_dir().expect("Could not get current dir");
        let cargo_home = home::cargo_home().expect("Could not get Cargo home");
        let build_state = BuildState {
            working_dir_path: current_dir.clone(),
            source_stem: "test_run_script".into(),
            source_name: "test_run_script.rs".into(),
            source_dir_path: current_dir.join("tests/assets").into(),
            source_path: current_dir.join("tests/assets/test_run_script.rs").into(),
            cargo_home: cargo_home,
            target_dir_path: TMPDIR.join("rs-script/test_run_script"),
            target_path: TMPDIR.join("rs-script/test_run_script/target/debug/test_run_script"),
            cargo_toml_path: TMPDIR.join("rs-script/test_run_script/Cargo.toml"),
            rs_manifest: None,
            cargo_manifest: None,
            must_gen: true,
            must_build: true,
        };
        dbg!(&build_state);
        let proc_flags = ProcFlags::empty();
        // let result = execute(args);
        let result = run(&proc_flags, &cli.args, &build_state);
        assert!(result.is_ok());
    }
}
