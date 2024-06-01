use clap::Parser;
use rs_script::{get_proc_flags, Cli, ProcFlags};

#[test]
fn test_get_proc_flags() {
    let args = vec!["program_name", "--edit"];
    let cli = Cli::parse_from(args);
    let flags = get_proc_flags(&cli);
    assert!(flags
        .expect("Couldn't access ProcFlags")
        .contains(ProcFlags::EDIT & ProcFlags::ALL));
}

#[test]
fn test_conflicts_with_all() {
    let args = vec!["program_name", "--expr", "--edit"];
    let result = Cli::try_parse_from(args);
    assert!(result.is_err()); // or check for specific error
}
