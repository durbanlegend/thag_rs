#! /usr/bin/env thag
#![feature(duration_constructors)]

//: Minimal snippet showing how to add nice additional constructors such as `from_weeks` (and days, hours and
//: minutes) to `std::time::Duration`.
//:
//: These are enabled by adding the inner attribute `#![feature(duration_constructors)]` to the script.
//: I've used a snippet to illustrate that this is possible: an inner attribute (i.e. an attribute prefixed
//: with `#!` (`#![...]`)) must be placed at the top of the crate it applies to, so when wrapping the snippet
//: in a fn main, thag_rs pulls any inner attributes out to the top of the program.
//:
//: Notice we also have a shebang so that this script may be run as `demo/duration_snippet.rs` with execute
//: permission. The shebang must be on the very first line but coexists peacefully with the inner attribute.
//:
//: See tracking issue https://github.com/rust-lang/rust/issues/120301 for the `Duration` constructor issue..
//:
//: E.g. `(*nix)`:
//:
//:     chmod u+g demo/duration_snippet.rs      // Only required the first time of course
//:     demo/duration_snippet.rs -qq
//:     1209600s
//:
//: Or more concisely:
//:
//:     f=demo/duration_snippet.rs && chmod u+g $f && $f -qq
//:     1209600s
//:
//# Purpose: Demonstrate that some fairly subtle moves are possible even with the simplest of snippets.
//# Categories: technique
std::time::Duration::from_weeks(2)
