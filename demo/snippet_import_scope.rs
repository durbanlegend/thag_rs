/*[toml]
[dependencies]
ibig = "0.3.6"
*/

//: Demo scope of import statements.
//# Purpose: Prototype to confirm leaving imports in situ when wrapping snippets.
println!("ibi::Ubig with value 123={}", {
    use ibig::ubig; // This guarantees that this println will work without the other import.
    ubig!(123)
});

use ibig::ubig;     // Removal of this will cause the below to fail, but not the above.
// Expect this to fail without the above import of ibig::ubig, because
// the one inside the previous println is not in scope.
println!("Hoping to return ibig::UBig with value 987");
ubig!(987)
