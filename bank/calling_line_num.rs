extern crate backtrace;

use backtrace::{ Backtrace, BacktraceFrame, BacktraceSymbol };

fn previous_symbol(level: u32) -> Option<BacktraceSymbol> {
    let (trace, curr_file, curr_line) = (Backtrace::new(), file!(), line!());
    let frames = trace.frames();
    frames.iter()
          .flat_map(BacktraceFrame::symbols)
          .skip_while(|s| s.filename().map(|p| !p.ends_with(curr_file)).unwrap_or(true)
                       || s.lineno() != Some(curr_line))
          .nth(1 + level as usize).cloned()
}

fn foo() {
    let sym = previous_symbol(1);
    println!("called from {:?}:{:?}",
             sym.as_ref().and_then(BacktraceSymbol::filename),
             sym.as_ref().and_then(BacktraceSymbol::lineno));
}

fn bar() { foo(); }

fn main() { bar(); }
