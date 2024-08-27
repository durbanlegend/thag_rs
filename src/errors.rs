use std::ffi::OsString;
use std::{error::Error, io};
use toml::de::Error as TomlDeError;
use toml::ser::Error as TomlSerError;

#[derive(Debug)]
pub enum ThagError {
    Cancelled,                     // For user electing to cancel
    ClapError(clap::error::Error), // For clap errors
    Command(String),               // For errors during Cargo build or program execution
    FromStr(String),               // For parsing CargoManifest from a string
    Io(io::Error),                 // For I/O errors
    NoneOption(String),            // For unwrapping Options
    OsString(OsString),            // For unconvertible OsStrings
    // Path(String),               // For Path and PathBuf issues
    StrumParse(strum::ParseError), // For strum parse enum
    TomlDe(TomlDeError),           // For TOML deserialization errors
    TomlSer(TomlSerError),         // For TOML serialization errors
    Toml(cargo_toml::Error),       // For cargo_toml errors
}

impl ThagError {}

impl From<io::Error> for ThagError {
    fn from(err: io::Error) -> Self {
        ThagError::Io(err)
    }
}

impl From<clap::error::Error> for ThagError {
    fn from(err: clap::error::Error) -> Self {
        ThagError::ClapError(err)
    }
}

impl From<strum::ParseError> for ThagError {
    fn from(err: strum::ParseError) -> Self {
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

impl From<cargo_toml::Error> for ThagError {
    fn from(err: cargo_toml::Error) -> Self {
        ThagError::Toml(err)
    }
}

impl From<String> for ThagError {
    fn from(err_msg: String) -> Self {
        ThagError::FromStr(err_msg)
    }
}

impl std::fmt::Display for ThagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThagError::Cancelled => write!(f, "Cancelled"),
            ThagError::ClapError(e) => write!(f, "{e:?}"),
            ThagError::Command(s) | ThagError::FromStr(s) | ThagError::NoneOption(s) => {
                for line in s.lines() {
                    writeln!(f, "{line}")?;
                }
                Ok(())
            }
            ThagError::Io(e) => write!(f, "{e:?}"),
            ThagError::OsString(o) => {
                writeln!(f, "{o:#?}")?;
                Ok(())
            }
            ThagError::StrumParse(e) => write!(f, "{e:?}"),
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
            ThagError::Command(ref _e)
            | ThagError::FromStr(ref _e)
            | ThagError::NoneOption(ref _e) => Some(self),
            ThagError::ClapError(ref e) => Some(e),
            ThagError::Io(ref e) => Some(e),
            ThagError::OsString(ref _o) => Some(self),
            ThagError::StrumParse(ref e) => Some(e),
            ThagError::TomlDe(ref e) => Some(e),
            ThagError::TomlSer(ref e) => Some(e),
            ThagError::Toml(ref e) => Some(e),
            ThagError::Cancelled => Some(self),
        }
    }
}
