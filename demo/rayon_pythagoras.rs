/// Highly simplified version of the published benchmark from the `rayon` crate.
/// https://github.com/rayon-rs/rayon/blob/main/rayon-demo/src/pythagoras/mod.rs
//# Purpose: Demo the featured crate and the performance effect of using `rayon`.
//# Categories: crates, performance, timing
// How many Pythagorean triples exist less than or equal to 4 million?
// i.e. a²+b²=c² and a,b,c ≤ 4000000
use num::Integer;
use rayon::prelude::*;
use rayon::range::Iter;

/// Use Euclid's formula to count Pythagorean triples
///
/// https://en.wikipedia.org/wiki/Pythagorean_triple#Generating_a_triple
///
/// For coprime integers m and n, with m > n and m-n is odd, then
///     a = m²-n², b = 2mn, c = m²+n²
///
/// This is a coprime triple.  Multiplying by factors k covers all triples.
use std::time::Duration;

use thag_proc_macros::timing;

const MAX_M: u32 = 2_000;
const MAX_C: u32 = MAX_M * MAX_M;

#[timing]
/// Without using rayon.
fn euclid() -> u32 {
    (1u32..MAX_M)
        .map(|m| -> u32 {
            (1..m)
                .filter(|n| (m - n).is_odd() && m.gcd(n) == 1)
                .map(|n| MAX_C / (m * m + n * n)) // Number of integer multipliers k that can create eligible triples from this primary triple pair (m,n)
                .sum() // Over this m and all eligible n
        })
        .sum() // Over all eligible m, n
}

#[timing]
/// Using rayon
fn par_euclid_weightless() -> u32 {
    (1u32..MAX_M)
        .into_par_iter()
        .map(|m| -> u32 {
            (1..m)
                .into_par_iter()
                .filter(|n| (m - n).is_odd() && m.gcd(n) == 1)
                .map(|n| MAX_C / (m * m + n * n))
                .sum()
        })
        .sum()
}

println!("How many Pythagorean triples exist less than or equal to 4 million?");
println!("i.e. a²+b²=c² and a,b,c ≤ {MAX_C}");
println!("rayon calculation gives {}.", par_euclid_weightless());
println!("Regular calculation gives {}.", euclid());
