const MAPPINGS: [(i32, &str, &str); 7] = [
    (10, "Key bindings", "Description"),
    (20, "q, Esc", "Close the file dialog"),
    (30, "j, ↓", "Move down in the file list"),
    (40, "k, ↑", "Move up in the file list"),
    (50, "Enter", "Select the current item"),
    (60, "u", "Move one directory up"),
    (70, "I", "Toggle showing hidden files"),
];
struct MyStruct;
impl MyStruct {
    pub fn adjust_mappings(&self) -> &'static [(i32, &'static str, &'static str)] {
        {
            println!("Base mappings from named constant:");
        };
        for (seq, key, desc) in MAPPINGS {
            println!("Seq: {0}, key: {1}, desc: {2}", seq, key, desc);
        }
        static ADJUSTED_MAPPINGS: std::sync::OnceLock<
            &'static [(i32, &'static str, &'static str)],
        > = std::sync::OnceLock::new();
        let adjusted_mappings = ADJUSTED_MAPPINGS.get_or_init(|| {
            const BASE_MAPPINGS: &[(i32, &'static str, &'static str)] = &MAPPINGS;
            let filtered_mappings: Vec<(i32, &'static str, &'static str)> = {
                let mut result = Vec::new();
                for mapping in BASE_MAPPINGS.iter() {
                    if !(mapping.1 == "I" || mapping.1 == "u") {
                        result.push(*mapping);
                    }
                }
                result
            };
            &[
                &filtered_mappings[..],
                (61i32, "u", "Up one"),
                (71i32, "I", "Toggle hidden"),
            ]
        });
        adjusted_mappings
    }
}
#[automatically_derived]
impl ::core::default::Default for MyStruct {
    #[inline]
    fn default() -> MyStruct {
        MyStruct {}
    }
}
fn main() {
    let my_struct: MyStruct = Default::default();
    my_struct.adjust_mappings();
}
