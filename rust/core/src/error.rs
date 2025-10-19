use std::fmt;

/// Canonical error type for core crate.
#[derive(Debug, Clone)]
pub struct KelivoError {
    message: String,
}

impl KelivoError {
    pub fn new<M: Into<String>>(message: M) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for KelivoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for KelivoError {}

impl From<String> for KelivoError {
    fn from(message: String) -> Self {
        KelivoError { message }
    }
}

impl From<&str> for KelivoError {
    fn from(message: &str) -> Self {
        KelivoError {
            message: message.to_string(),
        }
    }
}

impl From<anyhow::Error> for KelivoError {
    fn from(error: anyhow::Error) -> Self {
        KelivoError {
            message: error.to_string(),
        }
    }
}

pub type KelivoResult<T> = Result<T, KelivoError>;
