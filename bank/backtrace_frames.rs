fn main() {
    use backtrace::Backtrace;

    let mut current_backtrace = Backtrace::new_unresolved();
    current_backtrace.resolve();
    // println!("{current_backtrace:?}"); // symbol names now present
    for frame in Backtrace::frames(&current_backtrace) {
        for symbol in frame.symbols() {
            eprintln!("name={:?}", symbol.name());
        }
        // eprintln!("frame={frame:?}");
    }
}
