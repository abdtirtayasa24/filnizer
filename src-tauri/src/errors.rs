use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Validation failed: {message}")]
    Validation { message: String },

    #[error("Database error: {0}")]
    Database(String),

    #[error("Filesystem error: {0}")]
    Filesystem(String),

    #[error("External tool error: {0}")]
    ExternalTool(String),

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation { .. } => "VALIDATION_ERROR",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Filesystem(_) => "FILESYSTEM_ERROR",
            Self::ExternalTool(_) => "EXTERNAL_TOOL_ERROR",
            Self::Unexpected(_) => "UNEXPECTED_ERROR",
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Filesystem(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::AppError;

    #[test]
    fn app_error_serializes_safe_payload() {
        let payload = serde_json::to_value(AppError::validation("missing folder")).unwrap();

        assert_eq!(payload["code"], "VALIDATION_ERROR");
        assert_eq!(payload["message"], "Validation failed: missing folder");
    }
}
