use std::{error::Error, io};
// use std::path::PathBuf::std;

#[derive(Debug)]
pub(crate) enum BuildRunError {
    Io(io::Error),   // For I/O errors
    Command(String), // For errors during Cargo build or program execution
}

impl From<io::Error> for BuildRunError {
    fn from(err: io::Error) -> Self {
        BuildRunError::Io(err)
    }
}

impl From<String> for BuildRunError {
    fn from(err_msg: String) -> Self {
        BuildRunError::Command(err_msg)
    }
}

impl std::fmt::Display for BuildRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildRunError::Io(e) => write!(f, "{e:?}"),
            BuildRunError::Command(string) => {
                for line in string.lines() {
                    writeln!(f, "{line}")?;
                }
                Ok(())
            }
        }
    }
}

impl Error for BuildRunError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            // The cause is the underlying implementation error type. Is implicitly
            // cast to the trait object `&error::Error`. This works because the
            // underlying type already implements the `Error` trait.
            BuildRunError::Io(ref e) => Some(e),
            BuildRunError::Command(ref _e) => Some(self),
        }
    }
}
