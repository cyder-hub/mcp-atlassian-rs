#![allow(dead_code)]

use std::fmt::{Display, Formatter};

use serde_json::Value;

#[derive(Debug)]
pub enum AtlassianError {
    InvalidBaseUrl { message: String },
    HttpStatus { status: u16, message: String },
    Transport { message: String },
    JsonDecode { message: String },
    UnexpectedShape { message: String },
    InvalidInput { message: String },
}

impl AtlassianError {
    pub fn transport(error: reqwest::Error) -> Self {
        Self::Transport {
            message: error.without_url().to_string(),
        }
    }

    pub fn json_decode(error: reqwest::Error) -> Self {
        Self::JsonDecode {
            message: error.without_url().to_string(),
        }
    }

    pub fn invalid_base_url(message: impl Into<String>) -> Self {
        Self::InvalidBaseUrl {
            message: message.into(),
        }
    }

    pub fn unexpected_shape(message: impl Into<String>) -> Self {
        Self::UnexpectedShape {
            message: message.into(),
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }

    pub fn http_status(status: reqwest::StatusCode, message: impl Into<String>) -> Self {
        Self::HttpStatus {
            status: status.as_u16(),
            message: sanitize_message(message.into()),
        }
    }
}

impl Display for AtlassianError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidBaseUrl { message } => {
                write!(formatter, "invalid Atlassian base URL: {message}")
            }
            Self::HttpStatus { status, message } => {
                write!(formatter, "Atlassian API returned HTTP {status}: {message}")
            }
            Self::Transport { message } => {
                write!(formatter, "Atlassian transport error: {message}")
            }
            Self::JsonDecode { message } => {
                write!(formatter, "Atlassian JSON decode error: {message}")
            }
            Self::UnexpectedShape { message } => {
                write!(formatter, "Atlassian response shape error: {message}")
            }
            Self::InvalidInput { message } => {
                write!(formatter, "invalid Atlassian input: {message}")
            }
        }
    }
}

impl std::error::Error for AtlassianError {}

fn sanitize_message(message: String) -> String {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return "request failed".to_string();
    }

    if let Ok(Value::Object(object)) = serde_json::from_str::<Value>(trimmed) {
        let mut parts = Vec::new();

        if let Some(messages) = object.get("errorMessages").and_then(Value::as_array) {
            parts.extend(
                messages
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::trim)
                    .filter(|message| !message.is_empty())
                    .map(ToString::to_string),
            );
        }

        if let Some(errors) = object.get("errors").and_then(Value::as_object) {
            for (key, value) in errors {
                if let Some(message) = value.as_str().map(str::trim)
                    && !message.is_empty()
                {
                    parts.push(format!("{key}: {message}"));
                }
            }
        }

        if !parts.is_empty() {
            return parts.join("; ").chars().take(500).collect();
        }
    }

    "request failed with non-empty error response".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_error_display_contains_status_and_jira_error_summary() {
        let error = AtlassianError::http_status(
            reqwest::StatusCode::NOT_FOUND,
            r#"{"errorMessages":["issue not found"],"errors":{"issue":"missing"}}"#,
        );
        let output = error.to_string();

        assert!(output.contains("HTTP 404"));
        assert!(output.contains("issue not found"));
        assert!(output.contains("issue: missing"));
    }

    #[test]
    fn status_error_display_does_not_echo_plain_response_body() {
        let echoed_header = format!("Bearer {}", "test-pat-value");
        let error = AtlassianError::http_status(
            reqwest::StatusCode::UNAUTHORIZED,
            format!("authentication failed for {echoed_header}"),
        );
        let output = error.to_string();

        assert!(output.contains("HTTP 401"));
        assert!(output.contains("non-empty error response"));
        assert!(!output.contains("Bearer"));
        assert!(!output.contains("test-pat-value"));
    }
}
