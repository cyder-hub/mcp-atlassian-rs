#![allow(dead_code)]

use rmcp::{ErrorData, model::CallToolResult};
use serde_json::Value;

use crate::{
    confluence::client::ConfluenceClient, context::AppContext, gitlab::client::GitlabClient,
    jira::client::JiraClient, tool_registry,
};

pub mod confluence;
pub mod error;
pub mod gitlab;
pub mod jira;
pub mod output;

pub use error::{OperationError, OperationErrorCategory};
pub use output::OutputPresentation;

#[derive(Debug, Clone, PartialEq)]
pub struct OperationResult {
    pub value: Value,
    pub is_error: bool,
    pub presentation: OutputPresentation,
}

impl OperationResult {
    pub fn success(value: Value) -> Self {
        Self {
            value,
            is_error: false,
            presentation: OutputPresentation::KeyValue,
        }
    }

    pub fn structured_error(value: Value) -> Self {
        Self {
            value,
            is_error: true,
            presentation: OutputPresentation::KeyValue,
        }
    }

    pub fn with_presentation(mut self, presentation: OutputPresentation) -> Self {
        self.presentation = presentation;
        self
    }
}

pub fn guard_operation(tool_name: &str, context: &AppContext) -> Result<(), OperationError> {
    tool_registry::guard_operation_access(tool_name, context)
        .map(|_| ())
        .map_err(OperationError::from_tool_guard)
}

pub fn jira_client(context: &AppContext) -> Result<JiraClient, OperationError> {
    let config = context
        .jira_config()
        .cloned()
        .ok_or_else(|| OperationError::config("Jira is not configured"))?;
    JiraClient::new(config).map_err(OperationError::from_upstream)
}

pub fn confluence_client(context: &AppContext) -> Result<ConfluenceClient, OperationError> {
    let config = context
        .confluence_config()
        .cloned()
        .ok_or_else(|| OperationError::config("Confluence is not configured"))?;
    ConfluenceClient::new(config).map_err(OperationError::from_upstream)
}

pub fn gitlab_client(context: &AppContext) -> Result<GitlabClient, OperationError> {
    let config = context
        .gitlab_config()
        .cloned()
        .ok_or_else(|| OperationError::config("GitLab is not configured"))?;
    GitlabClient::new(config).map_err(OperationError::from_upstream)
}

pub fn operation_result_to_mcp(result: OperationResult) -> CallToolResult {
    if result.is_error {
        CallToolResult::structured_error(result.value)
    } else {
        CallToolResult::structured(result.value)
    }
}

pub fn operation_error_to_mcp(error: OperationError) -> ErrorData {
    match error.category {
        OperationErrorCategory::InvalidInput
        | OperationErrorCategory::Config
        | OperationErrorCategory::ServiceUnavailable
        | OperationErrorCategory::DisabledTool => {
            ErrorData::invalid_params(error.to_string(), None)
        }
        OperationErrorCategory::HttpStatus
        | OperationErrorCategory::Transport
        | OperationErrorCategory::JsonDecode
        | OperationErrorCategory::UnexpectedShape
        | OperationErrorCategory::Business => ErrorData::internal_error(error.to_string(), None),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::json;

    use crate::{
        config::{HttpConfig, RuntimeConfig},
        gitlab::config::GitlabConfig,
        gitlab::tools::GITLAB_GET_PROJECT_TOOL_NAME,
        tool_registry::default_toolsets,
        upstream::{auth::UpstreamAuth, custom_headers::CustomHeaders, proxy::ProxyConfig},
    };

    use super::*;

    #[test]
    fn operations_guard_allows_enabled_tool_with_available_service() {
        let context = AppContext::from_config(&RuntimeConfig {
            gitlab: Some(gitlab_config()),
            mcp_enabled_toolsets: default_toolsets(),
            http: HttpConfig::default(),
            ..RuntimeConfig::default()
        });

        assert!(guard_operation(GITLAB_GET_PROJECT_TOOL_NAME, &context).is_ok());
    }

    #[test]
    fn operations_guard_ignores_mcp_disabled_tools() {
        let context = AppContext::from_config(&RuntimeConfig {
            gitlab: Some(gitlab_config()),
            mcp_disabled_tools: BTreeSet::from([GITLAB_GET_PROJECT_TOOL_NAME.to_string()]),
            mcp_enabled_toolsets: default_toolsets(),
            http: HttpConfig::default(),
            ..RuntimeConfig::default()
        });

        assert!(guard_operation(GITLAB_GET_PROJECT_TOOL_NAME, &context).is_ok());
    }

    #[test]
    fn operations_guard_blocks_missing_service() {
        let context = AppContext::from_config(&RuntimeConfig {
            mcp_enabled_toolsets: default_toolsets(),
            http: HttpConfig::default(),
            ..RuntimeConfig::default()
        });
        let error = guard_operation(GITLAB_GET_PROJECT_TOOL_NAME, &context).unwrap_err();

        assert_eq!(error.category, OperationErrorCategory::ServiceUnavailable);
        assert_eq!(error.exit_code(), 3);
    }

    #[test]
    fn operations_mcp_adapter_uses_structured_error_flag() {
        let result = operation_result_to_mcp(OperationResult::structured_error(json!({
            "success": false,
            "error": "not available"
        })));

        assert_eq!(
            result.structured_content.as_ref().unwrap()["success"],
            json!(false)
        );
        assert_eq!(result.is_error, Some(true));
    }

    fn gitlab_config() -> GitlabConfig {
        GitlabConfig {
            base_url: "https://gitlab.example".to_string(),
            auth: UpstreamAuth::HeaderToken {
                header_name: reqwest::header::HeaderName::from_static("private-token"),
                token: "test-token".to_string(),
            },
            ssl_verify: true,
            proxy: ProxyConfig::default(),
            custom_headers: CustomHeaders::default(),
            mtls: None,
            projects_filter: BTreeSet::new(),
            timeout_seconds: 75,
        }
    }
}
