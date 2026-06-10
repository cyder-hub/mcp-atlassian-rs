use std::{sync::Arc, time::Instant};

use crate::{
    atlassian::request_auth::parse_request_auth_headers_with_oauth_bearer,
    confluence::client::ConfluenceClient,
    context::AppContext,
    gitlab::client::GitlabClient,
    jira::client::JiraClient,
    mcp_errors::upstream_error,
    tool_registry,
    upstream::{error::UpstreamError, redaction::redact_text},
};
use axum::http::{HeaderMap, request::Parts};
use rmcp::{
    ErrorData, RoleServer, ServerHandler,
    handler::server::router::tool::ToolRouter,
    handler::server::tool::ToolCallContext,
    model::{
        CallToolRequestParams, CallToolResult, Implementation, ListToolsResult,
        PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
    tool_handler,
};
use serde_json::{Value, json};

mod confluence_handlers;
mod confluence_values;
mod gitlab_handlers;
mod jira_handlers;
mod jira_payloads;
mod schema;
mod session_auth;
mod tool_log;

pub use session_auth::RequestAuthSessionStore;

use schema::{sanitize_tool_for_clients, sanitize_tools_for_clients};
use tool_log::sanitize_tool_log_arguments;
#[cfg(test)]
use tool_log::{TOOL_LOG_MAX_STRING_CHARS, TOOL_LOG_REDACTED, TOOL_LOG_TRUNCATED};

pub(crate) fn wrap_array(value: Value) -> Value {
    match value {
        Value::Array(items) => json!({ "items": items }),
        other => other,
    }
}
pub const SERVER_NAME: &str = "mcp-atlassian-rs";

#[derive(Clone)]
pub struct AtlassianMcpServer {
    context: Arc<AppContext>,
    tool_router: ToolRouter<Self>,
    session_auth_fingerprints: RequestAuthSessionStore,
}

impl AtlassianMcpServer {
    pub fn new(context: Arc<AppContext>) -> Self {
        Self::with_session_auth_store(context, RequestAuthSessionStore::default())
    }

    pub fn with_session_auth_store(
        context: Arc<AppContext>,
        session_auth_fingerprints: RequestAuthSessionStore,
    ) -> Self {
        Self {
            context,
            tool_router: Self::tool_router(),
            session_auth_fingerprints,
        }
    }

    fn with_context(&self, context: Arc<AppContext>) -> Self {
        Self {
            context,
            tool_router: Self::tool_router(),
            session_auth_fingerprints: self.session_auth_fingerprints.clone(),
        }
    }

    fn tool_router() -> ToolRouter<Self> {
        Self::jira_tool_router() + Self::confluence_tool_router() + Self::gitlab_tool_router()
    }

    fn scoped_for_request_context(
        &self,
        request_context: &RequestContext<RoleServer>,
    ) -> Result<Self, ErrorData> {
        let Some(parts) = request_context.extensions.get::<Parts>() else {
            return Ok(self.clone());
        };

        self.scoped_for_request_headers(&parts.headers)
    }

    fn scoped_for_request_headers(&self, headers: &HeaderMap) -> Result<Self, ErrorData> {
        let request_auth = parse_request_auth_headers_with_oauth_bearer(
            headers,
            self.context.ignore_header_auth(),
            self.context.allowed_url_domains(),
            self.context.atlassian_oauth_enabled(),
        )
        .map_err(|error| ErrorData::invalid_params(redact_text(&error.to_string()), None))?;

        self.session_auth_fingerprints
            .enforce_request_headers(headers, &request_auth.fingerprint)?;

        if request_auth.has_overrides() {
            Ok(self.with_context(Arc::new(self.context.with_request_auth(&request_auth))))
        } else {
            Ok(self.clone())
        }
    }

    fn current_tools_result(&self) -> ListToolsResult {
        ListToolsResult {
            tools: sanitize_tools_for_clients(
                self.filtered_tools_from(self.tool_router.list_all()),
            ),
            ..Default::default()
        }
    }

    fn filtered_tools_from<I>(&self, tools: I) -> Vec<Tool>
    where
        I: IntoIterator<Item = Tool>,
    {
        tool_registry::visible_tools(tools, &self.context)
    }

    fn guard_registered_tool_call(&self, name: &str) -> Result<(), ErrorData> {
        if !self.tool_router.has_route(name) {
            return Err(ErrorData::invalid_params("tool not available", None));
        }

        tool_registry::guard_tool_call(name, &self.context)
    }

    fn jira_client(&self) -> Result<JiraClient, ErrorData> {
        let Some(config) = self.context.jira_config() else {
            return Err(ErrorData::invalid_params("Jira is not configured", None));
        };

        JiraClient::new(config.clone()).map_err(upstream_error)
    }

    #[allow(dead_code)]
    fn confluence_client(&self) -> Result<ConfluenceClient, ErrorData> {
        let Some(config) = self.context.confluence_config() else {
            return Err(ErrorData::invalid_params(
                "Confluence is not configured",
                None,
            ));
        };

        ConfluenceClient::new(config.clone()).map_err(upstream_error)
    }

    #[allow(dead_code)]
    fn gitlab_client(&self) -> Result<GitlabClient, ErrorData> {
        let Some(config) = self.context.gitlab_config() else {
            return Err(ErrorData::invalid_params("GitLab is not configured", None));
        };

        GitlabClient::new(config.clone()).map_err(upstream_error)
    }

    #[cfg(test)]
    fn guard_tool_call_with_metadata<F>(
        &self,
        name: &str,
        route_exists: bool,
        metadata_for: F,
    ) -> Result<(), ErrorData>
    where
        F: Fn(&str) -> Option<tool_registry::ToolMetadata>,
    {
        if !route_exists {
            return Err(ErrorData::invalid_params("tool not available", None));
        }

        tool_registry::guard_tool_call_with_metadata(name, &self.context, metadata_for)
    }

    #[cfg(test)]
    fn filtered_tools_from_with_metadata<I, F>(&self, tools: I, metadata_for: F) -> Vec<Tool>
    where
        I: IntoIterator<Item = Tool>,
        F: Fn(&str) -> Option<tool_registry::ToolMetadata>,
    {
        tool_registry::visible_tools_with_metadata(tools, &self.context, metadata_for)
    }
}

impl Default for AtlassianMcpServer {
    fn default() -> Self {
        Self::new(Arc::new(AppContext::default()))
    }
}
fn required_non_empty_arg(value: String, field_name: &'static str) -> Result<String, ErrorData> {
    let value = value.trim();
    if value.is_empty() {
        Err(upstream_error(UpstreamError::invalid_input(format!(
            "{field_name} must not be empty"
        ))))
    } else {
        Ok(value.to_string())
    }
}

fn optional_non_empty_arg(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn optional_positive_i64_arg(
    value: Option<i64>,
    field_name: &'static str,
) -> Result<Option<i64>, ErrorData> {
    match value {
        Some(value) if value <= 0 => Err(upstream_error(UpstreamError::invalid_input(format!(
            "{field_name} must be positive"
        )))),
        value => Ok(value),
    }
}

fn optional_positive_u64_arg(
    value: Option<u64>,
    field_name: &'static str,
) -> Result<Option<u64>, ErrorData> {
    match value {
        Some(0) => Err(upstream_error(UpstreamError::invalid_input(format!(
            "{field_name} must be positive"
        )))),
        value => Ok(value),
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for AtlassianMcpServer {
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let tool_name = request.name.to_string();
        let debug_arguments = tracing::enabled!(tracing::Level::DEBUG)
            .then(|| sanitize_tool_log_arguments(request.arguments.as_ref()));
        let started_at = Instant::now();

        if let Some(arguments) = debug_arguments.as_ref() {
            tracing::debug!(
                tool = %tool_name,
                arguments = %arguments,
                "MCP tool call started"
            );
        }

        let result = async {
            let scoped_server = self.scoped_for_request_context(&context)?;
            scoped_server.guard_registered_tool_call(tool_name.as_str())?;

            let tool_call_context = ToolCallContext::new(&scoped_server, request, context);
            scoped_server.tool_router.call(tool_call_context).await
        }
        .await;
        let elapsed_ms = started_at.elapsed().as_millis();

        match &result {
            Ok(_) => {
                tracing::debug!(
                    tool = %tool_name,
                    elapsed_ms,
                    "MCP tool call completed"
                );
            }
            Err(error) => {
                tracing::warn!(
                    tool = %tool_name,
                    "MCP tool call failed"
                );
                if let Some(arguments) = debug_arguments.as_ref() {
                    tracing::debug!(
                        tool = %tool_name,
                        arguments = %arguments,
                        error_code = error.code.0,
                        error_message = %redact_text(error.message.as_ref()),
                        elapsed_ms,
                        "MCP tool call failed details"
                    );
                }
            }
        }

        result
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(self
            .scoped_for_request_context(&context)?
            .current_tools_result())
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tool_router
            .get(name)
            .cloned()
            .filter(|tool| !self.filtered_tools_from([tool.clone()]).is_empty())
            .map(sanitize_tool_for_clients)
    }

    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(SERVER_NAME, env!("CARGO_PKG_VERSION")))
            .with_instructions(format!(
                "Rust MCP Atlassian exposes 88 Jira, Confluence, and GitLab business tools. Tool visibility is controlled by TOOL_PROFILE, TOOLSETS, ENABLED_TOOLS, and DISABLED_TOOLS. Jira, Confluence, and GitLab tools are available when their service configuration and authentication are complete. See docs/support-matrix.md for per-tool and runtime support status."
            ))
    }
}

#[cfg(test)]
mod tests;
