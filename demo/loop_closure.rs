/// Exploring the possibility of incorporating a line processor similar
/// to `rust-script`'s `--loop` or `runner`'s `--lines`. Might go with
/// the latter since I'm not sure what the closure logic buys us. It's
/// going to be checked by the compiler anyway. Compare with `demo/loop_expr.rs`.
///
/// P.S.: The `--loop` option has since been implemented in `thag(_rs)`, without closure logic.
//# Purpose: Evaluate closure logic for line processing.
//# Categories: exploration, technique
use std::io::Read;

let mut n = 0;
let mut filter = move |l: &str | {
    n += 1;
    println!("{n:>6}: {}", l.trim_end())
};

let mut buffer = String::new();
std::io::stdin().lock().read_to_string(&mut buffer)?;

for line in buffer.lines() {
    filter(line);
}
