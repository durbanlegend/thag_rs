/// Very simple demo of the `unzip` iterator function.
//# Purpose: Demo
let (v, r): (Vec<i32>, Vec<i32>) = (-1..13).map(|x| (x, x * x)).unzip();

println!("v={v:#?}");
println!("r={r:#?}");
