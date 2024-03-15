use std::io;
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
