pub fn concat_extend<'a>(a: &'a [u8], b: &'a [u8]) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::from(a);
    v.extend_from_slice(b);
    v
}

let a: &[u8] = &[1, 2, 3];
let b: &[u8] = &[4, 5, 6, 7];

println!("concat_extend={:#?}", concat_extend(a, b));
