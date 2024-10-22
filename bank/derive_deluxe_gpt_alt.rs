/*[toml]
[dependencies]
thag_proc_macros = { path = "/Users/donf/projects/thag_rs/src/proc_macros" }
*/

use thag_proc_macros::DeriveKeyMapList;

#[derive(DeriveKeyMapList, Default)]
#[deluxe(
    base = [
        (10, "key1", "desc1"),
        (20, "key2", "desc2"),
        (30, "key3", "desc3")],
    delete = ["key2", "key4"])]
struct MyStruct;

fn main() {
    let my_struct: MyStruct = Default::default();

    my_struct.print_values(); // This will print the overridden values.
}
