use thag_demo_proc_macros::my_attribute;

#[my_attribute]
fn my_function() {
    let not_in_use = "abc";
    println!("Hello, world!");
}

my_function();
