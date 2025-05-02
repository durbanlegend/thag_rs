use std::error::Error;

#[derive(Clone, Debug)]
pub enum ProfileError {
    General(String),
    Inquire(String),
    InvalidSection(String),
    Io(String),
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::General(e) => write!(f, "Profiling error: {e}"),
            Self::Inquire(e) => write!(f, "{e}"),
            Self::InvalidSection(e) => write!(f, "Invalid profile section: {e}"),
            Self::Io(e) => write!(f, "IO operation failed: {e}"),
        }
    }
}

impl std::error::Error for ProfileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::General(_e) => None,
            Self::Inquire(_e) => None,
            Self::InvalidSection(_e) => None,
            Self::Io(_e) => None,
        }
    }
}

impl From<std::io::Error> for ProfileError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

#[cfg(feature = "analyze-tool")]
impl From<inquire::InquireError> for ProfileError {
    fn from(err: inquire::InquireError) -> Self {
        Self::Inquire(err.to_string())
    }
}

pub type ProfileResult<T> = Result<T, ProfileError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_display() {
        // Test display implementation for all error variants
        assert_eq!(
            ProfileError::General("test error".to_string()).to_string(),
            "Profiling error: test error"
        );
        assert_eq!(
            ProfileError::Inquire("test inquiry".to_string()).to_string(),
            "test inquiry"
        );
        assert_eq!(
            ProfileError::InvalidSection("test section".to_string()).to_string(),
            "Invalid profile section: test section"
        );
        assert_eq!(
            ProfileError::Io("test io".to_string()).to_string(),
            "IO operation failed: test io"
        );
    }

    #[test]
    fn test_from_implementations() {
        // Test the General variant
        let string_error = "test error".to_string();
        let error = ProfileError::General(string_error);
        assert!(matches!(error, ProfileError::General(_)));

        // Test the InvalidSection variant
        let string_error = "test error".to_string();
        let error = ProfileError::InvalidSection(string_error);
        assert!(matches!(error, ProfileError::InvalidSection(_)));

        // Test the Inquire variant
        let string_error = "test inquiry".to_string();
        let error = ProfileError::Inquire(string_error);
        assert!(matches!(error, ProfileError::Inquire(_)));

        // Test the Io variant
        let string_error = "test io".to_string();
        let error = ProfileError::Io(string_error);
        assert!(matches!(error, ProfileError::Io(_)));
    }

    #[test]
    fn test_from_io_error() {
        // Test conversion from std::io::Error
        let io_error = IoError::new(ErrorKind::NotFound, "file not found");
        let profile_error = ProfileError::from(io_error);

        assert!(matches!(profile_error, ProfileError::Io(_)));
        assert!(profile_error.to_string().contains("file not found"));
    }

    #[test]
    fn test_result_type() {
        // Test the ProfileResult type alias
        let success_result: ProfileResult<i32> = Ok(42);
        let error_result: ProfileResult<i32> = Err(ProfileError::General("test error".to_string()));

        assert_eq!(success_result.unwrap(), 42);
        assert!(error_result.is_err());

        let error = error_result.unwrap_err();
        assert!(matches!(error, ProfileError::General(_)));
    }

    #[test]
    fn test_clone_and_debug() {
        // Test Clone implementation
        let error = ProfileError::General("test error".to_string());
        let cloned_error = error.clone();

        match cloned_error {
            ProfileError::General(msg) => assert_eq!(msg, "test error"),
            _ => panic!("Unexpected error variant after cloning"),
        }

        // Test Debug implementation
        let debug_string = format!("{:?}", error);
        assert!(debug_string.contains("General"));
        assert!(debug_string.contains("test error"));
    }

    #[test]
    fn test_error_trait_implementation() {
        // Test Error trait implementation
        let error = ProfileError::General("test error".to_string());
        let dyn_error: &dyn std::error::Error = &error;

        // Test source() method
        assert!(dyn_error.source().is_none());

        // Verify it can be used where Error is required
        fn takes_error(_err: &dyn std::error::Error) {}
        takes_error(&error);
    }

    #[cfg(feature = "analyze-tool")]
    #[test]
    fn test_from_inquire_error() {
        // This test depends on the analyze-tool feature being enabled
        use inquire::InquireError;

        let inquire_error = InquireError::NotTTY;
        let profile_error = ProfileError::from(inquire_error);

        assert!(matches!(profile_error, ProfileError::Inquire(_)));
        // dbg!(&profile_error);
        assert!(profile_error
            .to_string()
            .contains("The input device is not a TTY"));
    }
}
