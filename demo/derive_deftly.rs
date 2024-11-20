/*[toml]
[dependencies]
derive-deftly = "0.14.2"
*/
/// Introductory example from the `derive-deftly` user guide.
//# Purpose: Explore proc macro alternatives.
//# Categories: crates, exploration, technique
use derive_deftly::define_derive_deftly;

define_derive_deftly! {
    HelloWorld:

    impl $ttype {
        pub fn greet() {
            println!("Greetings from {}", stringify!($ttype));
        }
    }
}

use derive_deftly::Deftly;

#[derive(Clone, Debug, Deftly)]
#[derive_deftly(HelloWorld)]
pub struct MyStruct;

MyStruct::greet();
