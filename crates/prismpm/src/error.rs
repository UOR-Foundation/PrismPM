//! Public error types and diagnostic mappings for PrismPM.

use std::fmt;

/// Public error type for PrismPM operations.
#[derive(Debug)]
pub struct PrismError {
    /// Diagnostic error code (PPxxxx).
    pub code: &'static str,
    /// Detailed error message.
    pub message: String,
}

impl PrismError {
    /// Create a new PrismError.
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for PrismError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for PrismError {}

