use inquire::{MultiSelect, Text};
use std::error::Error;

#[derive(Debug)]
struct ErrorVariant {
    name: String,
    wrapped_type: Option<String>,
    display_message: String,
}

#[derive(Debug)]
struct ErrorModule {
    name: String,
    variants: Vec<ErrorVariant>,
}

// Common error patterns we'll offer
const COMMON_ERRORS: &[(&str, &str, Option<&str>)] = &[
    ("IoError", "IO operation failed: {}", Some("std::io::Error")),
    ("ParseError", "Failed to parse: {}", Some("String")),
    ("ValidationError", "Validation failed: {}", Some("String")),
    ("NotFound", "Resource not found", None),
    ("Custom", "{}", Some("String")),
];

fn main() -> Result<(), Box<dyn Error>> {
    let module_name = Text::new("Error module name:")
        .with_default("MyError")
        .prompt()?;

    let selected = MultiSelect::new(
        "Select error variants:",
        COMMON_ERRORS.iter().map(|(name, _, _)| *name).collect(),
    )
    .prompt()?;

    let variants: Vec<ErrorVariant> = selected
        .iter()
        .filter_map(|&name| {
            COMMON_ERRORS
                .iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, msg, wrapped)| ErrorVariant {
                    name: name.to_string(),
                    wrapped_type: wrapped.map(String::from),
                    display_message: msg.to_string(),
                })
        })
        .collect();

    let module = ErrorModule {
        name: module_name,
        variants,
    };

    let code = generate_error_module(&module);
    println!("Generated code:\n{}", code);

    Ok(())
}

fn generate_error_module(module: &ErrorModule) -> String {
    let mut output = String::new();

    // Generate enum
    output.push_str("#[derive(Debug)]\n");
    output.push_str(&format!("pub enum {} {{\n", module.name));
    for variant in &module.variants {
        if let Some(wrapped) = &variant.wrapped_type {
            output.push_str(&format!("    {}({}),\n", variant.name, wrapped));
        } else {
            output.push_str(&format!("    {},\n", variant.name));
        }
    }
    output.push_str("}\n\n");

    // Generate Display impl
    output.push_str(&format!("impl std::fmt::Display for {} {{\n", module.name));
    output.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
    output.push_str("        match self {\n");
    for variant in &module.variants {
        if let Some(_wrapped) = &variant.wrapped_type {
            output.push_str(&format!(
                "            {}::{}(arg) => write!(f, \"{}\", arg),\n",
                module.name, variant.name, variant.display_message
            ));
        } else {
            output.push_str(&format!(
                "            {}::{} => write!(f, \"{}\"),\n",
                module.name, variant.name, variant.display_message
            ));
        }
    }
    output.push_str("        }\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Generate Error impl
    output.push_str(&format!(
        "impl std::error::Error for {} {{}}\n\n",
        module.name
    ));

    // Generate From impls for wrapped types
    for variant in &module.variants {
        if let Some(wrapped) = &variant.wrapped_type {
            if wrapped != "String" {
                // Skip String as it's handled differently
                output.push_str(&format!("impl From<{}> for {} {{\n", wrapped, module.name));
                output.push_str(&format!("    fn from(err: {}) -> Self {{\n", wrapped));
                output.push_str(&format!("        {}::{}(err)\n", module.name, variant.name));
                output.push_str("    }\n");
                output.push_str("}\n\n");
            }
        }
    }

    output
}
