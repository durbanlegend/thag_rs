#[cfg(any(feature = "cargo_toml", feature = "toml"))]
use crate::shared::disentangle;
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
use std::borrow::Cow;
use std::string::FromUtf8Error;
use std::sync::{MutexGuard, PoisonError as LockError};
use std::{error::Error, io};
use strum::ParseError as StrumParseError;
#[cfg(feature = "syn")]
use syn::Error as SynError;
#[cfg(feature = "toml")]
use toml::de::Error as TomlDeError;
#[cfg(feature = "toml")]
use toml::ser::Error as TomlSerError;

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
    NoneOption(&'static str), // For unwrapping Options
    OsString(std::ffi::OsString), // For unconvertible OsStrings
    #[cfg(feature = "reedline")]
    Reedline(ReedlineError), // For reedline errors
    #[cfg(feature = "serde_merge")]
    SerdeMerge(SerdeMergeError), // For serde_merge errors
    StrumParse(StrumParseError), // For strum parse enum
    #[cfg(feature = "syn")]
    Syn(SynError), // For syn errors
    Theme(ThemeError), // For thag_rs::styling theme errors
    #[cfg(feature = "toml")]
    TomlDe(TomlDeError), // For TOML deserialization errors
    #[cfg(feature = "toml")]
    TomlSer(TomlSerError), // For TOML serialization errors
    #[cfg(feature = "cargo_toml")]
    Toml(CargoTomlError), // For cargo_toml errors
    UnsupportedTerm, // For terminal interrogation
    Validation(String), // For config.toml and similar validation
    VarError(std::env::VarError), // For std::env::var errors
}

impl From<FromUtf8Error> for ThagError {
    fn from(err: FromUtf8Error) -> Self {
        Self::FromUtf8(err)
    }
}

impl From<io::Error> for ThagError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

#[cfg(feature = "clap")]
impl From<ClapError> for ThagError {
    fn from(err: ClapError) -> Self {
        Self::ClapError(err)
    }
}

impl From<StrumParseError> for ThagError {
    fn from(err: StrumParseError) -> Self {
        Self::StrumParse(err)
    }
}

impl From<ThemeError> for ThagError {
    fn from(err: ThemeError) -> Self {
        ThagError::Theme(err)
    }
}

#[cfg(feature = "toml")]
impl From<TomlDeError> for ThagError {
    fn from(err: TomlDeError) -> Self {
        Self::TomlDe(err)
    }
}

#[cfg(feature = "toml")]
impl From<TomlSerError> for ThagError {
    fn from(err: TomlSerError) -> Self {
        Self::TomlSer(err)
    }
}

#[cfg(feature = "cargo_toml")]
impl From<CargoTomlError> for ThagError {
    fn from(err: CargoTomlError) -> Self {
        Self::Toml(err)
    }
}

impl From<String> for ThagError {
    fn from(s: String) -> Self {
        Self::FromStr(Cow::Owned(s))
    }
}

impl From<&'static str> for ThagError {
    fn from(s: &'static str) -> Self {
        Self::FromStr(Cow::Borrowed(s))
    }
}

#[cfg(feature = "reedline")]
impl From<ReedlineError> for ThagError {
    fn from(err: ReedlineError) -> Self {
        Self::Reedline(err)
    }
}

#[cfg(feature = "serde_merge")]
impl From<SerdeMergeError> for ThagError {
    fn from(err: SerdeMergeError) -> Self {
        Self::SerdeMerge(err)
    }
}

#[cfg(feature = "syn")]
impl From<SynError> for ThagError {
    fn from(err: SynError) -> Self {
        Self::Syn(err)
    }
}

impl<'a, T> From<LockError<MutexGuard<'a, T>>> for ThagError {
    fn from(_err: LockError<MutexGuard<'a, T>>) -> Self {
        Self::LockMutexGuard("Lock poisoned")
    }
}

#[cfg(feature = "bitflags")]
impl From<BitFlagsParseError> for ThagError {
    fn from(err: BitFlagsParseError) -> Self {
        Self::BitFlagsParse(err)
    }
}

impl From<Box<dyn Error + Send + Sync + 'static>> for ThagError {
    fn from(err: Box<dyn Error + Send + Sync + 'static>) -> Self {
        Self::Dyn(err)
    }
}

impl From<std::env::VarError> for ThagError {
    fn from(err: std::env::VarError) -> Self {
        Self::VarError(err)
    }
}

impl std::fmt::Display for ThagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Use display formatting instead of debug formatting where possible
            #[cfg(feature = "bitflags")]
            Self::BitFlagsParse(e) => write!(f, "{e}"),
            Self::Cancelled => write!(f, "Cancelled"),
            #[cfg(feature = "clap")]
            Self::ClapError(e) => write!(f, "{e}"),
            Self::Command(s) | Self::Logic(s) | Self::NoneOption(s) => {
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
            #[cfg(feature = "reedline")]
            Self::Reedline(e) => write!(f, "{e}"),
            #[cfg(feature = "serde_merge")]
            Self::SerdeMerge(e) => write!(f, "{e}"),
            Self::StrumParse(e) => write!(f, "{e}"),
            #[cfg(feature = "syn")]
            Self::Syn(e) => write!(f, "{e}"),
            Self::Theme(e) => write!(f, "{e}"),
            #[cfg(feature = "toml")]
            Self::TomlDe(e) => {
                // Extract the actual error message without all the nested structure
                let msg = e.to_string();
                write!(f, "toml::de::Error: {}", disentangle(msg.as_str()))?;
                Ok(())
            }
            #[cfg(feature = "toml")]
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
            Self::UnsupportedTerm => write!(f, "Unsupported terminal type"),
            Self::Validation(e) => write!(f, "{e}"),
            Self::VarError(e) => write!(f, "{e}"),
        }
    }
}

impl Error for ThagError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            // The cause is the underlying implementation error type. Is implicitly
            // cast to the trait object `&error::Error`. This works because the
            // underlying type already implements the `Error` trait.
            #[cfg(feature = "bitflags")]
            Self::BitFlagsParse(e) => Some(e),
            Self::Cancelled | Self::UnsupportedTerm => Some(self),
            #[cfg(feature = "clap")]
            Self::ClapError(ref e) => Some(e),
            Self::Command(_e) => Some(self),
            Self::Dyn(e) => Some(&**e),
            Self::FromStr(ref _e) => Some(self),
            Self::FromUtf8(e) => Some(e),
            Self::Io(ref e) => Some(e),
            Self::LockMutexGuard(_e) => Some(self),
            Self::Logic(_e) => Some(self),
            Self::NoneOption(_e) => Some(self),
            Self::OsString(ref _o) => Some(self),
            #[cfg(feature = "reedline")]
            Self::Reedline(e) => Some(e),
            #[cfg(feature = "serde_merge")]
            Self::SerdeMerge(ref e) => Some(e),
            Self::StrumParse(ref e) => Some(e),
            #[cfg(feature = "syn")]
            Self::Syn(e) => Some(e),
            Self::Theme(ref e) => Some(e),
            #[cfg(feature = "toml")]
            #[cfg(feature = "toml")]
            Self::TomlDe(ref e) => Some(e),
            #[cfg(feature = "toml")]
            Self::TomlSer(ref e) => Some(e),
            #[cfg(feature = "cargo_toml")]
            Self::Toml(ref e) => Some(e),
            Self::Validation(ref _e) => Some(self),
            Self::VarError(ref e) => Some(e),
        }
    }
}

#[derive(Debug)]
pub enum ThemeError {
    DarkThemeLightTerm,
    InsufficientColorSupport,
    LightThemeDarkTerm,
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeError::DarkThemeLightTerm => write!(
                f,
                "Only light themes may be selected for a light terminal background."
            ),
            ThemeError::InsufficientColorSupport => write!(
                f,
                "Configured or detected level of terminal colour support is insufficient for this theme."
            ),
            ThemeError::LightThemeDarkTerm => write!(
                f,
                "Only dark themes may be selected for a dark terminal background."
            ),
        }
    }
}

impl std::error::Error for ThemeError {}
