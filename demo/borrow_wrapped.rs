#[allow(unused_doc_comments)]
/// Snippet demonstrating how to reference or clone a wrapped value without
/// falling foul of the borrow checker.

//# Purpose: Demo a borrow-checker-friendly technique for accessing a wrapped value.
//# Categories: technique

#[derive(Clone, Debug)]
struct MyStruct {
    // fields
}

let optional_struct: Option<MyStruct> = Some(MyStruct { /* initialize fields */ });

// Borrow the value inside the Option without moving it
let borrowed_struct: Option<&MyStruct> = optional_struct.as_ref();

// Map over the borrowed value to apply a function or access its fields
if let Some(struct_ref) = borrowed_struct {
    // Use struct_ref here, it's a reference to the struct inside the Option
    // You can access its fields or pass it to functions without cloning
    println!("struct_ref={struct_ref:?}");
}

let cloned_optional_struct: Option<MyStruct> = optional_struct.clone();
println!("cloned_optional_struct={cloned_optional_struct:?}");
