#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    use clap::Parser;
    use std::path::PathBuf;
    use std::sync::Once;
    use std::time::Instant;
    use thag_rs::cmd_args::{Cli, ProcFlags};
    use thag_rs::code_utils::read_file_contents;
    use thag_rs::repl::{delete, disp_repl_banner, list, parse_line, process_source};
    #[cfg(not(windows))]
    use thag_rs::repl::{edit, edit_history, toml, HISTORY_FILE};
    use thag_rs::BuildState;

    fn init_logger() {
        env_logger::init();
    }

    // Set environment variables before running tests
    fn set_up() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            unsafe {
                std::env::set_var("TEST_ENV", "1");
                std::env::set_var("VISUAL", "cat");
                std::env::set_var("EDITOR", "cat");
            }
            init_logger();
        });
    }

    #[test]
    fn test_repl_command_print_help() {
        set_up();
        let mut output = Vec::new();
        let mut command = thag_rs::repl::ReplCommand::command();
        command.write_long_help(&mut output).unwrap();
        let help_output = String::from_utf8(output).unwrap();
        assert!(help_output.contains("REPL mode lets you type or paste a Rust expression"));
    }

    #[test]
    fn test_repl_parse_line() {
        set_up();
        let input = r#"command "arg 1" arg2"#;
        let (command, args) = parse_line(input);
        println!("\r");
        assert_eq!(command, "command");
        assert_eq!(args, vec!["arg 1".to_string(), "arg2".to_string()]);
    }

    #[test]
    fn test_repl_disp_repl_banner() {
        set_up();
        let cmd_list = "command1, command2";
        disp_repl_banner(cmd_list);
        // As this function prints to stdout, there's no direct return value to assert.
        // We assume that if it runs without panic, it is successful.
    }

    #[test]
    fn test_repl_delete() {
        set_up();
        let build_state = BuildState::default();

        let result = delete(&build_state);
        assert!(result.is_ok());
    }

    #[cfg(not(windows))]
    #[test]
    fn test_repl_edit_history() {
        use std::fs::read_to_string;

        use mockall::Sequence;
        use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
        use thag_rs::MockEventReader;

        set_up();
        let build_state = thag_rs::BuildState {
            cargo_home: PathBuf::from("tests/assets/"),
            ..Default::default()
        };

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
        let history_string =
            read_to_string(&history_path).expect(&format!("Error reading from {history_path:?}"));

        let staging_path: PathBuf = build_state.cargo_home.join("hist_staging.txt");
        let result = edit_history(&history_string, &staging_path, &mock_reader);
        dbg!(&result);
        assert!(&result.is_ok());
    }

    #[cfg(not(windows))]
    #[test]
    fn test_repl_edit() {
        set_up();
        let build_state = BuildState {
            source_path: PathBuf::from("tests/assets/hello_t.rs"),
            ..Default::default()
        };

        let result = edit(&build_state.source_path);
        assert!(result.is_ok());
    }

    #[cfg(not(windows))]
    #[test]
    fn test_repl_toml() {
        set_up();
        let build_state = BuildState {
            cargo_toml_path: PathBuf::from("tests/assets/Cargo_t.toml"),
            ..Default::default()
        };

        let result = toml(&build_state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_repl_process_source() {
        set_up();
        let args = Cli::parse_from(["test", "--repl"]);
        let proc_flags = ProcFlags::default();
        let source_path = PathBuf::from("tests/assets/hello_t.rs");
        let rs_source = read_file_contents(&source_path).expect("Missing source file");
        let mut build_state = BuildState {
            source_path,
            must_gen: true,
            ..Default::default()
        };
        let result = process_source(
            &rs_source,
            &mut build_state,
            &args,
            &proc_flags,
            Instant::now(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_repl_list() {
        set_up();

        let result = list(&BuildState::default());
        assert!(result.is_ok());
    }

    // #[test]
    // fn test_repl_run_repl() {
    //     set_up();
    //     let mut options = Cli::parse_from(["test", "repl"]);
    //     let proc_flags = ProcFlags::default();
    //     let mut build_state = BuildState::default();
    //     let start = Instant::now();
    //     let result = run_repl(&mut options, &proc_flags, &mut build_state, start);
    //     assert!(result.is_ok());
    // }
}
