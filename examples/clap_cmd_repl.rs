/*[toml]
[dependencies*
clapcmd = "0.3.3"
*/
use clapcmd::{Arg, ArgMatches, ClapCmd, ClapCmdResult, Command, ValueHint};

/// One example from the clapcmd crate
fn do_ls(cmd: &mut ClapCmd, matches: ArgMatches) -> ClapCmdResult {
    let empty = String::from("");
    let file: &String = matches.get_one("file").unwrap_or(&empty);
    let dir: &String = matches.get_one("dir").unwrap_or(&empty);
    cmd.info(format!("received info\nfile: {:?}\ndir: {:?}", file, dir));
    Ok(())
}

fn main() {
    let mut cmd = ClapCmd::default();
    cmd.add_command(
        do_ls,
        Command::new("ls")
            .about("ls a file")
            .arg(
                Arg::new("any")
                    .short('a')
                    .long("any")
                    .help("any to display")
                    .value_hint(ValueHint::AnyPath),
            )
            .arg(
                Arg::new("file")
                    .short('f')
                    .long("file")
                    .help("file to display")
                    .value_hint(ValueHint::FilePath),
            )
            .arg(
                Arg::new("dir")
                    .short('d')
                    .long("dir")
                    .help("dir to display")
                    .value_hint(ValueHint::DirPath),
            ),
    );
    cmd.run_loop();
}
