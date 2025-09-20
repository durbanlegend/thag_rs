use criterion::{criterion_group, criterion_main, Criterion};
use lazy_static::lazy_static;
use phf::phf_set;
use regex::Regex;
use std::collections::HashSet;
use std::hint::black_box;

// Approach 1: phf::Set with all terms
static PHF_FILTER: phf::Set<&'static str> = phf_set! {
    "f32", "f64",
    "i8", "i16", "i32", "i64", "i128", "isize",
    "u8", "u16", "u32", "u64", "u128", "usize",
    "bool", "str",
    "error", "fs",
    "self", "super", "crate"
};

// Approach 2: Static array with binary search
static ARRAY_FILTER: &[&str] = &[
    "bool", "crate", "f32", "f64", "fs", "i8", "i16", "i32", "i64", "i128", "isize", "self", "str",
    "super", "u8", "u16", "u32", "u64", "u128", "usize",
];

// Approach 3: Regex + smaller phf::Set
lazy_static! {
    static ref NUMERIC_RE: Regex = Regex::new(r"^[fiu]\d{1,3}$").unwrap();
    static ref HASHSET_FILTER: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.extend(["bool", "str", "error", "fs", "self", "super", "crate"]);
        set
    };
}

fn filter_phf(word: &str) -> bool {
    PHF_FILTER.contains(word)
}

fn filter_array(word: &str) -> bool {
    ARRAY_FILTER.binary_search(&word).is_ok()
}

fn filter_regex_plus_hashset(word: &str) -> bool {
    NUMERIC_RE.is_match(word) || HASHSET_FILTER.contains(word)
}

fn criterion_benchmark(c: &mut Criterion) {
    // Test cases that should and shouldn't match
    let test_words = [
        "f32",
        "i64",
        "u128", // numerics
        "bool",
        "str",
        "self", // keywords
        "serde",
        "tokio",
        "rand", // real crates
        "something",
        "else",
        "entirely", // non-matches
    ];

    c.bench_function("phf_set", |b| {
        b.iter(|| {
            for word in test_words.iter() {
                black_box(filter_phf(black_box(word)));
            }
        })
    });

    c.bench_function("static_array", |b| {
        b.iter(|| {
            for word in test_words.iter() {
                black_box(filter_array(black_box(word)));
            }
        })
    });

    c.bench_function("regex_plus_hashset", |b| {
        b.iter(|| {
            for word in test_words.iter() {
                black_box(filter_regex_plus_hashset(black_box(word)));
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
