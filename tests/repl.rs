#[cfg(test)]
mod tests {
    use clap::{ArgMatches, Parser};
    use home::home_dir;
    use rs_script::cmd_args::{Cli, ProcFlags};
    use rs_script::repl::{
        delete, disp_repl_banner, edit, edit_history, list, parse_line, run_expr, toml, Context,
    };
    use rs_script::shared::BuildState;
    use std::path::PathBuf;
    use std::time::Instant;

    // Helper function to create a mock context
    fn create_mock_context<'a>(
        options: &'a mut Cli,
        proc_flags: &'a ProcFlags,
        build_state: &'a mut BuildState,
    ) -> Context<'a> {
        let start = Instant::now();
        Context {
            options,
            proc_flags,
            build_state,
            start,
        }
    }

    // Set environment variables before running tests
    fn set_up() {
        std::env::set_var("TEST_ENV", "1");
        std::env::set_var("VISUAL", "cat");
        std::env::set_var("EDITOR", "cat");
    }

    #[test]
    fn test_parse_line() {
        set_up();
        let input = r#"command "arg 1" arg2"#;
        let (command, args) = parse_line(input);
        println!("\r");
        assert_eq!(command, "command");
        assert_eq!(args, vec!["arg 1".to_string(), "arg2".to_string()]);
    }

    #[test]
    fn test_disp_repl_banner() {
        set_up();
        let cmd_list = "command1, command2";
        disp_repl_banner(cmd_list);
        // As this function prints to stdout, there's no direct return value to assert.
        // We assume that if it runs without panic, it is successful.
    }

    #[test]
    fn test_delete() {
        set_up();
        let mut options = Cli::parse_from(&["test"]);
        let proc_flags = ProcFlags::default();
        let mut build_state = BuildState::default();
        let mut context = create_mock_context(&mut options, &proc_flags, &mut build_state);
        let result = delete(&ArgMatches::default(), &mut context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_edit_history() {
        set_up();
        let mut options = Cli::parse_from(&["test"]);
        let proc_flags = ProcFlags::default();
        let mut build_state = rs_script::BuildState {
            cargo_home: home_dir().unwrap().join(".cargo"),
            ..Default::default()
        };
        let mut context = create_mock_context(&mut options, &proc_flags, &mut build_state);
        let result = edit_history(&ArgMatches::default(), &mut context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_edit() {
        set_up();
        let mut options = Cli::parse_from(&["test"]);
        let proc_flags = ProcFlags::default();
        let mut build_state = BuildState {
            source_path: PathBuf::from("tests/assets/hello_t.rs"),
            ..Default::default()
        };
        // build_state.source_path = std::path::PathBuf::from("tests/assets/hello_t.rs");
        let mut context = create_mock_context(&mut options, &proc_flags, &mut build_state);
        let result = edit(&ArgMatches::default(), &mut context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_toml() {
        set_up();
        let mut options = Cli::parse_from(&["test"]);
        let proc_flags = ProcFlags::default();
        let mut build_state = BuildState {
            cargo_toml_path: PathBuf::from("tests/assets/Cargo_t.toml"),
            ..Default::default()
        };
        let mut context = create_mock_context(&mut options, &proc_flags, &mut build_state);
        let result = toml(&ArgMatches::default(), &mut context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_expr() {
        set_up();
        let mut options = Cli::parse_from(&["test"]);
        let proc_flags = ProcFlags::default();
        let mut build_state = BuildState::default();
        let mut context = create_mock_context(&mut options, &proc_flags, &mut build_state);
        let result = run_expr(&ArgMatches::default(), &mut context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list() {
        set_up();
        let mut options = Cli::parse_from(&["test"]);
        let proc_flags = ProcFlags::default();
        let mut build_state = BuildState::default();
        let mut context = create_mock_context(&mut options, &proc_flags, &mut build_state);
        let result = list(&ArgMatches::default(), &mut context);
        assert!(result.is_ok());
    }

    // #[test]
    // fn test_run_repl() {
    //     set_up();
    //     let mut options = Cli::parse_from(&["test"]);
    //     let proc_flags = ProcFlags::default();
    //     let mut build_state = BuildState::default();
    //     let start = Instant::now();
    //     let result = run_repl(&mut options, &proc_flags, &mut build_state, start);
    //     assert!(result.is_ok());
    // }
}
