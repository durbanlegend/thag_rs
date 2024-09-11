#[cfg(test)]
mod tests {
    use clap::Parser;
    #[cfg(not(windows))]
    use std::path::PathBuf;
    use thag_rs::cmd_args::{Cli, ProcFlags};
    use thag_rs::repl::{delete, disp_repl_banner, list, parse_line, run_expr, HISTORY_FILE};
    #[cfg(not(windows))]
    use thag_rs::repl::{edit, edit_history, toml};
    use thag_rs::shared::BuildState;

    // // Helper function to create a mock context
    // fn create_mock_context<'a>(
    //     options: &'a mut Cli,
    //     proc_flags: &'a ProcFlags,
    //     build_state: &'a mut BuildState,
    // ) -> Context<'a> {
    //     let start = Instant::now();
    //     Context {
    //         args: options,
    //         proc_flags,
    //         build_state,
    //         start,
    //     }
    // }

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
        // let mut options = Cli::parse_from(["test", "repl"]);
        // let proc_flags = ProcFlags::default();
        let build_state = BuildState::default();
        // let mut context = create_mock_context(&mut options, &proc_flags, &mut build_state);
        let result = delete(&build_state);
        assert!(result.is_ok());
    }

    #[cfg(not(windows))]
    #[test]
    fn test_edit_history() {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
        use mockall::Sequence;
        use thag_rs::stdin::MockEventReader;

        set_up();
        // let mut options = Cli::parse_from(["test", "repl"]);
        // let proc_flags = ProcFlags::default();
        let build_state = thag_rs::BuildState {
            cargo_home: PathBuf::from("tests/assets/"),
            ..Default::default()
        };
        // let mut context = create_mock_context(&mut options, &proc_flags, &mut build_state);
        let mut seq = Sequence::new();
        let mut mock_reader = MockEventReader::new();

        mock_reader
            .expect_read_event()
            .times(1)
            .in_sequence(&mut seq)
            .return_once(|| Ok(Event::Paste("Hello,\nworld".to_string())));

        mock_reader
            .expect_read_event()
            .times(1)
            .in_sequence(&mut seq)
            .return_once(|| {
                Ok(Event::Key(KeyEvent::new(
                    KeyCode::Char('!'),
                    KeyModifiers::NONE,
                )))
            });

        mock_reader
            .expect_read_event()
            .times(1)
            .in_sequence(&mut seq)
            .return_once(|| {
                Ok(Event::Key(KeyEvent::new(
                    KeyCode::Char('d'),
                    KeyModifiers::CONTROL,
                )))
            });

        let history_path = build_state.cargo_home.join(HISTORY_FILE);
        let staging_path: PathBuf = build_state.cargo_home.join("hist_staging");
        let result = edit_history(&history_path, &staging_path, &mock_reader);
        assert!(result.is_ok());
    }

    #[cfg(not(windows))]
    #[test]
    fn test_edit() {
        set_up();
        // let mut options = Cli::parse_from(["test", "repl"]);
        // let proc_flags = ProcFlags::default();
        let build_state = BuildState {
            source_path: PathBuf::from("tests/assets/hello_t.rs"),
            ..Default::default()
        };
        // build_state.source_path = std::path::PathBuf::from("tests/assets/hello_t.rs");
        // let mut context = create_mock_context(&mut options, &proc_flags, &mut build_state);
        let result = edit(&build_state);
        assert!(result.is_ok());
    }

    #[cfg(not(windows))]
    #[test]
    fn test_toml() {
        set_up();
        // let mut options = Cli::parse_from(["test", "repl"]);
        // let proc_flags = ProcFlags::default();
        let build_state = BuildState {
            cargo_toml_path: PathBuf::from("tests/assets/Cargo_t.toml"),
            ..Default::default()
        };
        // let mut context = create_mock_context(&mut options, &proc_flags, &mut build_state);
        let result = toml(&build_state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_expr() {
        set_up();
        let args = Cli::parse_from(["test", "--repl"]);
        let proc_flags = ProcFlags::default();
        let mut build_state = BuildState {
            must_gen: true,
            ..Default::default()
        };
        // let mut context = create_mock_context(&mut args, &proc_flags, &mut build_state);
        let result = run_expr(&args, &proc_flags, &mut build_state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list() {
        set_up();
        // let mut options = Cli::parse_from(["test", "repl"]);
        // let proc_flags = ProcFlags::default();
        // let mut context = create_mock_context(&mut options, &proc_flags, &mut build_state);
        let result = list(&BuildState::default());
        assert!(result.is_ok());
    }

    // #[test]
    // fn test_run_repl() {
    //     set_up();
    //     let mut options = Cli::parse_from(["test", "repl"]);
    //     let proc_flags = ProcFlags::default();
    //     let mut build_state = BuildState::default();
    //     let start = Instant::now();
    //     let result = run_repl(&mut options, &proc_flags, &mut build_state, start);
    //     assert!(result.is_ok());
    // }
}
