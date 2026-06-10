use rmcp::ErrorData;

use crate::upstream::{error::UpstreamError, redaction::redact_text};

pub(crate) fn upstream_error(error: UpstreamError) -> ErrorData {
    let message = upstream_error_message(&error);
    match error {
        UpstreamError::InvalidInput { .. } | UpstreamError::InvalidBaseUrl { .. } => {
            ErrorData::invalid_params(message, None)
        }
        UpstreamError::HttpStatus { .. }
        | UpstreamError::Transport { .. }
        | UpstreamError::JsonDecode { .. }
        | UpstreamError::UnexpectedShape { .. } => ErrorData::internal_error(message, None),
    }
}

fn upstream_error_message(error: &UpstreamError) -> String {
    match error {
        UpstreamError::InvalidInput { .. } => {
            format!(
                "Upstream error category=invalid_input: {}",
                redact_text(&error.to_string())
            )
        }
        UpstreamError::InvalidBaseUrl { .. } => {
            format!(
                "Upstream error category=invalid_base_url: {}",
                redact_text(&error.to_string())
            )
        }
        UpstreamError::HttpStatus { status, .. } => {
            format!(
                "Upstream error category=http_status status={status}: {}",
                redact_text(&error.to_string())
            )
        }
        UpstreamError::Transport { .. } => {
            format!(
                "Upstream error category=transport: {}",
                redact_text(&error.to_string())
            )
        }
        UpstreamError::JsonDecode { .. } => {
            format!(
                "Upstream error category=json_decode: {}",
                redact_text(&error.to_string())
            )
        }
        UpstreamError::UnexpectedShape { .. } => {
            format!(
                "Upstream error category=unexpected_shape: {}",
                redact_text(&error.to_string())
            )
        }
    }
}
