use std::iter::successors;

let fib = successors(Some((0, 1)), |&(a, b)| Some((b, a + b)));

for (n, _) in fib.take(20) {
    println!("{:?}", n);
}
