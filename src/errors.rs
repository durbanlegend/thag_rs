use std::ffi::OsString;
use std::{error::Error, io};
use toml::de::Error as TomlDeError;
use toml::ser::Error as TomlSerError;

#[derive(Debug)]
pub enum BuildRunError {
    Cancelled,                     // For user electing to cancel
    ClapError(clap::error::Error), // For clap errors
    Command(String),               // For errors during Cargo build or program execution
    FromStr(String),               // For parsing CargoManifest from a string
    Io(io::Error),                 // For I/O errors
    NoneOption(String),            // For unwrapping Options
    OsString(OsString),            // For unconvertible OsStrings
    // Path(String),          // For Path and PathBuf issues
    StrumParse(strum::ParseError), // For strum parse enum
    TomlDe(TomlDeError),           // For TOML deserialization errors
    TomlSer(TomlSerError),         // For TOML serialization errors
}

impl BuildRunError {}

impl From<io::Error> for BuildRunError {
    fn from(err: io::Error) -> Self {
        BuildRunError::Io(err)
    }
}

impl From<clap::error::Error> for BuildRunError {
    fn from(err: clap::error::Error) -> Self {
        BuildRunError::ClapError(err)
    }
}

impl From<strum::ParseError> for BuildRunError {
    fn from(err: strum::ParseError) -> Self {
        BuildRunError::StrumParse(err)
    }
}

impl From<TomlDeError> for BuildRunError {
    fn from(err: TomlDeError) -> Self {
        BuildRunError::TomlDe(err)
    }
}

impl From<TomlSerError> for BuildRunError {
    fn from(err: TomlSerError) -> Self {
        BuildRunError::TomlSer(err)
    }
}

impl From<String> for BuildRunError {
    fn from(err_msg: String) -> Self {
        BuildRunError::FromStr(err_msg)
    }
}

impl std::fmt::Display for BuildRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildRunError::Cancelled => write!(f, "Cancelled"),
            BuildRunError::ClapError(e) => write!(f, "{e:?}"),
            BuildRunError::Command(s)
            | BuildRunError::FromStr(s)
            | BuildRunError::NoneOption(s) => {
                for line in s.lines() {
                    writeln!(f, "{line}")?;
                }
                Ok(())
            }
            BuildRunError::Io(e) => write!(f, "{e:?}"),
            BuildRunError::OsString(o) => {
                writeln!(f, "{o:#?}")?;
                Ok(())
            }
            BuildRunError::StrumParse(e) => write!(f, "{e:?}"),
            BuildRunError::TomlDe(e) => write!(f, "{e:?}"),
            BuildRunError::TomlSer(e) => write!(f, "{e:?}"),
        }
    }
}

impl Error for BuildRunError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            // The cause is the underlying implementation error type. Is implicitly
            // cast to the trait object `&error::Error`. This works because the
            // underlying type already implements the `Error` trait.
            BuildRunError::Command(ref _e)
            | BuildRunError::FromStr(ref _e)
            | BuildRunError::NoneOption(ref _e) => Some(self),
            BuildRunError::ClapError(ref e) => Some(e),
            BuildRunError::Io(ref e) => Some(e),
            BuildRunError::OsString(ref _o) => Some(self),
            BuildRunError::StrumParse(ref e) => Some(e),
            BuildRunError::TomlDe(ref e) => Some(e),
            BuildRunError::TomlSer(ref e) => Some(e),
            BuildRunError::Cancelled => Some(self),
        }
    }
}
