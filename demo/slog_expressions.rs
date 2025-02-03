/*[toml]
[dependencies]
slog = "2.7.0"
slog-term = "2.9.1"
*/

/// Published example from `slog` crate (misc/examples/expressions.rs).
//# Purpose: Demo a popular logging crate.
//# Categories: crates
use slog::{self, o, slog_warn, warn};
use slog_term;

use std::sync::Mutex;

struct Foo;

impl Foo {
    fn bar(&self) -> u32 {
        1
    }
}

struct X {
    foo: Foo,
}

fn baz() -> bool {
    true
}

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    use slog::Drain;

    let drain = Mutex::new(slog_term::FullFormat::new(decorator).build()).fuse();
    let log = slog::Logger::root(drain, o!("version" => env!("CARGO_PKG_VERSION")));

    let foo = Foo;
    let r = X { foo };

    warn!(log, "logging message");
    slog_warn!(log, "logging message");

    warn!(log, "logging message"; "a" => "b");
    slog_warn!(log, "logging message"; "a" => "b");

    warn!(log, "logging message bar={}", r.foo.bar());
    slog_warn!(log, "logging message bar={}", r.foo.bar());

    warn!(
        log,
        "logging message bar={} foo={}",
        r.foo.bar(),
        r.foo.bar()
    );
    slog_warn!(
        log,
        "logging message bar={} foo={}",
        r.foo.bar(),
        r.foo.bar()
    );

    warn!(
        log,
        "logging message bar={} foo={}",
        r.foo.bar(),
        r.foo.bar(),
    );
    slog_warn!(
        log,
        "logging message bar={} foo={}",
        r.foo.bar(),
        r.foo.bar(),
    );

    warn!(log, "logging message bar={}", r.foo.bar(); "x" => 1);
    slog_warn!(log, "logging message bar={}", r.foo.bar(); "x" => 1);

    warn!(log, "logging message bar={}", r.foo.bar(); "x" => 1);
    slog_warn!(log, "logging message bar={}", r.foo.bar(); "x" => 1);

    warn!(log, "logging message bar={}", r.foo.bar(); "x" => 1, "y" => r.foo.bar());
    slog_warn!(log, "logging message bar={}", r.foo.bar(); "x" => 1, "y" => r.foo.bar());

    warn!(log, "logging message bar={}", r.foo.bar(); "x" => r.foo.bar());
    slog_warn!(log, "logging message bar={}", r.foo.bar(); "x" => r.foo.bar());

    warn!(log, "logging message bar={}", r.foo.bar(); "x" => r.foo.bar(), "y" => r.foo.bar());
    slog_warn!(log, "logging message bar={}", r.foo.bar(); "x" => r.foo.bar(), "y" => r.foo.bar());

    warn!(log, "logging message bar={}", r.foo.bar(); "x" => r.foo.bar(), "y" => r.foo.bar());
    slog_warn!(log, "logging message bar={}", r.foo.bar(); "x" => r.foo.bar(), "y" => r.foo.bar());
}
