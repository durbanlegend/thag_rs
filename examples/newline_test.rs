use std::path;

fn main() {
    let formatted_string = format!(
        "This is a string\nwith a newline.\nCurrent directory is {:#?}",
        std::env::current_dir().unwrap()
    );
    println!("{}", formatted_string);
    let path = path::PathBuf::from("/Users/donf/projects/runner/examples");
    std::env::set_current_dir(path);
    println!(
        "Current directory changed to {:#?}",
        std::env::current_dir().unwrap()
    );
}
