# `category_enum` Procedural Macro Documentation

The `category_enum` procedural macro generates an enum named `Category` with predefined variants. This macro simplifies and standardizes the management of script categories in your project.

---

## Overview

The `category_enum` macro generates:

- An `enum` named `Category` with predefined variants.

- A `FromStr` implementation for converting strings to `Category`.

- A helper function `all_categories` that returns a list of all variants.

This allows you to:

- Use enums to represent script categories with compile-time checks.

- Easily iterate through all possible categories.

- Convert user input strings into category enum variants.

---

## Usage

Add the `category_enum` macro invocation in your project:

```rust
use demo_proc_macros::category_enum;

category_enum!();
```

---

## FromStr Implementation

You can convert strings to the Category enum:

```rust
use std::str::FromStr;

let category = Category::from_str("CLI").unwrap();
assert_eq!(category, Category::CLI);
```

---

## all_categories Helper

Get a list of all categories as a Vec<Category>:

```rust
let categories = Category::all_categories();
for category in categories {
    println!("{:?}", category);
}
```

---

## Features

1. **Predefined Enum Variants**: The enum includes the following variants:

```rust
AST
CLI
REPL
Async
Basic
BigNumbers
Crates
Educational
ErrorHandling
Exploration
Macros
Math
ProcMacros
Prototype
Recreational
Reference
Technique
Testing
Tools
TypeIdentification
```

2. **String Conversion**: Invalid strings will result in an error when using FromStr.

3. **Iterating Over Variants**: all_categories allows easy iteration and filtering based on categories.

---

## Example Use Case

### Filtering Scripts by Category

``` rust
use demo_proc_macros::category_enum;
use std::str::FromStr;

category_enum!();

fn main() {
    let input_categories = vec!["CLI", "Math", "Tools"];
    let selected: Vec<Category> = input_categories
        .iter()
        .filter_map(|&cat| Category::from_str(cat).ok())
        .collect();

    for category in selected {
        println!("Selected category: {:?}", category);
    }
}
```

---

## Error Handling
- If an invalid string is passed to Category::from_str, an Err is returned.

- Use proper error handling to manage user inputs.

---

## Limitations

- The macro is not configurable. It always generates the same predefined set of enum variants.

- For changes, modify the macro definition in the procedural macro crate.

---

## Contributing

If you have suggestions or want to add more categories, feel free to submit a pull request or open an issue on the repository.

---

## License

This crate is licensed under MIT and Apache 2. You may use either at your discretion.

---

## References

- Rust Procedural Macros
- Standard Library: FromStr

---
