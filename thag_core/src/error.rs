use std::{
    borrow::Cow,
    sync::{MutexGuard, PoisonError as LockError},
};
use thiserror::Error;
use toml::de::Error as TomlDeError;

#[allow(clippy::module_name_repetitions)]
#[derive(Error, Debug)]
pub enum ThagError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    FromStr(Cow<'static, str>),

    #[error("Logic error: {0}")]
    Logic(&'static str),

    #[error("Lock mutex guard error: {0}")]
    LockMutexGuard(&'static str), // For lock errors with MutexGuard

    #[error("None value encountered: {0}")]
    NoneOption(&'static str),

    #[error("syn error: {0}")]
    Syn(#[from] syn::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(toml::de::Error), // For TOML deserialization errors

    #[error("Validation error: {0}")]
    Validation(String),
}

impl<'a, T> From<LockError<MutexGuard<'a, T>>> for ThagError {
    fn from(_err: LockError<MutexGuard<'a, T>>) -> Self {
        Self::LockMutexGuard("Lock poisoned")
    }
}

impl From<&'static str> for ThagError {
    fn from(s: &'static str) -> Self {
        Self::FromStr(Cow::Borrowed(s))
    }
}

impl From<String> for ThagError {
    fn from(s: String) -> Self {
        Self::FromStr(Cow::Owned(s))
    }
}

impl From<TomlDeError> for ThagError {
    fn from(err: TomlDeError) -> Self {
        Self::TomlDe(err)
    }
}

pub type ThagResult<T> = std::result::Result<T, ThagError>;