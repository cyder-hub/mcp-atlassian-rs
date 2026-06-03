use std::sync::Arc;

use crate::{
    atlassian::error::AtlassianError,
    context::AppContext,
    jira::{
        client::{FieldOptionsRequest, GetIssueRequest, JiraClient, SearchRequest},
        formatting::{parse_optional_object, parse_optional_string_list},
        tools::{
            JiraAddCommentArgs, JiraEditCommentArgs, JiraGetFieldOptionsArgs, JiraGetIssueArgs,
            JiraGetProjectIssuesArgs, JiraGetTransitionsArgs, JiraSearchArgs, JiraSearchFieldsArgs,
            JiraTransitionIssueArgs,
        },
    },
    tool_registry,
};
use rmcp::{
    ErrorData, RoleServer, ServerHandler,
    handler::server::router::tool::ToolRouter,
    handler::server::tool::ToolCallContext,
    handler::server::wrapper::Parameters,
    model::{
        CallToolRequestParams, CallToolResult, Implementation, ListToolsResult,
        PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
    tool, tool_handler, tool_router,
};

pub const SERVER_NAME: &str = "mcp-atlassian-rs";

const MIGRATION_STATUS: &str = "mcp-atlassian-rs Stage 2 Jira core migration is complete. \
The Stage 1 MCP runtime/control plane and Stage 2 Jira config/auth/client/models/tool handlers are implemented. \
Jira core tools are available when Jira configuration and authentication are complete. \
Mock REST tests, MCP smoke checks, README, and the migration ledger are up to date.";

#[derive(Clone)]
pub struct AtlassianMcpServer {
    context: Arc<AppContext>,
    tool_router: ToolRouter<Self>,
}

impl AtlassianMcpServer {
    pub fn new(context: Arc<AppContext>) -> Self {
        Self {
            context,
            tool_router: Self::tool_router(),
        }
    }

    fn current_tools_result(&self) -> ListToolsResult {
        ListToolsResult {
            tools: self.filtered_tools_from(self.tool_router.list_all()),
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

        JiraClient::new(config.clone()).map_err(jira_error)
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

#[tool_router(router = tool_router)]
impl AtlassianMcpServer {
    #[tool(description = "Report the current Rust migration status for MCP Atlassian")]
    fn migration_status(&self) -> String {
        MIGRATION_STATUS.to_string()
    }

    #[tool(description = "Get a Jira issue by key")]
    async fn jira_get_issue(
        &self,
        Parameters(args): Parameters<JiraGetIssueArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let fields = parse_optional_string_list(args.fields, "fields").map_err(jira_error)?;
        let expand = parse_optional_string_list(args.expand, "expand").map_err(jira_error)?;
        let properties =
            parse_optional_string_list(args.properties, "properties").map_err(jira_error)?;
        let value = self
            .jira_client()?
            .get_issue(GetIssueRequest {
                issue_key: args.issue_key,
                fields,
                expand,
                comment_limit: args.comment_limit,
                properties,
                update_history: args.update_history,
            })
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Search Jira issues with JQL")]
    async fn jira_search(
        &self,
        Parameters(args): Parameters<JiraSearchArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let fields = parse_optional_string_list(args.fields, "fields").map_err(jira_error)?;
        let expand = parse_optional_string_list(args.expand, "expand").map_err(jira_error)?;
        let projects_filter = parse_optional_string_list(args.projects_filter, "projects_filter")
            .map_err(jira_error)?;
        let value = self
            .jira_client()?
            .search(SearchRequest {
                jql: args.jql,
                fields,
                limit: args.limit,
                start_at: args.start_at,
                projects_filter,
                expand,
                page_token: args.page_token,
            })
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "List Jira issues for a project")]
    async fn jira_get_project_issues(
        &self,
        Parameters(args): Parameters<JiraGetProjectIssuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_project_issues(args.project_key, args.limit, args.start_at)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Search Jira fields by keyword")]
    async fn jira_search_fields(
        &self,
        Parameters(args): Parameters<JiraSearchFieldsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .search_fields(args.keyword, args.limit)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get options for a Jira field")]
    async fn jira_get_field_options(
        &self,
        Parameters(args): Parameters<JiraGetFieldOptionsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_field_options(FieldOptionsRequest {
                field_id: args.field_id,
                context_id: args.context_id,
                project_key: args.project_key,
                issue_type: args.issue_type,
                contains: args.contains,
                return_limit: args.return_limit,
                values_only: args.values_only.unwrap_or(false),
            })
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Add a comment to a Jira issue")]
    async fn jira_add_comment(
        &self,
        Parameters(args): Parameters<JiraAddCommentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let visibility =
            parse_optional_object(args.visibility, "visibility").map_err(jira_error)?;
        let value = self
            .jira_client()?
            .add_comment(args.issue_key, args.body, visibility)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Edit a Jira issue comment")]
    async fn jira_edit_comment(
        &self,
        Parameters(args): Parameters<JiraEditCommentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let visibility =
            parse_optional_object(args.visibility, "visibility").map_err(jira_error)?;
        let value = self
            .jira_client()?
            .edit_comment(args.issue_key, args.comment_id, args.body, visibility)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get available transitions for a Jira issue")]
    async fn jira_get_transitions(
        &self,
        Parameters(args): Parameters<JiraGetTransitionsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_transitions(args.issue_key)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Transition a Jira issue")]
    async fn jira_transition_issue(
        &self,
        Parameters(args): Parameters<JiraTransitionIssueArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let fields = parse_optional_object(args.fields, "fields").map_err(jira_error)?;
        let value = self
            .jira_client()?
            .transition_issue(args.issue_key, args.transition_id, fields, args.comment)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }
}

fn jira_error(error: AtlassianError) -> ErrorData {
    match error {
        AtlassianError::InvalidInput { .. } => ErrorData::invalid_params(error.to_string(), None),
        _ => ErrorData::internal_error(error.to_string(), None),
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for AtlassianMcpServer {
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_registered_tool_call(request.name.as_ref())?;

        let tool_call_context = ToolCallContext::new(self, request, context);
        self.tool_router.call(tool_call_context).await
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
    }

    fn get_info(&self) -> ServerInfo {
        let access_mode = if self.context.read_only() {
            "read-only"
        } else {
            "read/write"
        };

        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(SERVER_NAME, env!("CARGO_PKG_VERSION")))
            .with_instructions(format!(
                "Rust MCP Atlassian Stage 2 migration. The MCP control plane is initialized in {access_mode} mode. Jira core tools are available when Jira configuration and authentication are complete; Confluence tools are not migrated yet."
            ))
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, net::SocketAddr, sync::Arc};

    use crate::{
        atlassian::auth::AtlassianAuth,
        config::{HttpConfig, RuntimeConfig},
        context::AppContext,
        jira::config::{JiraConfig, JiraDeployment},
        jira::tools,
        tool_registry::{MIGRATION_STATUS_TOOL_NAME, ToolAccess, ToolMetadata, ToolService},
    };
    use axum::{
        Json, Router,
        body::Bytes,
        extract::State,
        http::{HeaderMap, Method, StatusCode},
        response::{IntoResponse, Response},
        routing::any,
    };
    use rmcp::model::{JsonObject, Tool};
    use rmcp::{ServerHandler, handler::server::wrapper::Parameters};
    use serde_json::json;
    use tokio::sync::Mutex;

    use super::*;

    fn server_with_config(config: RuntimeConfig) -> AtlassianMcpServer {
        AtlassianMcpServer::new(Arc::new(AppContext::from_config(&config)))
    }

    const SYNTHETIC_JIRA_READ: ToolMetadata = ToolMetadata {
        name: "stage1_synthetic_jira_read",
        service: ToolService::Jira,
        access: ToolAccess::Read,
        toolset: Some("jira_issues"),
        title: "Synthetic Jira read",
        description: "Test-only Jira read metadata.",
    };

    const SYNTHETIC_JIRA_WRITE: ToolMetadata = ToolMetadata {
        name: "stage1_synthetic_jira_write",
        service: ToolService::Jira,
        access: ToolAccess::Write,
        toolset: Some("jira_issues"),
        title: "Synthetic Jira write",
        description: "Test-only Jira write metadata.",
    };

    const SYNTHETIC_CONFLUENCE_READ: ToolMetadata = ToolMetadata {
        name: "stage1_synthetic_confluence_read",
        service: ToolService::Confluence,
        access: ToolAccess::Read,
        toolset: Some("confluence_pages"),
        title: "Synthetic Confluence read",
        description: "Test-only Confluence read metadata.",
    };

    fn metadata_for_test_tool(name: &str) -> Option<ToolMetadata> {
        match name {
            "stage1_synthetic_jira_read" => Some(SYNTHETIC_JIRA_READ),
            "stage1_synthetic_jira_write" => Some(SYNTHETIC_JIRA_WRITE),
            "stage1_synthetic_confluence_read" => Some(SYNTHETIC_CONFLUENCE_READ),
            _ => tool_registry::metadata_for(name),
        }
    }

    fn runtime_config() -> RuntimeConfig {
        RuntimeConfig {
            http: HttpConfig::default(),
            ..RuntimeConfig::default()
        }
    }

    fn jira_config() -> JiraConfig {
        jira_config_with_base_url("https://jira.example".to_string())
    }

    fn jira_config_with_base_url(base_url: String) -> JiraConfig {
        JiraConfig {
            base_url,
            deployment: JiraDeployment::ServerDataCenter,
            auth: AtlassianAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            },
            ssl_verify: true,
            projects_filter: BTreeSet::new(),
            timeout_seconds: 75,
        }
    }

    fn tool(name: &'static str) -> Tool {
        Tool::new(name, "", Arc::<JsonObject>::new(Default::default()))
    }

    fn current_tool_names(server: &AtlassianMcpServer) -> Vec<String> {
        tool_names(server.current_tools_result().tools)
    }

    fn tool_names(tools: Vec<Tool>) -> Vec<String> {
        tools
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect()
    }

    fn expected_stage_two_default_tools() -> Vec<String> {
        vec![
            tools::JIRA_ADD_COMMENT_TOOL_NAME.to_string(),
            tools::JIRA_EDIT_COMMENT_TOOL_NAME.to_string(),
            tools::JIRA_GET_FIELD_OPTIONS_TOOL_NAME.to_string(),
            tools::JIRA_GET_ISSUE_TOOL_NAME.to_string(),
            tools::JIRA_GET_PROJECT_ISSUES_TOOL_NAME.to_string(),
            tools::JIRA_GET_TRANSITIONS_TOOL_NAME.to_string(),
            tools::JIRA_SEARCH_TOOL_NAME.to_string(),
            tools::JIRA_SEARCH_FIELDS_TOOL_NAME.to_string(),
            tools::JIRA_TRANSITION_ISSUE_TOOL_NAME.to_string(),
            MIGRATION_STATUS_TOOL_NAME.to_string(),
        ]
    }

    #[derive(Clone, Debug)]
    struct RecordedRequest {
        method: Method,
        path: String,
    }

    #[derive(Clone)]
    struct MockJiraState {
        requests: Arc<Mutex<Vec<RecordedRequest>>>,
    }

    async fn mock_jira_handler(
        State(state): State<MockJiraState>,
        method: Method,
        headers: HeaderMap,
        uri: axum::http::Uri,
        body: Bytes,
    ) -> Response {
        let _ = body;
        let path = uri
            .path_and_query()
            .map(ToString::to_string)
            .unwrap_or_else(|| uri.path().to_string());
        state.requests.lock().await.push(RecordedRequest {
            method: method.clone(),
            path: path.clone(),
        });

        let expected_header = format!("Bearer {}", "test-pat-value");
        if headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            != Some(expected_header.as_str())
        {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"errorMessages": ["auth"]})),
            )
                .into_response();
        }

        if method == Method::GET && path.starts_with("/rest/api/2/issue/ABC-1") {
            return (
                StatusCode::OK,
                Json(json!({
                    "id": "10001",
                    "key": "ABC-1",
                    "fields": {"summary": "Mock issue"}
                })),
            )
                .into_response();
        }

        (
            StatusCode::NOT_FOUND,
            Json(json!({"errorMessages": ["missing"]})),
        )
            .into_response()
    }

    async fn mock_jira_server() -> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let app = Router::new()
            .fallback(any(mock_jira_handler))
            .with_state(MockJiraState {
                requests: requests.clone(),
            });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address: SocketAddr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://{address}"), requests)
    }

    #[test]
    fn server_info_advertises_tools() {
        let info = AtlassianMcpServer::default().get_info();

        assert_eq!(info.server_info.name, SERVER_NAME);
        assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));
        assert!(info.capabilities.tools.is_some());
        assert!(info.capabilities.prompts.is_none());
        assert!(info.capabilities.resources.is_none());
    }

    #[test]
    fn tool_metadata_is_generated() {
        assert_eq!(
            AtlassianMcpServer::migration_status_tool_attr().name,
            MIGRATION_STATUS_TOOL_NAME
        );
    }

    #[test]
    fn migration_status_reports_stage_scope() {
        let server = AtlassianMcpServer::default();
        let status = server.migration_status();

        assert!(status.contains("Stage 2 Jira core migration is complete"));
        assert!(status.contains("Jira config/auth/client/models/tool handlers are implemented"));
    }

    #[test]
    fn server_info_uses_app_context() {
        let config = RuntimeConfig {
            read_only: true,
            ..RuntimeConfig::default()
        };
        let server = AtlassianMcpServer::new(Arc::new(AppContext::from_config(&config)));
        let info = server.get_info();
        let instructions = info.instructions.unwrap_or_default();

        assert!(instructions.contains("read-only mode"));
        assert!(instructions.contains("Jira core tools are available"));
    }

    #[test]
    fn tool_discovery_uses_registry_and_keeps_migration_status_visible_by_default() {
        let server = AtlassianMcpServer::default();

        assert_eq!(
            current_tool_names(&server),
            vec![MIGRATION_STATUS_TOOL_NAME.to_string()]
        );
        assert!(server.get_tool(MIGRATION_STATUS_TOOL_NAME).is_some());
    }

    #[test]
    fn tool_discovery_lists_stage_two_jira_core_tools_when_configured() {
        let server = server_with_config(RuntimeConfig {
            jira: Some(jira_config()),
            ..runtime_config()
        });

        assert_eq!(
            current_tool_names(&server),
            expected_stage_two_default_tools()
        );
        assert!(server.get_tool(tools::JIRA_GET_ISSUE_TOOL_NAME).is_some());
        assert!(server.get_tool("jira_create_issue").is_none());
    }

    #[tokio::test]
    async fn jira_get_issue_handler_returns_structured_content_from_mock_rest() {
        let (base_url, requests) = mock_jira_server().await;
        let server = server_with_config(RuntimeConfig {
            jira: Some(jira_config_with_base_url(base_url)),
            ..runtime_config()
        });
        let result = server
            .jira_get_issue(Parameters(tools::JiraGetIssueArgs {
                issue_key: "ABC-1".to_string(),
                fields: Some(json!(["summary"])),
                expand: None,
                comment_limit: None,
                properties: None,
                update_history: None,
            }))
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(
            result.structured_content.as_ref().unwrap()["key"],
            json!("ABC-1")
        );
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, Method::GET);
        assert!(requests[0].path.starts_with("/rest/api/2/issue/ABC-1"));
    }

    #[tokio::test]
    async fn jira_tool_handler_rejects_invalid_json_object_input_before_http() {
        let (base_url, requests) = mock_jira_server().await;
        let server = server_with_config(RuntimeConfig {
            jira: Some(jira_config_with_base_url(base_url)),
            ..runtime_config()
        });
        let error = server
            .jira_transition_issue(Parameters(tools::JiraTransitionIssueArgs {
                issue_key: "ABC-1".to_string(),
                transition_id: "31".to_string(),
                fields: Some(json!("[]")),
                comment: None,
            }))
            .await
            .unwrap_err();
        let requests = requests.lock().await;

        assert!(error.message.contains("fields must be a JSON object"));
        assert!(requests.is_empty());
    }

    #[test]
    fn tool_discovery_applies_toolsets_and_read_only_to_real_jira_tools() {
        let fields_only = server_with_config(RuntimeConfig {
            jira: Some(jira_config()),
            enabled_toolsets: BTreeSet::from(["jira_fields".to_string()]),
            ..runtime_config()
        });
        let read_only = server_with_config(RuntimeConfig {
            read_only: true,
            jira: Some(jira_config()),
            ..runtime_config()
        });

        assert_eq!(
            current_tool_names(&fields_only),
            vec![
                tools::JIRA_GET_FIELD_OPTIONS_TOOL_NAME.to_string(),
                tools::JIRA_SEARCH_FIELDS_TOOL_NAME.to_string(),
                MIGRATION_STATUS_TOOL_NAME.to_string(),
            ]
        );
        assert!(
            !current_tool_names(&read_only)
                .contains(&tools::JIRA_ADD_COMMENT_TOOL_NAME.to_string())
        );
        assert!(
            read_only
                .guard_registered_tool_call(tools::JIRA_TRANSITION_ISSUE_TOOL_NAME)
                .is_err()
        );
    }

    #[test]
    fn tool_discovery_applies_enabled_tools_filter_to_migration_status() {
        let server = server_with_config(RuntimeConfig {
            enabled_tools: Some(BTreeSet::from(["some_other_tool".to_string()])),
            ..runtime_config()
        });

        assert!(current_tool_names(&server).is_empty());
        assert!(server.get_tool(MIGRATION_STATUS_TOOL_NAME).is_none());
        assert!(
            server
                .guard_registered_tool_call(MIGRATION_STATUS_TOOL_NAME)
                .is_err()
        );
    }

    #[test]
    fn tool_discovery_does_not_apply_toolsets_to_migration_status() {
        let server = server_with_config(RuntimeConfig {
            enabled_toolsets: BTreeSet::new(),
            ..runtime_config()
        });

        assert_eq!(
            current_tool_names(&server),
            vec![MIGRATION_STATUS_TOOL_NAME.to_string()]
        );
    }

    #[test]
    fn tool_discovery_fails_closed_for_unmapped_tools() {
        let server = AtlassianMcpServer::default();
        let tools =
            server.filtered_tools_from([tool(MIGRATION_STATUS_TOOL_NAME), tool("unmapped_tool")]);
        let names: Vec<_> = tools
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect();

        assert_eq!(names, vec![MIGRATION_STATUS_TOOL_NAME.to_string()]);
    }

    #[test]
    fn tool_discovery_applies_future_service_and_toolset_policy_at_server_boundary() {
        let unavailable = AtlassianMcpServer::default();
        let available = server_with_config(RuntimeConfig {
            jira: Some(jira_config()),
            confluence_url: Some("https://confluence.example".to_string()),
            ..runtime_config()
        });
        let jira_fields_only = server_with_config(RuntimeConfig {
            jira: Some(jira_config()),
            enabled_toolsets: BTreeSet::from(["jira_fields".to_string()]),
            ..runtime_config()
        });

        assert_eq!(
            tool_names(unavailable.filtered_tools_from_with_metadata(
                [
                    tool("stage1_synthetic_jira_read"),
                    tool("stage1_synthetic_confluence_read"),
                ],
                metadata_for_test_tool,
            )),
            Vec::<String>::new()
        );
        assert_eq!(
            tool_names(available.filtered_tools_from_with_metadata(
                [
                    tool("stage1_synthetic_jira_read"),
                    tool("stage1_synthetic_confluence_read"),
                ],
                metadata_for_test_tool,
            )),
            vec![
                "stage1_synthetic_confluence_read".to_string(),
                "stage1_synthetic_jira_read".to_string(),
            ]
        );
        assert!(
            jira_fields_only
                .filtered_tools_from_with_metadata(
                    [tool("stage1_synthetic_jira_read")],
                    metadata_for_test_tool,
                )
                .is_empty()
        );
    }

    #[test]
    fn direct_call_guard_applies_future_read_only_policy_at_server_boundary() {
        let read_only_server = server_with_config(RuntimeConfig {
            read_only: true,
            jira: Some(jira_config()),
            ..runtime_config()
        });
        let read_write_server = server_with_config(RuntimeConfig {
            jira: Some(jira_config()),
            ..runtime_config()
        });

        let error = read_only_server
            .guard_tool_call_with_metadata(
                "stage1_synthetic_jira_write",
                true,
                metadata_for_test_tool,
            )
            .unwrap_err();

        assert_eq!(error.message, "tool is disabled in read-only mode");
        assert!(
            read_write_server
                .guard_tool_call_with_metadata(
                    "stage1_synthetic_jira_write",
                    true,
                    metadata_for_test_tool,
                )
                .is_ok()
        );
        assert!(
            read_write_server
                .guard_tool_call_with_metadata(
                    "stage1_synthetic_jira_write",
                    false,
                    metadata_for_test_tool,
                )
                .is_err()
        );
    }

    #[tokio::test]
    async fn read_only_guard_blocks_real_jira_write_tool_before_http_request() {
        let (base_url, requests) = mock_jira_server().await;
        let read_only_server = server_with_config(RuntimeConfig {
            read_only: true,
            jira: Some(jira_config_with_base_url(base_url)),
            ..runtime_config()
        });
        let error = read_only_server
            .guard_registered_tool_call(tools::JIRA_ADD_COMMENT_TOOL_NAME)
            .unwrap_err();
        let requests = requests.lock().await;

        assert_eq!(error.message, "tool is disabled in read-only mode");
        assert!(requests.is_empty());
    }
}
