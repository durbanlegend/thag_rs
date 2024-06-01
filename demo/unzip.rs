fn main() {
    let (v, r): (Vec<i32>, Vec<i32>) = (-1..13).map(|x| (x, x * x)).unzip();

    println!("v={v:#?}");
    println!("r={r:#?}");
}
