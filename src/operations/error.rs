use std::fmt::{Display, Formatter};

use serde::Serialize;

use crate::{
    tool_registry::ToolGuardError,
    upstream::{error::UpstreamError, redaction::redact_text},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationErrorCategory {
    InvalidInput,
    Config,
    ServiceUnavailable,
    DisabledTool,
    HttpStatus,
    Transport,
    JsonDecode,
    UnexpectedShape,
    Business,
}

impl OperationErrorCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::Config => "config",
            Self::ServiceUnavailable => "service_unavailable",
            Self::DisabledTool => "disabled_tool",
            Self::HttpStatus => "http_status",
            Self::Transport => "transport",
            Self::JsonDecode => "json_decode",
            Self::UnexpectedShape => "unexpected_shape",
            Self::Business => "business",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationError {
    pub category: OperationErrorCategory,
    pub message: String,
}

impl OperationError {
    pub fn new(category: OperationErrorCategory, message: impl Into<String>) -> Self {
        Self {
            category,
            message: redact_text(&message.into()),
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(OperationErrorCategory::InvalidInput, message)
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::new(OperationErrorCategory::Config, message)
    }

    pub fn business(message: impl Into<String>) -> Self {
        Self::new(OperationErrorCategory::Business, message)
    }

    pub fn from_tool_guard(error: ToolGuardError) -> Self {
        match error {
            ToolGuardError::UnknownTool => Self::new(
                OperationErrorCategory::ServiceUnavailable,
                "operation is not available",
            ),
            ToolGuardError::DisabledTool => Self::new(
                OperationErrorCategory::DisabledTool,
                "operation is disabled by runtime tool controls",
            ),
            ToolGuardError::ServiceUnavailable => Self::new(
                OperationErrorCategory::ServiceUnavailable,
                "operation service is not configured",
            ),
        }
    }

    pub fn from_upstream(error: UpstreamError) -> Self {
        match &error {
            UpstreamError::InvalidInput { .. } => Self::new(
                OperationErrorCategory::InvalidInput,
                upstream_message("invalid_input", &error),
            ),
            UpstreamError::InvalidBaseUrl { .. } => Self::new(
                OperationErrorCategory::Config,
                upstream_message("config", &error),
            ),
            UpstreamError::HttpStatus { status, .. } => Self::new(
                OperationErrorCategory::HttpStatus,
                format!(
                    "Upstream error category=http_status status={status}: {}",
                    redact_text(&error.to_string())
                ),
            ),
            UpstreamError::Transport { .. } => Self::new(
                OperationErrorCategory::Transport,
                upstream_message("transport", &error),
            ),
            UpstreamError::JsonDecode { .. } => Self::new(
                OperationErrorCategory::JsonDecode,
                upstream_message("json_decode", &error),
            ),
            UpstreamError::UnexpectedShape { .. } => Self::new(
                OperationErrorCategory::UnexpectedShape,
                upstream_message("unexpected_shape", &error),
            ),
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self.category {
            OperationErrorCategory::InvalidInput => 2,
            OperationErrorCategory::Config
            | OperationErrorCategory::ServiceUnavailable
            | OperationErrorCategory::DisabledTool => 3,
            OperationErrorCategory::HttpStatus
            | OperationErrorCategory::Transport
            | OperationErrorCategory::JsonDecode
            | OperationErrorCategory::UnexpectedShape => 4,
            OperationErrorCategory::Business => 5,
        }
    }
}

impl Display for OperationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "operation error category={}: {}",
            self.category.as_str(),
            self.message
        )
    }
}

impl std::error::Error for OperationError {}

fn upstream_message(category: &str, error: &UpstreamError) -> String {
    format!(
        "Upstream error category={category}: {}",
        redact_text(&error.to_string())
    )
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;

    use super::*;

    #[test]
    fn operations_error_exit_code_mapping_covers_categories() {
        assert_eq!(OperationError::invalid_input("bad args").exit_code(), 2);
        assert_eq!(OperationError::config("missing env").exit_code(), 3);
        assert_eq!(
            OperationError::new(OperationErrorCategory::ServiceUnavailable, "missing").exit_code(),
            3
        );
        assert_eq!(
            OperationError::new(OperationErrorCategory::DisabledTool, "disabled").exit_code(),
            3
        );
        assert_eq!(
            OperationError::from_upstream(UpstreamError::http_status(
                StatusCode::BAD_GATEWAY,
                r#"{"errorMessages":["bad gateway"]}"#,
            ))
            .exit_code(),
            4
        );
        assert_eq!(
            OperationError::new(OperationErrorCategory::Transport, "transport").exit_code(),
            4
        );
        assert_eq!(OperationError::business("partial failure").exit_code(), 5);
    }

    #[test]
    fn operations_error_redacts_messages() {
        let error = OperationError::invalid_input("token=secret-value");

        assert!(error.message.contains("token=<redacted>"));
        assert!(!error.message.contains("secret-value"));
    }
}
