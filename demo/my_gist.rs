#![allow(unused)]
#![allow(unused_must_use)]

fn main() {
    use std::fmt;
    use std::io::{self, Write};
    
    let mut some_writer = io::stdout();
    write!(&mut some_writer, "{}", format_args!("Print with a {}", "macro"));
    
    fn my_fmt_fn(args: fmt::Arguments<'_>) {
        writeln!(&mut io::stdout(), "{args}");
    }
    my_fmt_fn(format_args!(", or a {} too", "function"));
}
