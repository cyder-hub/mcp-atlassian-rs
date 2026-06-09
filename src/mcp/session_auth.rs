use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use crate::{
    atlassian::{
        redaction::redact_text,
        request_auth::{RequestAuthFingerprint, parse_request_auth_headers_with_oauth_bearer},
    },
    context::AppContext,
};
use axum::http::{HeaderMap, HeaderValue};
use rmcp::ErrorData;

const HEADER_MCP_SESSION_ID: &str = "mcp-session-id";

#[derive(Clone, Default)]
pub struct RequestAuthSessionStore {
    fingerprints: Arc<Mutex<BTreeMap<String, RequestAuthFingerprint>>>,
}

impl RequestAuthSessionStore {
    pub fn parse_and_enforce_headers(
        &self,
        headers: &HeaderMap,
        context: &AppContext,
    ) -> Result<RequestAuthFingerprint, ErrorData> {
        let request_auth = parse_request_auth_headers_with_oauth_bearer(
            headers,
            context.ignore_header_auth(),
            context.allowed_url_domains(),
            context.atlassian_oauth_enabled(),
        )
        .map_err(|error| ErrorData::invalid_params(redact_text(&error.to_string()), None))?;

        self.enforce_request_headers(headers, &request_auth.fingerprint)?;
        Ok(request_auth.fingerprint)
    }

    pub fn enforce_request_headers(
        &self,
        headers: &HeaderMap,
        fingerprint: &RequestAuthFingerprint,
    ) -> Result<(), ErrorData> {
        let Some(session_id) = session_id_from_headers(headers) else {
            return Ok(());
        };

        self.bind_session_id(&session_id, fingerprint)
    }

    pub fn bind_response_headers(
        &self,
        headers: &HeaderMap,
        fingerprint: &RequestAuthFingerprint,
    ) -> Result<(), ErrorData> {
        let Some(session_id) = session_id_from_headers(headers) else {
            return Ok(());
        };

        self.bind_session_id(&session_id, fingerprint)
    }

    fn bind_session_id(
        &self,
        session_id: &str,
        fingerprint: &RequestAuthFingerprint,
    ) -> Result<(), ErrorData> {
        let mut fingerprints = self.fingerprints.lock().map_err(|_| {
            ErrorData::internal_error("session auth fingerprint store is unavailable", None)
        })?;

        match fingerprints.get(session_id) {
            Some(existing) if existing == fingerprint => Ok(()),
            Some(_) => Err(ErrorData::invalid_params(
                "per-request authentication changed for MCP session",
                None,
            )),
            None => {
                fingerprints.insert(session_id.to_string(), fingerprint.clone());
                Ok(())
            }
        }
    }
}

fn session_id_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(HEADER_MCP_SESSION_ID)
        .or_else(|| headers.get("Mcp-Session-Id"))
        .and_then(header_value_to_trimmed_string)
}

fn header_value_to_trimmed_string(value: &HeaderValue) -> Option<String> {
    value
        .to_str()
        .ok()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}
