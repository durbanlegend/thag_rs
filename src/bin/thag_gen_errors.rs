/*[toml]
[dependencies]
heck = "0.5.0"
inquire = "0.7.5"
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["core", "simplelog"] }
*/

/// Quick and easy prompted generator for new custom error types and new variants required
/// by existing custom error types. Prompts for the new or existing custom error type, the
/// new variants, any types wrapped by the new variants, and any special display messages.
/// The output can be saved to a new error module in the case of a new custom error type,
/// or simply copied and pasted in sections from the output into an existing error module
/// in the case of an existing custom error type.
///
/// Strategy and grunt work thanks to `ChatGPT`.
//# Purpose: Facilitate generation and enhancement of custom error modules.
//# Categories: technique, tools
use heck::ToSnakeCase;
use inquire::{set_global_render_config, Confirm, MultiSelect, Select, Text};
use std::fmt::Write as _; // import without risk of name clashing
use std::{error::Error, fs, path::PathBuf};
use thag_rs::{auto_help, help_system::check_help_and_exit, themed_inquire_config};

#[derive(Debug)]
struct ErrorVariant {
    name: String,
    wrapped_type: Option<String>,
    display_message: String,
}

impl ErrorVariant {
    fn new_interactive() -> Result<Self, Box<dyn Error>> {
        let name = Text::new("Variant name:")
            .with_help_message("Enter name for the error variant (e.g. DatabaseError)")
            .prompt()?;

        let has_wrapped = Confirm::new("Does this variant wrap another type?")
            .with_default(false)
            .prompt()?;

        let wrapped_type = if has_wrapped {
            Some(
                Text::new("Wrapped type:")
                    .with_help_message("Enter the type to wrap (e.g. sqlx::Error)")
                    .prompt()?,
            )
        } else {
            None
        };

        let display_message = Text::new("Display message:")
            .with_help_message("Enter the error message format (use {} for wrapped value)")
            .with_default(if wrapped_type.is_some() {
                "{}"
            } else {
                "An error occurred"
            })
            .prompt()?;

        Ok(Self {
            name,
            wrapped_type,
            display_message,
        })
    }

    fn edit_interactive(&mut self) -> Result<(), Box<dyn Error>> {
        let field = Select::new(
            "Which field would you like to edit?",
            vec!["Name", "Wrapped type", "Display message", "Cancel"],
        )
        .prompt()?;

        match field {
            "Name" => {
                self.name = Text::new("New variant name:")
                    .with_help_message("Enter name for the error variant (e.g. DatabaseError)")
                    .with_default(&self.name)
                    .prompt()?;
            }
            "Wrapped type" => {
                let has_wrapped = Confirm::new("Should this variant wrap another type?")
                    .with_default(self.wrapped_type.is_some())
                    .prompt()?;

                self.wrapped_type = if has_wrapped {
                    Some(
                        Text::new("New wrapped type:")
                            .with_help_message("Enter the type to wrap (e.g. sqlx::Error)")
                            .with_default(self.wrapped_type.as_deref().unwrap_or(""))
                            .prompt()?,
                    )
                } else {
                    None
                };
            }
            "Display message" => {
                self.display_message = Text::new("New display message:")
                    .with_help_message("Enter the error message format (use {} for wrapped value)")
                    .with_default(&self.display_message)
                    .prompt()?;
            }
            "Cancel" => return Ok(()),
            _ => unreachable!(),
        }
        Ok(())
    }

    fn display_summary(&self) -> String {
        self.wrapped_type.as_ref().map_or_else(
            || format!(r#"{} - "{}""#, self.name, self.display_message),
            |wrapped| {
                format!(
                    r#"{} ({}) - "{}""#,
                    self.name, wrapped, self.display_message
                )
            },
        )
    }
}

#[derive(Debug)]
struct ErrorModule {
    name: String,
    variants: Vec<ErrorVariant>,
}

const COMMON_ERRORS: &[(&str, &str, Option<&str>)] = &[
    ("IoError", "IO operation failed: {}", Some("std::io::Error")),
    ("ParseError", "Failed to parse: {}", Some("String")),
    ("ValidationError", "Validation failed: {}", Some("String")),
    ("NotFound", "Resource not found", None),
    ("Custom", "{}", Some("String")),
];

fn review_and_edit_variants(variants: &mut Vec<ErrorVariant>) -> Result<(), Box<dyn Error>> {
    loop {
        let choices = variants
            .iter()
            .map(ErrorVariant::display_summary)
            .chain(std::iter::once("Done editing".to_string()))
            .collect::<Vec<_>>();

        let selected =
            Select::new("Review and edit variants (select one to edit):", choices).prompt()?;

        if selected == "Done editing" {
            break;
        }

        // Find the selected variant
        let idx = variants
            .iter()
            .position(|v| v.display_summary() == selected)
            .expect("Selected variant not found");

        let action = Select::new(
            "What would you like to do with this variant?",
            vec!["Edit", "Delete", "Cancel"],
        )
        .prompt()?;

        match action {
            "Edit" => {
                variants[idx].edit_interactive()?;
            }
            "Delete" => {
                if Confirm::new(&format!(
                    "Are you sure you want to delete {}?",
                    variants[idx].name
                ))
                .with_default(false)
                .prompt()?
                {
                    variants.remove(idx);
                }
            }
            "Cancel" => (),
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn get_save_location(module_name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let default_filename = format!("{}.rs", module_name.to_snake_case());

    let dir_choice = Select::new(
        "Select directory to save the error module:",
        vec!["Current directory", "src", "Custom location"],
    )
    .prompt()?;

    let dir_path = match dir_choice {
        "Current directory" => PathBuf::from("."),
        "src" => {
            let src_dir = PathBuf::from("src");
            if !src_dir.exists() {
                fs::create_dir(&src_dir)?;
            }
            src_dir
        }
        "Custom location" => {
            let input = Text::new("Enter directory path:")
                .with_default(".")
                .prompt()?;
            let path = PathBuf::from(input);
            if !path.exists() {
                fs::create_dir_all(&path)?;
            }
            path
        }
        _ => unreachable!(),
    };

    let filename = Text::new("Enter filename:")
        .with_default(&default_filename)
        .prompt()?;

    let full_path = dir_path.join(filename);

    // Check if file exists
    if full_path.exists() {
        let overwrite = Confirm::new(&format!(
            "File {} already exists. Overwrite?",
            full_path.display()
        ))
        .with_default(false)
        .prompt()?;

        if !overwrite {
            return get_save_location(module_name); // Recurse to try again
        }
    }

    Ok(full_path)
}

#[allow(clippy::too_many_lines)]
fn generate_tests(module: &ErrorModule) -> String {
    let mut output = String::new();

    output.push_str("#[cfg(test)]\nmod tests {\n");
    output.push_str("    use super::*;\n\n");

    // Test Display implementations
    output.push_str("    #[test]\n");
    output.push_str("    fn test_display() {\n");

    for variant in &module.variants {
        output.push_str("        assert_eq!(\n");
        if let Some(wrapped) = &variant.wrapped_type {
            if wrapped == "std::io::Error" {
                let _ = writeln!(
                    output,
                    r#"            {}::{}(std::io::Error::new(std::io::ErrorKind::Other, "test error")).to_string(),
"#,
                    module.name, variant.name
                );
                // Use the actual display message format
                let _ = writeln!(
                    output,
                    r#"            "{}"
"#,
                    variant.display_message.replace("{}", "test error")
                );
            } else if wrapped == "String" {
                let _ = writeln!(
                    output,
                    r#"            {}::{}("test error".to_string()).to_string(),
"#,
                    module.name, variant.name
                );
                // Use the actual display message format
                let _ = writeln!(
                    output,
                    r#"            "{}"
"#,
                    variant.display_message.replace("{}", "test error")
                );
            } else {
                // Add comment for custom wrapped types
                let _ = writeln!(
                    output,
                    "            // TODO: Provide appropriate test value for {wrapped}\n"
                );
                let _ = writeln!(
                    output,
                    "            // {}::{}(your_test_value).to_string(),\n",
                    module.name, variant.name
                );
                output.push_str(
                    r#"            // "test error"  // TODO: Review expected output
"#,
                );
                output.push_str("        );\n");
                continue;
            }
        } else {
            let _ = writeln!(
                output,
                "            {}::{}.to_string(),\n",
                module.name, variant.name
            );
            // For variants without wrapped types, use the display message directly
            let _ = writeln!(
                output,
                r#"            "{}"
"#,
                variant.display_message
            );
        }
        output.push_str("        );\n");
    }
    output.push_str("    }\n\n");

    // Test From implementations
    output.push_str("    #[test]\n");
    output.push_str("    fn test_from_implementations() {\n");
    for variant in &module.variants {
        if let Some(wrapped) = &variant.wrapped_type {
            match wrapped.as_str() {
                "std::io::Error" => {
                    output.push_str(r#"        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "test error");
"#);
                    let _ = writeln!(
                        output,
                        "        let error = {}::{}(io_error);\n",
                        module.name, variant.name
                    );
                    let _ = writeln!(
                        output,
                        "        assert!(matches!(error, {}::{}(_)));\n",
                        module.name, variant.name
                    );
                }
                "String" => {
                    output.push_str(
                        r#"        let string_error = "test error".to_string();
"#,
                    );
                    let _ = writeln!(
                        output,
                        "        let error = {}::{}(string_error);\n",
                        module.name, variant.name
                    );
                    let _ = writeln!(
                        output,
                        "        assert!(matches!(error, {}::{}(_)));\n",
                        module.name, variant.name
                    );
                }
                _ => {
                    let _ = writeln!(
                        output,
                        "        // TODO: Add test for {wrapped} wrapped type\n"
                    );
                }
            }
            output.push('\n');
        }
    }
    output.push_str("    }\n");
    output.push_str("}\n");

    output
}

fn main() -> Result<(), Box<dyn Error>> {
    // Check for help first - automatically extracts from source comments
    let help = auto_help!("thag_gen_errors");
    check_help_and_exit(&help);

    set_global_render_config(themed_inquire_config());

    let enum_name = Text::new("Error Enum name:")
        .with_default("MyError")
        .with_help_message("New or existing enum to hold the error types as variants")
        .prompt()?;

    // First, select from common errors
    let mut variants: Vec<ErrorVariant> = {
        let selected = MultiSelect::new(
            "Select common error variants:",
            COMMON_ERRORS.iter().map(|(name, _, _)| *name).collect(),
        )
        .prompt()?;

        selected
            .iter()
            .filter_map(|&name| {
                COMMON_ERRORS
                    .iter()
                    .find(|(n, _, _)| *n == name)
                    .map(|(_, msg, wrapped)| ErrorVariant {
                        name: name.to_string(),
                        wrapped_type: wrapped.map(String::from),
                        display_message: (*msg).to_string(),
                    })
            })
            .collect()
    };

    // Then, allow adding custom variants
    loop {
        let add_custom = Confirm::new("Would you like to add a custom error variant?")
            .with_default(false)
            .prompt()?;

        if !add_custom {
            break;
        }

        match ErrorVariant::new_interactive() {
            Ok(variant) => {
                // Check for duplicate names
                if variants.iter().any(|v| v.name == variant.name) {
                    println!("Error: A variant with that name already exists.");
                    continue;
                }
                variants.push(variant);
            }
            Err(e) => {
                println!("Error creating variant: {e}");
            }
        }
    }

    if variants.is_empty() {
        println!("Warning: No error variants selected or created.");
        return Ok(());
    }

    if Confirm::new("Would you like to review and edit the variants?")
        .with_default(true)
        .prompt()?
    {
        review_and_edit_variants(&mut variants)?;
    }

    let module = ErrorModule {
        name: enum_name.clone(),
        variants,
    };

    let mut code = generate_error_module(&module);

    // Add tests if requested
    if Confirm::new("Would you like to generate unit tests?")
        .with_default(true)
        .prompt()?
    {
        code.push('\n');
        code.push_str(&generate_tests(&module));
    }

    println!("Generated code:\n{code}");

    // Save the file
    if Confirm::new("Would you like to save this code to a file?")
        .with_default(true)
        .prompt()?
    {
        let path = get_save_location(&enum_name)?;
        fs::write(&path, code)?;
        println!("Code saved to: {}", path.display());
    }

    Ok(())
}

fn generate_error_module(module: &ErrorModule) -> String {
    let mut output = String::new();

    // Generate enum
    output.push_str("#[derive(Debug)]\n");
    let _ = writeln!(output, "pub enum {} {{\n", module.name);
    for variant in &module.variants {
        if let Some(wrapped) = &variant.wrapped_type {
            let _ = writeln!(output, "    {}({}),\n", variant.name, wrapped);
        } else {
            let _ = writeln!(output, "    {},\n", variant.name);
        }
    }
    output.push_str("}\n\n");

    // Generate From impls for wrapped types
    for variant in &module.variants {
        if let Some(wrapped) = &variant.wrapped_type {
            if wrapped != "String" {
                // Skip String as it's handled differently
                let _ = writeln!(output, "impl From<{wrapped}> for {} {{\n", module.name);
                let _ = writeln!(output, "    fn from(err: {wrapped}) -> Self {{\n");
                let _ = writeln!(output, "        Self::{}(err)\n", variant.name);
                output.push_str("    }\n");
                output.push_str("}\n\n");
            }
        }
    }

    // Generate Display impl
    let _ = writeln!(output, "impl std::fmt::Display for {} {{\n", module.name);
    output.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
    output.push_str("        match self {\n");
    for variant in &module.variants {
        if let Some(_wrapped) = &variant.wrapped_type {
            let _ = writeln!(
                output,
                r#"            Self::{}(e) => write!(f, "{}", e),
"#,
                variant.name, variant.display_message
            );
        } else {
            let _ = writeln!(
                output,
                r#"            Self::{} => write!(f, "{}"),
"#,
                variant.name, variant.display_message
            );
        }
    }
    output.push_str("        }\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Generate Error impl
    let _ = writeln!(output, "impl std::error::Error for {} {{\n", module.name);
    output.push_str("    fn source(&self) -> Option<&(dyn Error + 'static)> {\n");
    output.push_str("        match self {\n");
    for variant in &module.variants {
        if let Some(_wrapped) = &variant.wrapped_type {
            let _ = writeln!(
                output,
                "            Self::{}(e) => Some(e),\n",
                variant.name
            );
        } else {
            let _ = writeln!(output, "           Self::{} => Some(self),\n", variant.name);
        }
    }
    output.push_str("        }\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    output
}
