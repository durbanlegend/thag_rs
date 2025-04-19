use crate::shared::disentangle;
use crate::styling::TermBgLuma;
use crate::ColorSupport;
use std::borrow::Cow;
use std::num::ParseIntError;
use std::string::FromUtf8Error;
use std::sync::{MutexGuard, PoisonError as LockError};
use std::{error::Error, io};
use strum::ParseError as StrumParseError;
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

pub type ThagResult<T> = Result<T, ThagError>;

#[derive(Debug)]
pub enum ThagError {
    #[cfg(feature = "bitflags")]
    BitFlagsParse(BitFlagsParseError), // For bitflags parse error
    Cancelled, // For user electing to cancel
    #[cfg(feature = "clap")]
    ClapError(ClapError), // For clap errors
    Command(&'static str), // For errors during Cargo build or program execution
    Dyn(Box<dyn Error + Send + Sync + 'static>), // For boxed dynamic errors from 3rd parties
    FromStr(Cow<'static, str>), // For simple errors from a string
    FromUtf8(FromUtf8Error), // For simple errors from a utf8 array
    Io(std::io::Error), // For I/O errors
    LockMutexGuard(&'static str), // For lock errors with MutexGuard
    Logic(&'static str), // For logic errors
    NoneOption(String), // For unwrapping Options
    OsString(std::ffi::OsString), // For unconvertible OsStrings
    Parse,
    ParseInt(ParseIntError),
    Profiling(String),
    #[cfg(feature = "reedline")]
    Reedline(ReedlineError), // For reedline errors
    #[cfg(feature = "serde_merge")]
    SerdeMerge(SerdeMergeError), // For serde_merge errors
    StrumParse(StrumParseError), // For strum parse enum
    #[cfg(feature = "syn")]
    Syn(SynError), // For syn errors
    #[cfg(feature = "color_detect")]
    Termbg(termbg::Error), // For termbg errors
    Theme(ThemeError),           // For thag_rs::styling theme errors
    TomlDe(TomlDeError),         // For TOML deserialization errors
    TomlSer(TomlSerError),       // For TOML serialization errors
    #[cfg(feature = "cargo_toml")]
    Toml(CargoTomlError), // For cargo_toml errors
    UnsupportedTerm(String),     // For terminal interrogation
    Validation(String),          // For config.toml and similar validation
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
        }
    }
}

impl Error for ThagError {
    #[profiled]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            // The cause is the underlying implementation error type. Is implicitly
            // cast to the trait object `&error::Error`. This works because the
            // underlying type already implements the `Error` trait.
            #[cfg(feature = "bitflags")]
            Self::BitFlagsParse(e) => Some(e),
            Self::Cancelled => None,
            #[cfg(feature = "clap")]
            Self::ClapError(e) => Some(e),
            Self::Command(_) => None,
            Self::Dyn(e) => Some(&**e),
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
        }
    }
}

#[derive(Debug)]
pub enum ThemeError {
    BackgroundDetectionFailed,
    ColorSupportMismatch {
        required: ColorSupport,
        available: ColorSupport,
    },
    DarkThemeLightTerm,
    InsufficientColorSupport,
    InvalidAnsiCode(String),
    InvalidColorSupport(String),
    InvalidColorValue(String),
    InvalidStyle(String),
    InvalidTermBgLuma(String),
    LightThemeDarkTerm,
    NoValidBackground(String),
    TermBgLumaMismatch {
        theme: TermBgLuma,
        terminal: TermBgLuma,
    },
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
