use std::{sync::Arc, time::Instant};

#[cfg(test)]
use crate::upstream::error::UpstreamError;
use crate::{
    confluence::client::ConfluenceClient, context::AppContext, gitlab::client::GitlabClient,
    mcp_errors::upstream_error, tool_registry, upstream::redaction::redact_text,
};
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

mod confluence_handlers;
mod gitlab_handlers;
mod jira_handlers;
#[cfg(test)]
#[allow(dead_code)]
mod jira_payloads;
mod schema;
mod tool_log;

use schema::{sanitize_tool_for_clients, sanitize_tools_for_clients};
use tool_log::sanitize_tool_log_arguments;
#[cfg(test)]
use tool_log::{TOOL_LOG_MAX_STRING_CHARS, TOOL_LOG_REDACTED, TOOL_LOG_TRUNCATED};

pub const SERVER_NAME: &str = "workhub-rs";

#[derive(Clone)]
pub struct WorkhubMcpServer {
    context: Arc<AppContext>,
    tool_router: ToolRouter<Self>,
}

impl WorkhubMcpServer {
    pub fn new(context: Arc<AppContext>) -> Self {
        Self {
            context,
            tool_router: Self::tool_router(),
        }
    }

    fn tool_router() -> ToolRouter<Self> {
        Self::jira_tool_router() + Self::confluence_tool_router() + Self::gitlab_tool_router()
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

impl Default for WorkhubMcpServer {
    fn default() -> Self {
        Self::new(Arc::new(AppContext::default()))
    }
}
#[cfg(test)]
#[allow(dead_code)]
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

#[cfg(test)]
#[allow(dead_code)]
fn optional_non_empty_arg(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for WorkhubMcpServer {
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
            self.guard_registered_tool_call(tool_name.as_str())?;

            let tool_call_context = ToolCallContext::new(self, request, context);
            self.tool_router.call(tool_call_context).await
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
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(self.current_tools_result())
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
                "workhub-rs exposes 85 Jira, Confluence, and GitLab business tools. MCP tool visibility is controlled by MCP_TOOL_PROFILE, MCP_TOOLSETS, MCP_ENABLED_TOOLS, and MCP_DISABLED_TOOLS. Jira, Confluence, and GitLab tools are available when their service configuration and authentication are complete. The resource CLI ignores MCP tool visibility controls. See docs/support-matrix.md for per-tool and runtime support status."
            ))
    }
}

#[cfg(test)]
mod tests;
