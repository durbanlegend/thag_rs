use bitflags::parser::ParseError as BitFlagsParseError;
use cargo_toml::Error as CargoTomlError;
use clap::error::Error as ClapError;
use reedline::ReedlineError;
use serde_merge::error::Error as SerdeMergeError;
use std::borrow::Cow;
use std::sync::{MutexGuard, PoisonError as LockError};
use std::{error::Error, io};
use strum::ParseError as StrumParseError;
use syn::Error as SynError;
use toml::de::Error as TomlDeError;
use toml::ser::Error as TomlSerError;

pub type ThagResult<T> = Result<T, ThagError>;

#[derive(Debug)]
pub enum ThagError {
    BitFlagsParse(BitFlagsParseError), // For bitflags parse error
    Cancelled,                         // For user electing to cancel
    ClapError(ClapError),              // For clap errors
    Command(&'static str),             // For errors during Cargo build or program execution
    Dyn(Box<dyn Error>), // For boxed dynamic errors from 3rd parties (firestorm in first instance)
    FromStr(Cow<'static, str>), // For simple errors from a string
    Io(std::io::Error),  // For I/O errors
    LockMutexGuard(&'static str), // For lock errors with MutexGuard
    Logic(&'static str), // For logic errors
    NoneOption(&'static str), // For unwrapping Options
    OsString(std::ffi::OsString), // For unconvertible OsStrings
    Reedline(ReedlineError), // For reedline errors
    SerdeMerge(SerdeMergeError), // For serde_merge errors
    StrumParse(StrumParseError), // For strum parse enum
    Syn(SynError),       // For syn errors
    TomlDe(TomlDeError), // For TOML deserialization errors
    TomlSer(TomlSerError), // For TOML serialization errors
    Toml(CargoTomlError), // For cargo_toml errors
    UnsupportedTerm,
}

impl ThagError {}

impl From<io::Error> for ThagError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

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

impl From<TomlDeError> for ThagError {
    fn from(err: TomlDeError) -> Self {
        Self::TomlDe(err)
    }
}

impl From<TomlSerError> for ThagError {
    fn from(err: TomlSerError) -> Self {
        Self::TomlSer(err)
    }
}

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

impl From<ReedlineError> for ThagError {
    fn from(err: ReedlineError) -> Self {
        Self::Reedline(err)
    }
}

impl From<SerdeMergeError> for ThagError {
    fn from(err: SerdeMergeError) -> Self {
        Self::SerdeMerge(err)
    }
}

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

impl From<BitFlagsParseError> for ThagError {
    fn from(err: BitFlagsParseError) -> Self {
        Self::BitFlagsParse(err)
    }
}

impl From<Box<dyn Error>> for ThagError {
    fn from(err: Box<dyn Error>) -> Self {
        Self::Dyn(err)
    }
}

impl std::fmt::Display for ThagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BitFlagsParse(e) => write!(f, "{e:?}"),
            Self::Cancelled => write!(f, "Cancelled"),
            Self::ClapError(e) => write!(f, "{e:?}"),
            Self::Command(s) | Self::Logic(s) | Self::NoneOption(s) => {
                for line in s.lines() {
                    writeln!(f, "{line}")?;
                }
                Ok(())
            }
            Self::Dyn(e) => write!(f, "{e:?}"),
            Self::FromStr(s) => {
                for line in s.lines() {
                    writeln!(f, "{line}")?;
                }
                Ok(())
            }
            Self::Io(e) => write!(f, "{e:?}"),
            Self::LockMutexGuard(e) => write!(f, "{e:?}"),
            Self::OsString(o) => {
                writeln!(f, "{o:#?}")?;
                Ok(())
            }
            Self::Reedline(e) => write!(f, "{e:?}"),
            Self::SerdeMerge(e) => write!(f, "{e:?}"),
            Self::StrumParse(e) => write!(f, "{e:?}"),
            Self::Syn(e) => write!(f, "{e:?}"),
            Self::TomlDe(e) => write!(f, "{e:?}"),
            Self::TomlSer(e) => write!(f, "{e:?}"),
            Self::Toml(e) => write!(f, "{e:?}"),
            Self::UnsupportedTerm => write!(f, "Unsupported terminal type"),
        }
    }
}

impl Error for ThagError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            // The cause is the underlying implementation error type. Is implicitly
            // cast to the trait object `&error::Error`. This works because the
            // underlying type already implements the `Error` trait.
            Self::BitFlagsParse(e) => Some(e),
            Self::Cancelled | Self::UnsupportedTerm => Some(self),
            Self::ClapError(ref e) => Some(e),
            Self::Command(_e) => Some(self),
            Self::Dyn(e) => Some(&**e),
            Self::FromStr(ref _e) => Some(self),
            Self::Io(ref e) => Some(e),
            Self::LockMutexGuard(_e) => Some(self),
            Self::Logic(_e) => Some(self),
            Self::NoneOption(_e) => Some(self),
            Self::OsString(ref _o) => Some(self),
            Self::Reedline(e) => Some(e),
            Self::SerdeMerge(ref e) => Some(e),
            Self::StrumParse(ref e) => Some(e),
            Self::Syn(e) => Some(e),
            Self::TomlDe(ref e) => Some(e),
            Self::TomlSer(ref e) => Some(e),
            Self::Toml(ref e) => Some(e),
        }
    }
}
