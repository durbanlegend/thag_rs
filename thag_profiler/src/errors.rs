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

    #[test]
    fn test_display() {
        assert_eq!(
            ProfileError::General("test error".to_string()).to_string(),
            "Profiling error: test error"
        );
        assert_eq!(
            ProfileError::InvalidSection("test error".to_string()).to_string(),
            "Invalid profile section: test error"
        );
    }

    #[test]
    fn test_from_implementations() {
        let string_error = "test error".to_string();
        let error = ProfileError::General(string_error);
        assert!(matches!(error, ProfileError::General(_)));

        let string_error = "test error".to_string();
        let error = ProfileError::InvalidSection(string_error);
        assert!(matches!(error, ProfileError::InvalidSection(_)));
    }
}
