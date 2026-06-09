use rmcp::ErrorData;

use crate::atlassian::{error::AtlassianError, redaction::redact_text};

pub(crate) fn atlassian_error(error: AtlassianError) -> ErrorData {
    let message = atlassian_error_message(&error);
    match error {
        AtlassianError::InvalidInput { .. } | AtlassianError::InvalidBaseUrl { .. } => {
            ErrorData::invalid_params(message, None)
        }
        AtlassianError::HttpStatus { .. }
        | AtlassianError::Transport { .. }
        | AtlassianError::JsonDecode { .. }
        | AtlassianError::UnexpectedShape { .. } => ErrorData::internal_error(message, None),
    }
}

fn atlassian_error_message(error: &AtlassianError) -> String {
    match error {
        AtlassianError::InvalidInput { .. } => {
            format!(
                "Atlassian error category=invalid_input: {}",
                redact_text(&error.to_string())
            )
        }
        AtlassianError::InvalidBaseUrl { .. } => {
            format!(
                "Atlassian error category=invalid_base_url: {}",
                redact_text(&error.to_string())
            )
        }
        AtlassianError::HttpStatus { status, .. } => {
            format!(
                "Atlassian error category=http_status status={status}: {}",
                redact_text(&error.to_string())
            )
        }
        AtlassianError::Transport { .. } => {
            format!(
                "Atlassian error category=transport: {}",
                redact_text(&error.to_string())
            )
        }
        AtlassianError::JsonDecode { .. } => {
            format!(
                "Atlassian error category=json_decode: {}",
                redact_text(&error.to_string())
            )
        }
        AtlassianError::UnexpectedShape { .. } => {
            format!(
                "Atlassian error category=unexpected_shape: {}",
                redact_text(&error.to_string())
            )
        }
    }
}
