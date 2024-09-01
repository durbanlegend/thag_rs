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
    NoneOption(&'static str), // For unwrapping Options
    OsString(std::ffi::OsString), // For unconvertible OsStrings
    Reedline(ReedlineError), // For reedline errors
    SerdeMerge(SerdeMergeError), // For serde_merge errors
    StrumParse(StrumParseError), // For strum parse enum
    Syn(SynError),       // For syn errors
    TomlDe(TomlDeError), // For TOML deserialization errors
    TomlSer(TomlSerError), // For TOML serialization errors
    Toml(CargoTomlError), // For cargo_toml errors
}

impl ThagError {}

impl From<io::Error> for ThagError {
    fn from(err: io::Error) -> Self {
        ThagError::Io(err)
    }
}

impl From<ClapError> for ThagError {
    fn from(err: ClapError) -> Self {
        ThagError::ClapError(err)
    }
}

impl From<StrumParseError> for ThagError {
    fn from(err: StrumParseError) -> Self {
        ThagError::StrumParse(err)
    }
}

impl From<TomlDeError> for ThagError {
    fn from(err: TomlDeError) -> Self {
        ThagError::TomlDe(err)
    }
}

impl From<TomlSerError> for ThagError {
    fn from(err: TomlSerError) -> Self {
        ThagError::TomlSer(err)
    }
}

impl From<CargoTomlError> for ThagError {
    fn from(err: CargoTomlError) -> Self {
        ThagError::Toml(err)
    }
}

impl From<String> for ThagError {
    fn from(s: String) -> Self {
        ThagError::FromStr(Cow::Owned(s))
    }
}

impl From<&'static str> for ThagError {
    fn from(s: &'static str) -> Self {
        ThagError::FromStr(Cow::Borrowed(s))
    }
}

impl From<ReedlineError> for ThagError {
    fn from(err: ReedlineError) -> Self {
        ThagError::Reedline(err)
    }
}

impl From<SerdeMergeError> for ThagError {
    fn from(err: SerdeMergeError) -> Self {
        ThagError::SerdeMerge(err)
    }
}

impl From<SynError> for ThagError {
    fn from(err: SynError) -> Self {
        ThagError::Syn(err)
    }
}

impl<'a, T> From<LockError<MutexGuard<'a, T>>> for ThagError {
    fn from(_err: LockError<MutexGuard<'a, T>>) -> Self {
        ThagError::LockMutexGuard("Lock poisoned")
    }
}

impl From<BitFlagsParseError> for ThagError {
    fn from(err: BitFlagsParseError) -> Self {
        ThagError::BitFlagsParse(err)
    }
}

impl From<Box<dyn Error>> for ThagError {
    fn from(err: Box<dyn Error>) -> Self {
        ThagError::Dyn(err)
    }
}

impl std::fmt::Display for ThagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThagError::BitFlagsParse(e) => write!(f, "{e:?}"),
            ThagError::Cancelled => write!(f, "Cancelled"),
            ThagError::ClapError(e) => write!(f, "{e:?}"),
            ThagError::Command(s) | ThagError::NoneOption(s) => {
                for line in s.lines() {
                    writeln!(f, "{line}")?;
                }
                Ok(())
            }
            ThagError::Dyn(e) => write!(f, "{e:?}"),
            ThagError::FromStr(s) => {
                for line in s.lines() {
                    writeln!(f, "{line}")?;
                }
                Ok(())
            }
            ThagError::Io(e) => write!(f, "{e:?}"),
            ThagError::LockMutexGuard(e) => write!(f, "{e:?}"),
            ThagError::OsString(o) => {
                writeln!(f, "{o:#?}")?;
                Ok(())
            }
            ThagError::Reedline(e) => write!(f, "{e:?}"),
            ThagError::SerdeMerge(e) => write!(f, "{e:?}"),
            ThagError::StrumParse(e) => write!(f, "{e:?}"),
            ThagError::Syn(e) => write!(f, "{e:?}"),
            ThagError::TomlDe(e) => write!(f, "{e:?}"),
            ThagError::TomlSer(e) => write!(f, "{e:?}"),
            ThagError::Toml(e) => write!(f, "{e:?}"),
        }
    }
}

impl Error for ThagError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            // The cause is the underlying implementation error type. Is implicitly
            // cast to the trait object `&error::Error`. This works because the
            // underlying type already implements the `Error` trait.
            ThagError::BitFlagsParse(e) => Some(e),
            ThagError::Cancelled => Some(self),
            ThagError::ClapError(ref e) => Some(e),
            ThagError::Command(_e) => Some(self),
            ThagError::Dyn(e) => Some(&**e),
            ThagError::FromStr(ref _e) => Some(self),
            ThagError::Io(ref e) => Some(e),
            ThagError::LockMutexGuard(_e) => Some(self),
            ThagError::NoneOption(_e) => Some(self),
            ThagError::OsString(ref _o) => Some(self),
            ThagError::Reedline(e) => Some(e),
            ThagError::SerdeMerge(ref e) => Some(e),
            ThagError::StrumParse(ref e) => Some(e),
            ThagError::Syn(e) => Some(e),
            ThagError::TomlDe(ref e) => Some(e),
            ThagError::TomlSer(ref e) => Some(e),
            ThagError::Toml(ref e) => Some(e),
        }
    }
}
