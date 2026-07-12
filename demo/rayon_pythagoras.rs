/// Highly simplified version of the published benchmark from the `rayon` crate.
/// https://github.com/rayon-rs/rayon/blob/main/rayon-demo/src/pythagoras/mod.rs
//# Purpose: Demo the featured crate and the performance effect of using `rayon`.
//# Categories: crates, performance, timing
// How many Pythagorean triples exist less than or equal to a million?
// i.e. a²+b²=c² and a,b,c ≤ 1000000
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

#[timing]
/// Without using rayon.
fn euclid() -> u32 {
    (1u32..2_000)
        .map(|m| -> u32 {
            (1..m)
                .filter(|n| (m - n).is_odd() && m.gcd(n) == 1)
                .map(|n| 4_000_000 / (m * m + n * n))
                .sum()
        })
        .sum()
}

#[timing]
/// Using rayon
fn par_euclid_weightless() -> u32 {
    (1u32..2_000)
        .into_par_iter()
        .map(|m| -> u32 {
            (1..m)
                .into_par_iter()
                .filter(|n| (m - n).is_odd() && m.gcd(n) == 1)
                .map(|n| 4_000_000 / (m * m + n * n))
                .sum()
        })
        .sum()
}

par_euclid_weightless();
euclid();
