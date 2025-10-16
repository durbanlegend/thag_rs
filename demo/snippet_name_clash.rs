//: Demo scope of import statements. Two conflicting imports with the same name
//: `ubig` coexisting in the same `println!` invocation. Demonstrates that when
//: wrapping a snippet we can't assume it's OK to pull the imports up to the top
//: level.
//# Purpose: Prototype to confirm leaving imports in situ when wrapping snippets.
//# Categories: crates, learning
use dashu; // For dependency inference as alternative to toml block
use ibig; // For dependency inference as alternative to toml block

println!(
    "ibig UBig 123={}, dashu UBig 987={}",
    {
        use ibig::ubig;
        ubig!(123)
    },
    {
        use dashu::ubig;
        ubig!(987)
    }
);
