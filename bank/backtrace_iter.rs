use backtrace::Backtrace;

fn main() {
    let module_path = module_path!();
    eprintln!("module_path={module_path}");

    let mut current_backtrace = Backtrace::new_unresolved();
    current_backtrace.resolve();
    let _ = Backtrace::frames(&current_backtrace)
        .iter()
        .flat_map(backtrace::BacktraceFrame::symbols)
        .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
        // .skip_while(|name| !(name.contains("Profile::new") && !name.contains("{{closure}}")))
        // Be careful, this is very sensitive to changes in the function signatures of this module.
        // .skip(1)
        .take_while(|name| !name.contains("__rust_begin_short_backtrace"))
        .inspect(|frame| {
            eprintln!(
                "frame: {frame}, frame.contains(module_path)? {}",
                frame.contains(module_path)
            );
        })
        .collect::<Vec<_>>();
}
