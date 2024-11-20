/*[toml]
[dependencies]
dashu = "0.4.2"
ibig = "0.3.6"
*/

//: Demo scope of import statements. Two conflicting imports with the same name
//: coexisting in the same println! invocation. Demonstrates that when wrapping
//: a snippet we can't assume it's OK to pull the imports up to the top level.
//# Purpose: Prototype to confirm leaving imports in situ when wrapping snippets.
//# Categories: crates, educational
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
