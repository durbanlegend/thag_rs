use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    eprintln!("Hello world!");
    // let x = "/Users/donf/projects/build_run/examples/test_filepath.rs";
    // // let mut file = File::open("/private/var/folders/rx/mng2ds0s6y53v12znz5jhpk80000gn/T/rs_repl/repl_000100/repl_000100.rs")?;
    // // let mut file = File::open(x)?;
    // // let mut contents = String::new();
    // // file.read_to_string(&mut contents)?;
    // // assert_eq!(contents, "Hello, world!");
    // let contents = std::fs::read_to_string(x);
    // println!("contents={contents:#?}");
    Ok(())
}
