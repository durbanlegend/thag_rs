/*[toml]
[dependencies]
derive-deftly = "0.14.2"
*/

use derive_deftly::define_derive_deftly;

define_derive_deftly! {
    Discriminant:

    #[derive(Copy,Clone,Eq,PartialEq,Debug)]
    enum ${paste $tname Discriminant} {
        $(
            $vname,
        )
    }

    impl<$tgens> $ttype where $twheres {
       fn discriminant(&self) -> ${paste $tname Discriminant} {
          match self {
              $(
                  $vpat => ${paste $tname Discriminant}::$vname,
              )
          }
        }

        $(
            fn ${paste is_ ${snake_case $vname}} (&self) -> bool {
                self.discriminant() ==
                    ${paste $tname Discriminant} ::$vname
            }
        )
    }
}

use derive_deftly::Deftly;
// use derive_deftly_template_Discriminant;
#[derive(Debug, Deftly)]
#[derive_deftly(Discriminant)]
enum AllTypes {
    NoData,
    Tuple(u8, u16),
    Struct { a: String, b: String },
}

for v in [AllTypes::NoData, AllTypes::Tuple(5, 8), AllTypes::Struct { a: "a".to_string(), b: "b".to_string() }] {
    let d = v.discriminant();
    println!("variant.discriminant()={:#?}, is_no_data? {}, is_tuple? {}, is_struct? {}", d, v.is_no_data(), v.is_tuple(), v.is_struct());
}
