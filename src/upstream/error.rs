#![allow(dead_code)]

use std::fmt::{Display, Formatter};

use serde_json::Value;

use crate::upstream::redaction::{REDACTED, is_sensitive_query_key, redact_text};

#[derive(Debug)]
pub enum UpstreamError {
    InvalidBaseUrl { message: String },
    HttpStatus { status: u16, message: String },
    Transport { message: String },
    JsonDecode { message: String },
    UnexpectedShape { message: String },
    InvalidInput { message: String },
}

impl UpstreamError {
    pub fn transport(error: reqwest::Error) -> Self {
        Self::Transport {
            message: redact_text(&error.without_url().to_string()),
        }
    }

    pub fn json_decode(error: reqwest::Error) -> Self {
        Self::JsonDecode {
            message: redact_text(&error.without_url().to_string()),
        }
    }

    pub fn json_decode_body(error: serde_json::Error, request_context: Option<&str>) -> Self {
        let error = error.to_string();
        let message = request_context
            .map(str::trim)
            .filter(|context| !context.is_empty())
            .map_or_else(
                || format!("error decoding response body: {error}"),
                |context| {
                    format!(
                        "error decoding response body from {}: {error}",
                        redact_text(context)
                    )
                },
            );

        Self::JsonDecode {
            message: redact_text(&message),
        }
    }

    pub fn invalid_base_url(message: impl Into<String>) -> Self {
        Self::InvalidBaseUrl {
            message: redact_text(&message.into()),
        }
    }

    pub fn unexpected_shape(message: impl Into<String>) -> Self {
        Self::UnexpectedShape {
            message: redact_text(&message.into()),
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: redact_text(&message.into()),
        }
    }

    pub fn http_status(status: reqwest::StatusCode, message: impl Into<String>) -> Self {
        Self::HttpStatus {
            status: status.as_u16(),
            message: sanitize_message(message.into()),
        }
    }
}

impl Display for UpstreamError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidBaseUrl { message } => {
                write!(formatter, "invalid upstream base URL: {message}")
            }
            Self::HttpStatus { status, message } => {
                write!(formatter, "upstream API returned HTTP {status}: {message}")
            }
            Self::Transport { message } => {
                write!(formatter, "upstream transport error: {message}")
            }
            Self::JsonDecode { message } => {
                write!(formatter, "upstream JSON decode error: {message}")
            }
            Self::UnexpectedShape { message } => {
                write!(formatter, "upstream response shape error: {message}")
            }
            Self::InvalidInput { message } => {
                write!(formatter, "invalid upstream input: {message}")
            }
        }
    }
}

impl std::error::Error for UpstreamError {}

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
                    .map(redact_text),
            );
        }

        if let Some(errors) = object.get("errors").and_then(Value::as_object) {
            for (key, value) in errors {
                if let Some(message) = value.as_str().map(str::trim)
                    && !message.is_empty()
                {
                    let value = if is_sensitive_query_key(key) {
                        REDACTED.to_string()
                    } else {
                        redact_text(message)
                    };
                    parts.push(format!("{key}: {value}"));
                }
            }
        }

        if !parts.is_empty() {
            return redact_text(&parts.join("; ")).chars().take(500).collect();
        }
    }

    "request failed with non-empty error response".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_error_display_contains_status_and_jira_error_summary() {
        let error = UpstreamError::http_status(
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
        let error = UpstreamError::http_status(
            reqwest::StatusCode::UNAUTHORIZED,
            format!("authentication failed for {echoed_header}"),
        );
        let output = error.to_string();

        assert!(output.contains("HTTP 401"));
        assert!(output.contains("non-empty error response"));
        assert!(!output.contains("Bearer"));
        assert!(!output.contains("test-pat-value"));
    }

    #[test]
    fn status_error_display_redacts_json_error_summary() {
        let error = UpstreamError::http_status(
            reqwest::StatusCode::BAD_REQUEST,
            r#"{"errorMessages":["Authorization Bearer json-secret-token"],"errors":{"token":"raw-token-value","issue":"failed /path?token=query-secret"}}"#,
        );
        let output = error.to_string();

        assert!(output.contains("HTTP 400"));
        assert!(output.contains("Bearer <redacted>"));
        assert!(output.contains("token: <redacted>"));
        assert!(!output.contains("json-secret-token"));
        assert!(!output.contains("raw-token-value"));
    }

    #[test]
    fn json_decode_body_redacts_request_context() {
        let error = serde_json::from_str::<Value>("not-json").unwrap_err();
        let output = UpstreamError::json_decode_body(
            error,
            Some("GET /rest/api/2/issue/ABC-1?token=query-secret&client=abc"),
        )
        .to_string();

        assert!(output.contains("token=<redacted>") || output.contains("token=%3Credacted%3E"));
        assert!(!output.contains("query-secret"));
    }
}
