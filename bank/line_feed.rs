fn read_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut buffer)?;
    Ok(buffer)
}

let lf1 = r#"
"#.as_bytes();
let lf2 = "\n".as_bytes();

println!("lf1={lf1:?}, lf2={lf2:?}");

println!("Type text wall at the prompt and hit Ctrl-D on a new line when done");

let input = read_stdin()?;
input
    .as_bytes()
    .iter()
    .for_each(|b| eprintln!("b = {b:?} = {b:x} = {}", *b as char));
