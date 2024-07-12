/*[toml]
[dependencies]
astro-float = "0.9.4"
*/
use astro_float::Consts;
use astro_float::RoundingMode;
use astro_float::ctx::Context;
use astro_float::expr;

// Create a context with precision 1024, rounding to the nearest even,
// and exponent range from -100000 to 100000.
let mut ctx = Context::new(1024, RoundingMode::ToEven,
    Consts::new().expect("Constants cache initialized"),
    -100000, 100000);

// Compute pi: pi = 6*arctan(1/sqrt(3))
let pi = expr!(6 * atan(1 / sqrt(3)), &mut ctx);

// Use library's constant value for verifying the result.
let pi_lib = ctx.const_pi();

// Compare computed constant with library's constant
assert_eq!(pi.cmp(&pi_lib), Some(0));

// Print using decimal radix.
println!("{}", pi);

// output: 3.14159265358979323846264338327950288419716939937510582097494459
//
