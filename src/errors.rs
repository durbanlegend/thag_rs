use std::borrow::Cow;
use std::num::ParseIntError;
use std::string::FromUtf8Error;
use std::sync::{MutexGuard, PoisonError as LockError};
use std::{error::Error, io};
use strum::ParseError as StrumParseError;
use thag_common::disentangle;
use thag_common::{ColorSupport, TermBgLuma};
use thag_profiler::profiled;
use toml::de::Error as TomlDeError;
use toml::ser::Error as TomlSerError;

#[cfg(feature = "bitflags")]
use bitflags::parser::ParseError as BitFlagsParseError;

#[cfg(feature = "cargo_toml")]
use cargo_toml::Error as CargoTomlError;

#[cfg(feature = "clap")]
use clap::error::Error as ClapError;

#[cfg(feature = "reedline")]
use reedline::ReedlineError;

#[cfg(feature = "serde_merge")]
use serde_merge::error::Error as SerdeMergeError;

#[cfg(feature = "syn")]
use syn::Error as SynError;

/// Result type alias for Thag operations that may fail with a `ThagError`.
pub type ThagResult<T> = Result<T, ThagError>;

#[derive(Debug)]
/// Error type for all Thag operations.
///
/// This enum encompasses all possible error conditions that can occur
/// during Thag's execution, including I/O errors, parsing errors,
/// configuration errors, and errors from external dependencies.
pub enum ThagError {
    /// Error from `thag_common` operations
    Common(thag_common::ThagCommonError),
    /// Error from `thag_common::config`
    Config(thag_common::ConfigError),
    /// Error from `thag_styling` operations
    #[cfg(feature = "thag_styling")]
    Styling(thag_styling::StylingError),
    #[cfg(feature = "bitflags")]
    /// Error parsing bitflags values
    BitFlagsParse(BitFlagsParseError), // For bitflags parse error
    /// User cancelled the operation
    Cancelled, // For user electing to cancel
    #[cfg(feature = "clap")]
    /// Error from `clap` command-line parsing
    ClapError(ClapError), // For clap errors
    /// Error during Cargo build or program execution
    Command(&'static str), // For errors during Cargo build or program execution
    /// Boxed dynamic error from third-party libraries
    Dyn(Box<dyn Error + Send + Sync + 'static>), // For boxed dynamic errors from 3rd parties
    /// Simple error from a string message
    FromStr(Cow<'static, str>), // For simple errors from a string
    /// Error converting from UTF-8 bytes
    FromUtf8(FromUtf8Error), // For simple errors from a utf8 array
    /// I/O operation error
    Io(std::io::Error), // For I/O errors
    /// Mutex guard lock error
    LockMutexGuard(&'static str), // For lock errors with MutexGuard
    /// Logic error in program flow
    Logic(&'static str), // For logic errors
    /// Error unwrapping None from Option
    NoneOption(String), // For unwrapping Options
    /// Error converting `OsString` to valid UTF-8
    OsString(std::ffi::OsString), // For unconvertible OsStrings
    /// Generic parsing error
    Parse,
    /// Integer parsing error
    ParseInt(ParseIntError),
    /// Profiling system error
    Profiling(String),
    #[cfg(feature = "reedline")]
    /// Reedline terminal interaction error
    Reedline(ReedlineError), // For reedline errors
    #[cfg(feature = "serde_merge")]
    /// Serde merge operation error
    SerdeMerge(SerdeMergeError), // For serde_merge errors
    /// Strum enum parsing error
    StrumParse(StrumParseError), // For strum parse enum
    #[cfg(feature = "syn")]
    /// Syn syntax parsing error
    Syn(SynError), // For syn errors
    #[cfg(feature = "color_detect")]
    /// Terminal background detection error
    Termbg(termbg::Error), // For termbg errors
    /// Theme-related error
    Theme(ThemeError), // For thag_rs::styling theme errors
    /// TOML deserialization error
    TomlDe(TomlDeError), // For TOML deserialization errors
    /// TOML serialization error
    TomlSer(TomlSerError), // For TOML serialization errors
    #[cfg(feature = "cargo_toml")]
    /// Cargo.toml parsing error
    Toml(CargoTomlError), // For cargo_toml errors
    /// Unsupported terminal type error
    UnsupportedTerm(String), // For terminal interrogation
    /// Configuration validation error
    Validation(String), // For config.toml and similar validation
    /// Environment variable error
    VarError(std::env::VarError), // For std::env::var errors
}

impl From<FromUtf8Error> for ThagError {
    #[profiled]
    fn from(err: FromUtf8Error) -> Self {
        Self::FromUtf8(err)
    }
}

impl From<io::Error> for ThagError {
    #[profiled]
    fn from(err: io::Error) -> Self {
        // Enable backtraces in all build types - this is a development tool after all
        use std::backtrace::Backtrace;

        // Get more context about the IO error
        eprintln!("IO Error: {err}");
        eprintln!("Kind: {:?}", err.kind());
        eprintln!("Raw OS Error: {:?}", err.raw_os_error());

        // Always collect and print a backtrace for IO errors
        // This ensures we get diagnostics in both debug and release builds
        let backtrace = Backtrace::force_capture(); // Always capture, even in release mode
        eprintln!("Location:\n{backtrace}");

        Self::Io(err)
    }
}

#[cfg(feature = "clap")]
impl From<ClapError> for ThagError {
    #[profiled]
    fn from(err: ClapError) -> Self {
        Self::ClapError(err)
    }
}

impl From<StrumParseError> for ThagError {
    #[profiled]
    fn from(err: StrumParseError) -> Self {
        Self::StrumParse(err)
    }
}

impl From<ThemeError> for ThagError {
    #[profiled]
    fn from(err: ThemeError) -> Self {
        Self::Theme(err)
    }
}

impl From<TomlDeError> for ThagError {
    #[profiled]
    fn from(err: TomlDeError) -> Self {
        Self::TomlDe(err)
    }
}

impl From<TomlSerError> for ThagError {
    #[profiled]
    fn from(err: TomlSerError) -> Self {
        Self::TomlSer(err)
    }
}

#[cfg(feature = "cargo_toml")]
impl From<CargoTomlError> for ThagError {
    #[profiled]
    fn from(err: CargoTomlError) -> Self {
        Self::Toml(err)
    }
}

impl From<String> for ThagError {
    #[profiled]
    fn from(s: String) -> Self {
        Self::FromStr(Cow::Owned(s))
    }
}

impl From<&'static str> for ThagError {
    #[profiled]
    fn from(s: &'static str) -> Self {
        Self::FromStr(Cow::Borrowed(s))
    }
}

impl From<ParseIntError> for ThagError {
    #[profiled]
    fn from(err: ParseIntError) -> Self {
        Self::ParseInt(err)
    }
}

impl From<thag_profiler::ProfileError> for ThagError {
    fn from(err: thag_profiler::ProfileError) -> Self {
        Self::Profiling(err.to_string())
    }
}

#[cfg(feature = "reedline")]
impl From<ReedlineError> for ThagError {
    #[profiled]
    fn from(err: ReedlineError) -> Self {
        Self::Reedline(err)
    }
}

#[cfg(feature = "serde_merge")]
impl From<SerdeMergeError> for ThagError {
    #[profiled]
    fn from(err: SerdeMergeError) -> Self {
        Self::SerdeMerge(err)
    }
}

#[cfg(feature = "syn")]
impl From<SynError> for ThagError {
    #[profiled]
    fn from(err: SynError) -> Self {
        Self::Syn(err)
    }
}

impl<'a, T> From<LockError<MutexGuard<'a, T>>> for ThagError {
    #[profiled]
    fn from(_err: LockError<MutexGuard<'a, T>>) -> Self {
        Self::LockMutexGuard("Lock poisoned")
    }
}

#[cfg(feature = "bitflags")]
impl From<BitFlagsParseError> for ThagError {
    #[profiled]
    fn from(err: BitFlagsParseError) -> Self {
        Self::BitFlagsParse(err)
    }
}

impl From<Box<dyn Error + Send + Sync + 'static>> for ThagError {
    #[profiled]
    fn from(err: Box<dyn Error + Send + Sync + 'static>) -> Self {
        Self::Dyn(err)
    }
}

#[cfg(feature = "color_detect")]
impl From<termbg::Error> for ThagError {
    #[profiled]
    fn from(err: termbg::Error) -> Self {
        Self::Termbg(err)
    }
}

impl From<std::env::VarError> for ThagError {
    #[profiled]
    fn from(err: std::env::VarError) -> Self {
        Self::VarError(err)
    }
}

impl std::fmt::Display for ThagError {
    #[profiled]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Use display formatting instead of debug formatting where possible
            #[cfg(feature = "bitflags")]
            Self::BitFlagsParse(e) => write!(f, "{e}"),
            Self::Cancelled => write!(f, "Cancelled"),
            #[cfg(feature = "clap")]
            Self::ClapError(e) => write!(f, "{e}"),
            Self::Command(s) | Self::Logic(s) => {
                for line in s.lines() {
                    writeln!(f, "{line}")?;
                }
                Ok(())
            }
            Self::NoneOption(s) => {
                for line in s.lines() {
                    writeln!(f, "{line}")?;
                }
                Ok(())
            }
            Self::Dyn(e) => write!(f, "{e}"),
            Self::FromStr(s) => {
                for line in s.lines() {
                    writeln!(f, "{line}")?;
                }
                Ok(())
            }
            Self::FromUtf8(e) => write!(f, "{e}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::LockMutexGuard(e) => write!(f, "{e}"),
            Self::OsString(o) => writeln!(f, "<invalid UTF-8: {o:?}>"),
            Self::Parse => write!(f, "Error parsing source data"),
            Self::ParseInt(e) => write!(f, "{e}"),
            Self::Profiling(e) => write!(f, "{e}"),
            #[cfg(feature = "reedline")]
            Self::Reedline(e) => write!(f, "{e}"),
            #[cfg(feature = "serde_merge")]
            Self::SerdeMerge(e) => write!(f, "{e}"),
            Self::StrumParse(e) => write!(f, "{e}"),
            #[cfg(feature = "syn")]
            Self::Syn(e) => write!(f, "{e}"),
            #[cfg(feature = "color_detect")]
            Self::Termbg(e) => write!(f, "{e}"),
            Self::Theme(e) => write!(f, "{e}"),

            Self::TomlDe(e) => {
                // Extract the actual error message without all the nested structure
                let msg = e.to_string();
                write!(f, "toml::de::Error: {}", disentangle(msg.as_str()))?;
                Ok(())
            }

            Self::TomlSer(e) => {
                // Extract the actual error message without all the nested structure
                let msg = e.to_string();
                write!(f, "toml::ser::Error: {}", disentangle(msg.as_str()))?;
                Ok(())
            }
            // Self::Toml(e) => write!(f, "{e}"),
            #[cfg(feature = "cargo_toml")]
            Self::Toml(e) => {
                // Extract the actual error message without all the nested structure
                let msg = e.to_string();
                write!(f, "cargo_toml error: {}", disentangle(msg.as_str()))?;
                Ok(())
            }
            Self::UnsupportedTerm(e) => write!(f, "Unsupported terminal type {e}"),
            Self::Validation(e) => write!(f, "{e}"),
            Self::VarError(e) => write!(f, "{e}"),
            Self::Common(e) => write!(f, "Common error: {e}"),
            Self::Config(e) => write!(f, "Config error: {e}"),
            #[cfg(feature = "thag_styling")]
            Self::Styling(e) => write!(f, "Styling error: {e}"),
        }
    }
}

impl Error for ThagError {
    #[profiled]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // Force all match arms to return the same type by providing an explicit type annotation
        let result: Option<&(dyn Error + 'static)> = match self {
            #[cfg(feature = "bitflags")]
            Self::BitFlagsParse(e) => Some(e),
            Self::Cancelled => None,
            #[cfg(feature = "clap")]
            Self::ClapError(e) => Some(e),
            Self::Command(_) => None,
            // Use as_ref() to convert from Box<dyn Error + Send + Sync> to &dyn Error
            Self::Dyn(e) => Some(e.as_ref()),
            Self::FromStr(_) => None,
            Self::FromUtf8(e) => Some(e),
            Self::Io(e) => Some(e),
            Self::LockMutexGuard(_) => None,
            Self::Logic(_) => None,
            Self::NoneOption(_) => None,
            Self::OsString(_) => None,
            Self::Parse => None,
            Self::ParseInt(e) => Some(e),
            Self::Profiling(_) => None,
            #[cfg(feature = "reedline")]
            Self::Reedline(e) => Some(e),
            #[cfg(feature = "serde_merge")]
            Self::SerdeMerge(e) => Some(e),
            Self::StrumParse(e) => Some(e),
            #[cfg(feature = "syn")]
            Self::Syn(e) => Some(e),
            #[cfg(feature = "color_detect")]
            Self::Termbg(e) => Some(e),
            Self::Theme(e) => Some(e),
            Self::TomlDe(e) => Some(e),
            Self::TomlSer(e) => Some(e),
            #[cfg(feature = "cargo_toml")]
            Self::Toml(e) => Some(e),
            Self::UnsupportedTerm(_) => None,
            Self::Validation(_) => None,
            Self::VarError(e) => Some(e),
            Self::Common(e) => Some(e),
            Self::Config(e) => Some(e),
            #[cfg(feature = "thag_styling")]
            Self::Styling(e) => Some(e),
        };
        result
    }
}

#[derive(Debug)]
/// Error type for theme-related operations.
///
/// This enum encompasses all possible error conditions that can occur
/// during theme loading, validation, and application, including color
/// support mismatches, invalid configurations, and terminal compatibility issues.
pub enum ThemeError {
    /// Failed to detect terminal background color
    BackgroundDetectionFailed,
    /// Color support mismatch between theme requirements and terminal capabilities
    ColorSupportMismatch {
        /// The color support level required by the theme
        required: ColorSupport,
        /// The color support level available in the terminal
        available: ColorSupport,
    },
    /// Attempted to use a dark theme with a light terminal background
    DarkThemeLightTerm,
    /// Terminal does not support sufficient colors for the requested operation
    InsufficientColorSupport,
    /// Invalid ANSI escape code format
    InvalidAnsiCode(String),
    /// Invalid color support specification
    InvalidColorSupport(String),
    /// Invalid color value format or specification
    InvalidColorValue(String),
    /// Invalid style attribute specification
    InvalidStyle(String),
    /// Invalid terminal background luminance value
    InvalidTermBgLuma(String),
    /// Attempted to use a light theme with a dark terminal background
    LightThemeDarkTerm,
    /// No valid background color found for the specified theme
    NoValidBackground(String),
    /// Terminal background luminance mismatch with theme requirements
    TermBgLumaMismatch {
        /// The background luminance required by the theme
        theme: TermBgLuma,
        /// The actual background luminance of the terminal
        terminal: TermBgLuma,
    },
    /// Unknown or unrecognized theme name
    UnknownTheme(String),
}

impl std::fmt::Display for ThemeError {
    #[profiled]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BackgroundDetectionFailed => {
                write!(f, "Background RGB not detected or configured for terminal")
            }
            Self::ColorSupportMismatch { required, available } => {
                write!(f, "Theme requires {required:?} colors but terminal only supports {available:?}")
            }
            Self::DarkThemeLightTerm => write!(
                f,
                "Only light themes may be selected for a light terminal background."
            ),
            Self::InsufficientColorSupport => write!(
                f,
                "Configured or detected level of terminal colour support is insufficient for this theme."
            ),
            Self::InvalidAnsiCode(e) => write!(f, "{e}"),
            Self::InvalidColorSupport(msg) => write!(f, "Invalid color support: {msg}"),
            Self::InvalidColorValue(msg) => write!(f, "Invalid color value: {msg}"),
            Self::InvalidStyle(style) => write!(f, "Invalid style attribute: {style}"),
            Self::InvalidTermBgLuma(name) => write!(f, "Unknown value: must be `light` or `dark`: {name}"),
            Self::LightThemeDarkTerm => write!(
                f,
                "Only dark themes may be selected for a dark terminal background."
            ),
            Self::NoValidBackground(theme) => write!(f, "No valid background found for theme {theme}"),
            Self::TermBgLumaMismatch { theme, terminal } => {
                write!(f, "Theme requires {theme:?} background but terminal is {terminal:?}")
            }
            Self::UnknownTheme(name) => write!(f, "Unknown theme: {name}"),
        }
    }
}

impl std::error::Error for ThemeError {}

// From trait implementations for new error types
impl From<thag_common::ThagCommonError> for ThagError {
    fn from(err: thag_common::ThagCommonError) -> Self {
        Self::Common(err)
    }
}

impl From<thag_common::ConfigError> for ThagError {
    fn from(err: thag_common::ConfigError) -> Self {
        Self::Config(err)
    }
}

#[cfg(feature = "thag_styling")]
impl From<thag_styling::StylingError> for ThagError {
    fn from(err: thag_styling::StylingError) -> Self {
        Self::Styling(err)
    }
}
