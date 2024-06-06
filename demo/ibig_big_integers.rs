/*[toml]
[dependencies]
ibig = "0.3.6"
*/

use ibig::{ibig, modular::ModuloRing, ubig, UBig};

let a = ubig!(12345678);
let b = ubig!(0x10ff);
let c = ibig!(-azz base 36);
let d: UBig = "15033211231241234523452345345787".parse()?;
let e = 2 * &b + 1;
let f = a * b.pow(10);

assert_eq!(e, ubig!(0x21ff));
assert_eq!(c.to_string(), "-14255");
assert_eq!(
    f.in_radix(16).to_string(),
    "1589bda8effbfc495d8d73c83d8b27f94954e"
);
assert_eq!(
    format!("hello {:#x}", d % ubig!(0xabcd1234134132451345)),
    "hello 0x1a7e7c487267d2658a93"
);

let ring = ModuloRing::new(&ubig!(10000));
let x = ring.from(12345);
let y = ring.from(55443);
assert_eq!(format!("{}", x - y), "6902 (mod 10000)");
