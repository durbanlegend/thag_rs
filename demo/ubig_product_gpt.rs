/*[toml]
[dependencies]
ibig = "0.3.6"
*/
use ibig::{ubig, UBig};
use std::iter::Product;
use std::ops::{Deref, DerefMut};

// Step 1: Define the Wrapper Type
#[derive(Debug, Clone)]
struct UBigWrapper(UBig);

// Step 2: Implement Deref and DerefMut
impl Deref for UBigWrapper {
    type Target = UBig;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UBigWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Step 3: Implement the Product Trait
impl Product for UBigWrapper {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(UBigWrapper(ubig!(1)), |acc, x| UBigWrapper(acc.0 * x.0))
    }
}

impl<'a> Product<&'a UBigWrapper> for UBigWrapper {
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(UBigWrapper(ubig!(1)), |acc, x| {
            UBigWrapper(acc.0.clone() * x.0.clone())
        })
    }
}

// Example Usage
fn main() {
    let nums: Vec<UBigWrapper> = vec![
        UBigWrapper(ubig!(2)),
        UBigWrapper(ubig!(3)),
        UBigWrapper(ubig!(4)),
    ];

    let product: UBigWrapper = nums.iter().product();
    println!("Product: {}", product.0); // Should print: Product: 24
}
