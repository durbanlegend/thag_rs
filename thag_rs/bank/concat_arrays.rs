pub fn concat_copy(a: [u8; 3], b: [u8; 4]) -> [u8; 7] {
    let mut ary = [0u8; 7];
    ary[..3].copy_from_slice(&a);
    ary[3..].copy_from_slice(&b);
    ary
}

pub fn concat_iter(a: [u8; 3], b: [u8; 4]) -> [u8; 7] {
    let mut iter = a.into_iter().chain(b);
    std::array::from_fn(|_| iter.next().unwrap())
}

let a: [u8; 3] = [1, 2, 3];
let b: [u8; 4] = [4, 5, 6, 7];

println!("concat_copy={:#?}", concat_copy(a, b));
println!("concat_iter={:#?}", concat_iter(a, b));
