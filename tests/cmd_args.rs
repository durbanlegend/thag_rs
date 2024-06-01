use clap::Parser;
use rs_script::{get_proc_flags, Cli, ProcFlags};

#[test]
fn test_get_args_script() {
    let args = vec!["rs_script", "demo_script", "--", "arg1", "arg2"];
    let cli = Cli::parse_from(args);
    assert!(Some("demo_script") == cli.script.as_deref());
    // println!("cli.args={:#?}", cli.args);
    assert!(vec!["arg1", "arg2"] == cli.args);
}

#[test]
fn test_get_args_expr() {
    let args = vec!["rs_script", "--expr", "'2 + 5'"];
    let cli = Cli::parse_from(args);
    // println!("cli.script.as_deref()={}", cli.script.as_deref());
    assert!(cli.script.as_deref().is_none());
    assert!(Some("'2 + 5'") == cli.expression.as_deref());
}

#[test]
fn test_get_args_stdin() {
    let args = vec!["rs_script", "-s"];
    let cli = Cli::parse_from(args);
    // println!("cli.script.as_deref()={}", cli.script.as_deref());
    assert!(cli.script.as_deref().is_none());
    assert!(cli.expression.as_deref().is_none());
}

#[test]
fn test_get_proc_flags() {
    let args = vec!["rs_script", "--expr", "'2 + 5'"];
    let cli = Cli::parse_from(args);
    let result = get_proc_flags(&cli);
    let proc_flags = result.expect("Couldn't access ProcFlags");
    assert!(proc_flags.contains(
        ProcFlags::EXPR & ProcFlags::ALL & ProcFlags::GENERATE & ProcFlags::BUILD & ProcFlags::RUN
    ));
    assert!(!proc_flags.contains(
        ProcFlags::FORCE
            | ProcFlags::TIMINGS
            | ProcFlags::VERBOSE
            | ProcFlags::TIMINGS
            | ProcFlags::REPL
            | ProcFlags::EDIT
            | ProcFlags::STDIN
    ));
}

#[test]
fn test_conflicts_with_all() {
    let args = vec!["rs_script", "--expr", "--edit"];
    let result = Cli::try_parse_from(args);
    // println!("result={result:#?}");
    assert!(result.is_err()); // or check for specific error
}
