use clap::Parser;
use thag_rs::{get_proc_flags, Cli, ProcFlags};

// Set environment variables before running tests
fn set_up() {
    std::env::set_var("TEST_ENV", "1");
    std::env::set_var("VISUAL", "cat");
    std::env::set_var("EDITOR", "cat");
}

#[test]
fn test_cmd_args_get_args_script() {
    set_up();
    let args = vec!["thag", "demo_script", "--", "arg1", "arg2"];
    let cli = Cli::parse_from(args);
    assert!(Some("demo_script") == cli.script.as_deref());
    // println!("cli.args={:#?}", cli.args);
    assert!(vec!["arg1", "arg2"] == cli.args);
}

#[test]
fn test_cmd_args_get_args_expr() {
    set_up();
    let args = vec!["thag", "--expr", "'2 + 5'"];
    let cli = Cli::parse_from(args);
    // println!("cli.script.as_deref()={}", cli.script.as_deref());
    assert!(cli.script.as_deref().is_none());
    assert!(Some("'2 + 5'") == cli.expression.as_deref());
}

#[test]
fn test_cmd_args_get_args_stdin() {
    set_up();
    let args = vec!["thag", "-s"];
    let cli = Cli::parse_from(args);
    // println!("cli.script.as_deref()={}", cli.script.as_deref());
    assert!(cli.script.as_deref().is_none());
    assert!(cli.expression.as_deref().is_none());
}

#[test]
fn test_cmd_args_get_proc_flags() {
    set_up();
    let args = vec!["thag", "--expr", "'2 + 5'"];
    let cli = Cli::parse_from(args);
    let result = get_proc_flags(&cli);
    let proc_flags = result.expect("Couldn't access ProcFlags");
    assert!(proc_flags
        .contains(ProcFlags::EXPR & ProcFlags::GENERATE & ProcFlags::BUILD & ProcFlags::RUN));
    assert!(!proc_flags.contains(
        ProcFlags::FORCE
            | ProcFlags::DEBUG
            | ProcFlags::VERBOSE
            | ProcFlags::TIMINGS
            | ProcFlags::REPL
            | ProcFlags::EDIT
            | ProcFlags::STDIN
    ));
}

#[test]
fn test_cmd_args_conflicts_with_all() {
    set_up();
    let args = vec!["thag", "--expr", "--edit"];
    let result = Cli::try_parse_from(args);
    // println!("result={result:#?}");
    assert!(result.is_err()); // or check for specific error
}

#[test]
fn test_cmd_args_proc_flags_generate_build_force_run() {
    set_up();
    let args = vec!["thag", "/demo/hello.rs", "-f"];
    let cli = Cli::parse_from(args);
    let result = get_proc_flags(&cli);
    let proc_flags = result.expect("Couldn't access ProcFlags");
    assert!(proc_flags
        .contains(ProcFlags::GENERATE | ProcFlags::BUILD | ProcFlags::FORCE | ProcFlags::RUN));
}

#[test]
fn test_cmd_args_proc_flags_generate_build_run() {
    set_up();
    let args = vec!["thag", "/demo/hello.rs", "-g"];
    let cli = Cli::parse_from(args);
    let result = get_proc_flags(&cli);
    let proc_flags = result.expect("Couldn't access ProcFlags");
    assert!(proc_flags.contains(ProcFlags::GENERATE | ProcFlags::BUILD | ProcFlags::RUN));
}

#[test]
fn test_cmd_args_proc_flags_build_run() {
    set_up();
    let args = vec!["thag", "/demo/hello.rs", "-b"];
    let cli = Cli::parse_from(args);
    let result = get_proc_flags(&cli);
    let proc_flags = result.expect("Couldn't access ProcFlags");
    assert!(proc_flags.contains(ProcFlags::BUILD | ProcFlags::RUN));
}

#[test]
fn test_cmd_args_proc_flags_norun() {
    set_up();
    let args = vec!["thag", "/demo/hello.rs", "-n"];
    let cli = Cli::parse_from(args);
    let result = get_proc_flags(&cli);
    let proc_flags = result.expect("Couldn't access ProcFlags");
    assert!(proc_flags.contains(ProcFlags::NORUN));
}

#[test]
fn test_cmd_args_proc_flags_expr() {
    set_up();
    let args = vec!["thag", "-e", "Hi"];
    let cli = Cli::parse_from(args);
    let result = get_proc_flags(&cli);
    let proc_flags = result.expect("Couldn't access ProcFlags");
    assert!(proc_flags
        .contains(ProcFlags::GENERATE | ProcFlags::BUILD | ProcFlags::RUN | ProcFlags::EXPR));
}

#[test]
fn test_cmd_args_proc_flags_edit() {
    set_up();
    let args = vec!["thag", "-d"];
    let cli = Cli::parse_from(args);
    let result = get_proc_flags(&cli);
    let proc_flags = result.expect("Couldn't access ProcFlags");
    assert!(proc_flags
        .contains(ProcFlags::GENERATE | ProcFlags::BUILD | ProcFlags::RUN | ProcFlags::EDIT));
}

#[test]
fn test_cmd_args_proc_flags_stdin() {
    set_up();
    let args = vec!["thag", "-s"];
    let cli = Cli::parse_from(args);
    let result = get_proc_flags(&cli);
    let proc_flags = result.expect("Couldn't access ProcFlags");
    assert!(proc_flags
        .contains(ProcFlags::GENERATE | ProcFlags::BUILD | ProcFlags::RUN | ProcFlags::STDIN));
}

#[test]
fn test_cmd_args_proc_flags_loop() {
    set_up();
    let args = vec!["thag", "-l", "&line"];
    let cli = Cli::parse_from(args);
    let result = get_proc_flags(&cli);
    let proc_flags = result.expect("Couldn't access ProcFlags");
    assert!(proc_flags
        .contains(ProcFlags::GENERATE | ProcFlags::BUILD | ProcFlags::RUN | ProcFlags::LOOP));
}

#[test]
fn test_cmd_args_proc_flags_repl() {
    set_up();
    let args = vec!["thag", "-r"];
    let cli = Cli::parse_from(args);
    let result = get_proc_flags(&cli);
    let proc_flags = result.expect("Couldn't access ProcFlags");
    assert!(proc_flags
        .contains(ProcFlags::GENERATE | ProcFlags::BUILD | ProcFlags::RUN | ProcFlags::REPL));
}

#[test]
fn test_cmd_args_proc_flags_executable() {
    set_up();
    let args = vec!["thag", "/demo/hello.rs", "-x"];
    let cli = Cli::parse_from(args);
    let result = get_proc_flags(&cli);
    let proc_flags = result.expect("Couldn't access ProcFlags");
    assert!(proc_flags.contains(
        ProcFlags::GENERATE | ProcFlags::BUILD | ProcFlags::NORUN | ProcFlags::EXECUTABLE
    ));
}
