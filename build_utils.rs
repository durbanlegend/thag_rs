use std::env;
use std::fmt;
use std::fs;
use std::io::{self};
use std::path::Path;
use std::path::PathBuf;

use toml::Value;

// Custom error type for build script
#[derive(Debug)]
#[allow(dead_code)]
pub enum BuildError {
    Io {
        source: io::Error,
    },
    Env {
        source: env::VarError,
    },
    InvalidFileName {
        path: PathBuf,
    },
    // InvalidUtf8 { path: PathBuf },
    MissingField {
        field: String,
        path: PathBuf,
    },
    MissingStyle {
        style: String,
        path: PathBuf,
    },
    InvalidValue {
        field: String,
        value: String,
        path: PathBuf,
    },
}

// Implement Display for better error messages
impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::Io { source } => write!(f, "IO error: {source}"),
            BuildError::Env { source } => write!(f, "Environment error: {source}"),
            BuildError::InvalidFileName { path } => {
                write!(f, "Invalid filename: {}", path.display())
            }
            BuildError::MissingField { field, path } => {
                write!(f, "Missing field {field}: {}", path.display())
            }
            BuildError::MissingStyle { style, path } => {
                write!(f, "Missing style {style}: {}", path.display())
            }
            BuildError::InvalidValue { field, value, path } => {
                write!(
                    f,
                    "Invalid value {value} for field {field}: {}",
                    path.display()
                )
            } // BuildError::InvalidUtf8 { path } => {
              //     write!(f, "Invalid UTF-8 in filename: {}", path.display())
              // }
        }
    }
}

impl From<io::Error> for BuildError {
    fn from(source: io::Error) -> Self {
        BuildError::Io { source }
    }
}

impl From<env::VarError> for BuildError {
    fn from(source: env::VarError) -> Self {
        BuildError::Env { source }
    }
}

pub type BuildResult<T> = Result<T, BuildError>;

pub fn validate_theme_file(path: &Path) -> Result<(), BuildError> {
    let content = fs::read_to_string(path).map_err(|e| BuildError::Io { source: e })?;

    let theme: Value = content.parse::<Value>().map_err(|e| BuildError::Io {
        source: io::Error::new(io::ErrorKind::InvalidData, e),
    })?;

    // Validate required top-level fields
    for field in [
        "name",
        "description",
        "term_bg_luma",
        "min_color_support",
        "palette",
    ] {
        if !theme.get(field).is_some() {
            return Err(BuildError::MissingField {
                field: field.to_string(),
                path: path.to_owned(),
            });
        }
    }

    // Validate term_bg_luma value
    if let Some(luma) = theme.get("term_bg_luma").and_then(|v| v.as_str()) {
        if !["light", "dark"].contains(&luma) {
            return Err(BuildError::InvalidValue {
                field: "term_bg_luma".to_string(),
                value: luma.to_string(),
                path: path.to_owned(),
            });
        }
    }

    // Validate color_support value
    if let Some(support) = theme.get("min_color_support").and_then(|v| v.as_str()) {
        if !["basic", "color_256", "true_color"].contains(&support) {
            return Err(BuildError::InvalidValue {
                field: "min_color_support".to_string(),
                value: support.to_string(),
                path: path.to_owned(),
            });
        }
    }

    // Validate palette fields
    if let Some(palette) = theme.get("palette").and_then(|v| v.as_table()) {
        let required_styles = [
            "heading1", "heading2", "heading3", "error", "warning", "success", "info", "emphasis",
            "code", "normal", "subtle", "hint", "debug", "trace",
        ];

        for style in required_styles {
            if !palette.contains_key(style) {
                return Err(BuildError::MissingStyle {
                    style: style.to_string(),
                    path: path.to_owned(),
                });
            }
        }
    }

    Ok(())
}
