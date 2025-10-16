use backtrace::Backtrace;

fn main() {
    let module_path = module_path!();
    eprintln!("module_path={module_path}");

    let here = Backtrace::frames(&Backtrace::new())
        .iter()
        .flat_map(backtrace::BacktraceFrame::symbols)
        .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
        .skip_while(|frame| !(frame.contains(module_path)))
        // Be careful, this is very sensitive to changes in the function signatures of this module.
        .take(1)
        // .inspect(|frame| {
        //     eprintln!(
        //         "frame: {frame}, frame.contains(module_path)? {}",
        //         frame.contains(module_path)
        //     );
        // })
        .last()
        .unwrap();

    println!("here={here}");
}
