//: Demo of deriving Pythagorean triples.
//:
//: Pythagorean triples are integer tuples (a, b, c) such that a^2 + b^2 = c^2).
//: They represent right-angled triangles with all sides having integer length in a given unit of measure.
//:
//: They form a tree with the root at (3, 4, 5), with each triple having 3 child triples.
//:
//: Per the Wikipedia page, the standard derivation is based on the formulae:
//:
//:     1. a = m^2 - n^2
//:     2. b = 2mn
//:     3. c = m^2 + n^2
//:     where m > n > 0 and one is always even, the other always odd.
//:
//: The next 3 values of m and n, corresponding to the 3 child triples of (3, 4, 5) are
//: derived by the following 3 formulae:
//:
//:     (m1, n1) = (2m - n, m)
//:     (m2, n2) = (2m + n, m)
//:     (m3, n3) = (m + 2n, n)
//:
//: So let's work out the 3 child triples of (3, 4, 5).
//# Purpose: Recreational, educational.

println!("Pythagorean triples are sets of 3 integers a, b and c that correspond to the dimensions of a right-angled triangle.");
println!("As if it isn't neat enough that such triangles exist, they form a tree structure and you can derive them all from the classic (3, 4, 5) triangle.");
println!();
println!("Enter a Pythagorean triple, e.g. 3 4 5 or (3,4,5). NB don't change the order of the 3 integers.");
loop {
    println!("Enter the triple as 3 numbers, or q to quit, then press Ctrl-D on a new line. Ctrl-C will also quit.");

    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer)?;

    let s = buffer.trim();

    if s == "q" { break };

    let vec = s
        .trim_start_matches('(')
        .trim_end_matches(')')
        .replace(',', " ")
        .split_whitespace()
        .map(|x| x.parse::<usize>().expect("Parse error"))
        .collect::<Vec<usize>>();

    let (a, b, c): (usize, usize, usize) = (vec[0], vec[1], vec[2]);

    // let (a, b, c): (usize, usize, usize) = (3, 4, 5);
    // let (m, n): (usize, usize) = (2, 1);  // From above: a + c = 3 + 5 = 8, = 2m^2, thus m = 2. b = 4 = 2mn = 4n, thus n = 1.
    let m: usize = (((a + c) as f64) / 2.0).sqrt() as usize;  // From 1. & 3. above: a + c = 3 + 5 = 8, = 2m^2, thus m = sqrt(8/2).
    let n: usize = b / (2 * m);     // From 2. above.

    let (m1, n1): (usize, usize) = (2 * m - n, m);
    let (m2, n2): (usize, usize) = (2 * m + n, m);
    let (m3, n3): (usize, usize) = (m + 2 * n, n);

    let (p1, p2, p3) = ((m1, n1), (m2, n2), (m3, n3));

    // println!("The next 3 generating pairs are {p1:?}, {p2:?} and {p3:?}");
    println!();

    let (a1, b1, c1): (usize, usize, usize) = (m1.pow(2) - n1.pow(2), 2 * m1 * n1, m1.pow(2) + n1.pow(2));
    let (a2, b2, c2): (usize, usize, usize) = (m2.pow(2) - n2.pow(2), 2 * m2 * n2, m2.pow(2) + n2.pow(2));
    let (a3, b3, c3): (usize, usize, usize) = (m3.pow(2) - n3.pow(2), 2 * m3 * n3, m3.pow(2) + n3.pow(2));

    let (t, t1, t2, t3) = ((a, b, c), (a1, b1, c1), (a2, b2, c2), (a3, b3, c3));

    println!("Triple {t:?} has child triples {t1:?}, {t2:?} and {t3:?}");
    println!();
    println!("Would you like to try another triple? You can enter any valid triple that won't break the limits of Rust's usize");
}
