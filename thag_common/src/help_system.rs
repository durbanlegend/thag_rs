//! Lightweight help system for thag tools
//!
//! This module provides a simple, consistent help system for tools that don't use clap.
//! It extracts help information from source code comments and provides formatted output.

use std::env;
use std::ffi::OsStr;
use std::fmt;

/// A lightweight help system that extracts information from source comments
pub struct HelpSystem {
    /// Tool name (extracted from binary name or provided)
    pub tool_name: String,
    /// Purpose from //# Purpose: comment
    pub purpose: Option<String>,
    /// Description from /// doc comments
    pub description: Option<String>,
    /// Usage examples from //# Usage: comment
    pub usage: Option<String>,
    /// Categories from //# Categories: comment
    pub categories: Vec<String>,
    /// Version information
    pub version: Option<String>,
}

impl Default for HelpSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl HelpSystem {
    /// Create a new help system with the tool name
    #[must_use]
    pub fn new() -> Self {
        let tool_name = program_name();

        Self {
            tool_name,
            purpose: None,
            description: None,
            usage: None,
            categories: Vec::new(),
            version: None,
        }
    }

    /// Create help system from current source file (for use with `auto_help!` macro)
    #[must_use]
    pub fn from_current_source(file_path: &str) -> Self {
        // Try to read the source file from the most likely locations
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

        // The file path from file!() macro is relative to the crate root
        let full_path = std::path::Path::new(&manifest_dir).join(file_path);

        std::fs::read_to_string(&full_path)
            .map_or_else(|_| Self::new(), |source| Self::from_source(&source))
    }

    /// Set the purpose
    #[must_use]
    pub fn with_purpose(mut self, purpose: impl Into<String>) -> Self {
        self.purpose = Some(purpose.into());
        self
    }

    /// Set the description
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set usage examples
    #[must_use]
    pub fn with_usage(mut self, usage: impl Into<String>) -> Self {
        self.usage = Some(usage.into());
        self
    }

    /// Set categories
    #[must_use]
    pub fn with_categories(mut self, categories: Vec<String>) -> Self {
        self.categories = categories;
        self
    }

    /// Set version
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Check if help was requested and display it if so
    #[must_use]
    pub fn check_help(&self) -> bool {
        let args: Vec<String> = env::args().collect();

        if args.len() > 1 {
            let help_args = ["--help", "-h", "help"];
            if help_args.contains(&args[1].as_str()) {
                println!("{self}");
                return true;
            }
        }
        false
    }

    /// Parse help information from source code
    #[must_use]
    pub fn from_source(source: &str) -> Self {
        let mut help = Self::new();

        let mut doc_lines = Vec::new();
        let mut in_doc_comment = false;

        for line in source.lines() {
            let line = line.trim();

            // Handle special comments
            if let Some(stripped) = line.strip_prefix("//#") {
                let parts: Vec<&str> = stripped.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let keyword = parts[0].trim();
                    let value = parts[1].trim();

                    match keyword.to_lowercase().as_str() {
                        "purpose" => help.purpose = Some(value.to_string()),
                        "usage" => help.usage = Some(value.to_string()),
                        "categories" => {
                            help.categories =
                                value.split(',').map(|cat| cat.trim().to_string()).collect();
                        }
                        _ => {}
                    }
                }
            }
            // Handle doc comments
            else if line.starts_with("///") {
                if let Some(content) = line.strip_prefix("///") {
                    let content = content.trim();
                    doc_lines.push(content);
                    in_doc_comment = true;
                }
            }
            // Stop collecting doc comments when we hit non-doc content
            else if in_doc_comment && !line.is_empty() && !line.starts_with("//") {
                break;
            }
        }

        if !doc_lines.is_empty() {
            help.description = Some(doc_lines.join("\n"));
            // eprintln!("help.description={:#?}", help.description);
        }

        help
    }

    /// Create help from a file path (reads and parses the file)
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o errors encountered trying to read the file.
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let source = std::fs::read_to_string(file_path)?;
        Ok(Self::from_source(&source))
    }
}

impl fmt::Display for HelpSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} {}",
            self.tool_name,
            self.version.as_deref().unwrap_or("")
        )?;

        if let Some(purpose) = &self.purpose {
            writeln!(f, "{purpose}")?;
        }

        if let Some(description) = &self.description {
            writeln!(f, "\n{description}")?;
        }

        writeln!(f, "\nUSAGE:")?;
        if let Some(usage) = &self.usage {
            writeln!(f, "    {usage}")?;
        } else {
            writeln!(f, "    {} [OPTIONS]", self.tool_name)?;
        }

        writeln!(f, "\nOPTIONS:")?;
        writeln!(f, "    -h, --help       Print help")?;

        if !self.categories.is_empty() {
            writeln!(f, "\nCATEGORIES: {}", self.categories.join(", "))?;
        }

        Ok(())
    }
}

/// Macro to create a help system - manually specify the source
#[macro_export]
macro_rules! help_system {
    ($source:expr) => {{
        $crate::help_system::HelpSystem::from_source($source)
            .with_version(env!("CARGO_PKG_VERSION"))
    }};

    // Simplified version - just create with tool name
    ($tool_name:expr) => {{
        $crate::help_system::HelpSystem::new().with_version(env!("CARGO_PKG_VERSION"))
    }};
}

/// Convenience function to check for help and exit if found
pub fn check_help_and_exit(help: &HelpSystem) {
    if help.check_help() {
        std::process::exit(0);
    }
}

/// Returns the stem (filename without extension) of the currently running executable.
/// Falls back to `"unknown"` if the path can't be resolved or isn't valid UTF-8.
#[must_use]
pub fn program_name() -> String {
    env::current_exe()
        .ok()
        .and_then(|p| p.file_stem().map(OsStr::to_os_string))
        .and_then(|os| os.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Macro to automatically extract help from current source file
#[macro_export]
macro_rules! auto_help {
    () => {{
        $crate::help_system::HelpSystem::from_current_source(file!())
            .with_version(env!("CARGO_PKG_VERSION"))
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_parsing() {
        let source = r#"
/// This is a test tool
/// that does amazing things
//# Purpose: Testing the help system
//# Categories: test, utility
//# Usage: test_tool [OPTIONS] <file>

fn main() {
    println!("Hello world");
}
"#;

        let help = HelpSystem::from_source(source);
        assert_eq!(help.purpose, Some("Testing the help system".to_string()));
        assert_eq!(help.categories, vec!["test", "utility"]);
        assert_eq!(
            help.description,
            Some("This is a test tool\nthat does amazing things".to_string())
        );
    }

    #[test]
    fn test_help_display() {
        let help = HelpSystem::new()
            .with_purpose("A test tool")
            .with_description("Does testing things")
            .with_version("1.0.0");

        let output = format!("{}", help);
        assert!(output.contains("test_tool 1.0.0"));
        assert!(output.contains("A test tool"));
        assert!(output.contains("Does testing things"));
    }
}
