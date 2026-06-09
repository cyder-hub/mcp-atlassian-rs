use super::*;

pub(super) fn server_with_config(config: RuntimeConfig) -> AtlassianMcpServer {
    AtlassianMcpServer::new(Arc::new(AppContext::from_config(&config)))
}

pub(super) const SYNTHETIC_JIRA_READ: ToolMetadata = ToolMetadata {
    name: "synthetic_jira_read",
    service: ToolService::Jira,
    access: ToolAccess::Read,
    toolset: Some("jira_issue_read"),
    title: "Synthetic Jira read",
    description: "Test-only Jira read metadata.",
};

pub(super) const SYNTHETIC_JIRA_WRITE: ToolMetadata = ToolMetadata {
    name: "synthetic_jira_write",
    service: ToolService::Jira,
    access: ToolAccess::Write,
    toolset: Some("jira_issue_write"),
    title: "Synthetic Jira write",
    description: "Test-only Jira write metadata.",
};

pub(super) const SYNTHETIC_CONFLUENCE_READ: ToolMetadata = ToolMetadata {
    name: "synthetic_confluence_read",
    service: ToolService::Confluence,
    access: ToolAccess::Read,
    toolset: Some("confluence_content_read"),
    title: "Synthetic Confluence read",
    description: "Test-only Confluence read metadata.",
};

pub(super) fn metadata_for_test_tool(name: &str) -> Option<ToolMetadata> {
    match name {
        "synthetic_jira_read" => Some(SYNTHETIC_JIRA_READ),
        "synthetic_jira_write" => Some(SYNTHETIC_JIRA_WRITE),
        "synthetic_confluence_read" => Some(SYNTHETIC_CONFLUENCE_READ),
        _ => tool_registry::metadata_for(name),
    }
}

pub(super) fn runtime_config() -> RuntimeConfig {
    RuntimeConfig {
        http: HttpConfig::default(),
        ..RuntimeConfig::default()
    }
}

pub(super) fn jira_config() -> JiraConfig {
    jira_config_with_base_url("https://jira.example".to_string())
}

pub(super) fn confluence_config() -> ConfluenceConfig {
    confluence_config_with_base_url("https://confluence.example".to_string())
}

pub(super) fn jira_cloud_config_with_base_url(base_url: String) -> JiraConfig {
    JiraConfig {
        base_url,
        deployment: JiraDeployment::Cloud,
        auth: AtlassianAuth::Basic {
            username: "test-user".to_string(),
            api_token: "test-api-token".to_string(),
        },
        oauth_cloud_id: None,
        ssl_verify: true,
        proxy: ProxyConfig::default(),
        custom_headers: CustomHeaders::default(),
        mtls: None,
        projects_filter: BTreeSet::new(),
        timeout_seconds: 75,
    }
}

pub(super) fn confluence_cloud_config_with_base_url(base_url: String) -> ConfluenceConfig {
    ConfluenceConfig {
        base_url,
        deployment: ConfluenceDeployment::Cloud,
        auth: AtlassianAuth::Basic {
            username: "test-user".to_string(),
            api_token: "test-api-token".to_string(),
        },
        oauth_cloud_id: None,
        ssl_verify: true,
        proxy: ProxyConfig::default(),
        custom_headers: CustomHeaders::default(),
        mtls: None,
        spaces_filter: BTreeSet::new(),
        timeout_seconds: 75,
    }
}

pub(super) fn confluence_config_with_base_url(base_url: String) -> ConfluenceConfig {
    ConfluenceConfig {
        base_url,
        deployment: ConfluenceDeployment::ServerDataCenter,
        auth: AtlassianAuth::Pat {
            personal_token: "test-pat-value".to_string(),
        },
        oauth_cloud_id: None,
        ssl_verify: true,
        proxy: ProxyConfig::default(),
        custom_headers: CustomHeaders::default(),
        mtls: None,
        spaces_filter: BTreeSet::new(),
        timeout_seconds: 75,
    }
}

pub(super) fn jira_config_with_base_url(base_url: String) -> JiraConfig {
    JiraConfig {
        base_url,
        deployment: JiraDeployment::ServerDataCenter,
        auth: AtlassianAuth::Pat {
            personal_token: "test-pat-value".to_string(),
        },
        oauth_cloud_id: None,
        ssl_verify: true,
        proxy: ProxyConfig::default(),
        custom_headers: CustomHeaders::default(),
        mtls: None,
        projects_filter: BTreeSet::new(),
        timeout_seconds: 75,
    }
}

pub(super) fn tool(name: &'static str) -> Tool {
    Tool::new(name, "", Arc::<JsonObject>::new(Default::default()))
}

pub(super) fn current_tool_names(server: &AtlassianMcpServer) -> Vec<String> {
    tool_names(server.current_tools_result().tools)
}

pub(super) fn tool_names(tools: Vec<Tool>) -> Vec<String> {
    tools
        .into_iter()
        .map(|tool| tool.name.to_string())
        .collect()
}

pub(super) fn header_map(headers: &[(&'static str, &'static str)]) -> HeaderMap {
    let mut header_map = HeaderMap::new();
    for (name, value) in headers {
        header_map.insert(*name, HeaderValue::from_static(value));
    }
    header_map
}

pub(super) fn request_service_headers() -> HeaderMap {
    header_map(&[
        ("X-Atlassian-Jira-Url", "https://8.8.8.8"),
        ("X-Atlassian-Jira-Personal-Token", "request-jira-token"),
        ("X-Atlassian-Confluence-Url", "https://8.8.4.4"),
        (
            "X-Atlassian-Confluence-Personal-Token",
            "request-confluence-token",
        ),
    ])
}

pub(super) fn query_value(path: &str, key: &str) -> Option<String> {
    let url = reqwest::Url::parse(&format!("http://example{path}")).unwrap();
    url.query_pairs()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.into_owned())
}

pub(super) fn assert_client_compatible_tool_schemas(tools: &[Tool]) {
    for tool in tools {
        let schema = Value::Object(tool.input_schema.as_ref().clone());
        assert_client_compatible_schema_value(&schema, &tool.name);
        assert_explicit_property_schemas(&schema, &tool.name);
    }
}

pub(super) fn assert_tool_schema_lacks_property(tools: &[Tool], tool_name: &str, property: &str) {
    let tool = tools
        .iter()
        .find(|tool| tool.name == tool_name)
        .unwrap_or_else(|| panic!("{tool_name} should be discoverable"));
    let properties = tool
        .input_schema
        .get("properties")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("{tool_name} should expose an object schema"));

    assert!(
        !properties.contains_key(property),
        "{tool_name} should not expose stale property {property}"
    );
}

pub(super) fn assert_client_compatible_schema_value(value: &Value, path: &str) {
    match value {
        Value::Bool(_) => panic!("{path} contains a boolean JSON schema"),
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                assert_client_compatible_schema_value(value, &format!("{path}[{index}]"));
            }
        }
        Value::Object(object) => {
            if object.get("default").is_some_and(Value::is_null) {
                panic!("{path} contains default: null");
            }
            if object
                .get("type")
                .and_then(Value::as_array)
                .is_some_and(|types| types.iter().any(|value| value.as_str() == Some("null")))
            {
                panic!("{path} contains nullable type array");
            }

            for (key, value) in object {
                if key == "additionalProperties" {
                    continue;
                }
                assert_client_compatible_schema_value(value, &format!("{path}.{key}"));
            }
        }
        Value::Null | Value::Number(_) | Value::String(_) => {}
    }
}

pub(super) fn assert_explicit_property_schemas(value: &Value, path: &str) {
    let Value::Object(object) = value else {
        return;
    };

    if let Some(properties) = object.get("properties").and_then(Value::as_object) {
        for (name, property_schema) in properties {
            let property_path = format!("{path}.properties.{name}");
            let Some(property) = property_schema.as_object() else {
                panic!("{property_path} is not an object schema");
            };
            let has_explicit_shape = [
                "type", "anyOf", "oneOf", "allOf", "$ref", "not", "const", "enum",
            ]
            .iter()
            .any(|key| property.contains_key(*key));
            assert!(has_explicit_shape, "{property_path} has no explicit shape");
        }
    }

    for (key, value) in object {
        if key == "additionalProperties" || key == "properties" {
            continue;
        }
        assert_explicit_property_schemas(value, &format!("{path}.{key}"));
    }
}

pub(super) fn jira_extension_candidate_tools() -> Vec<Tool> {
    tools::JIRA_EXTENSION_TOOL_NAMES
        .iter()
        .map(|&name| tool(name))
        .collect()
}

pub(super) fn jira_extension_write_tool_names() -> Vec<&'static str> {
    tools::JIRA_EXTENSION_TOOL_NAMES
        .iter()
        .copied()
        .filter(|name| {
            tool_registry::metadata_for(name)
                .is_some_and(|metadata| metadata.access == ToolAccess::Write)
        })
        .collect()
}

pub(super) fn jira_general_extension_tool_names() -> Vec<&'static str> {
    vec![
        tools::JIRA_GET_ALL_PROJECTS_TOOL_NAME,
        tools::JIRA_GET_PROJECT_VERSIONS_TOOL_NAME,
        tools::JIRA_GET_PROJECT_COMPONENTS_TOOL_NAME,
        tools::JIRA_CREATE_VERSION_TOOL_NAME,
        tools::JIRA_BATCH_CREATE_VERSIONS_TOOL_NAME,
        tools::JIRA_GET_USER_PROFILE_TOOL_NAME,
        tools::JIRA_GET_ISSUE_WATCHERS_TOOL_NAME,
        tools::JIRA_ADD_WATCHER_TOOL_NAME,
        tools::JIRA_REMOVE_WATCHER_TOOL_NAME,
        tools::JIRA_GET_WORKLOG_TOOL_NAME,
        tools::JIRA_ADD_WORKLOG_TOOL_NAME,
        tools::JIRA_GET_LINK_TYPES_TOOL_NAME,
        tools::JIRA_LINK_TO_EPIC_TOOL_NAME,
        tools::JIRA_CREATE_ISSUE_LINK_TOOL_NAME,
        tools::JIRA_CREATE_REMOTE_ISSUE_LINK_TOOL_NAME,
        tools::JIRA_REMOVE_ISSUE_LINK_TOOL_NAME,
        tools::JIRA_DOWNLOAD_ATTACHMENTS_TOOL_NAME,
        tools::JIRA_GET_ISSUE_IMAGES_TOOL_NAME,
    ]
}

pub(super) fn jira_general_extension_write_tool_names() -> Vec<&'static str> {
    jira_general_extension_tool_names()
        .into_iter()
        .filter(|name| {
            tool_registry::metadata_for(name)
                .is_some_and(|metadata| metadata.access == ToolAccess::Write)
        })
        .collect()
}

pub(super) fn jira_product_extension_tool_names() -> Vec<&'static str> {
    vec![
        tools::JIRA_GET_AGILE_BOARDS_TOOL_NAME,
        tools::JIRA_GET_BOARD_ISSUES_TOOL_NAME,
        tools::JIRA_GET_SPRINTS_FROM_BOARD_TOOL_NAME,
        tools::JIRA_GET_SPRINT_ISSUES_TOOL_NAME,
        tools::JIRA_CREATE_SPRINT_TOOL_NAME,
        tools::JIRA_UPDATE_SPRINT_TOOL_NAME,
        tools::JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME,
        tools::JIRA_GET_SERVICE_DESK_FOR_PROJECT_TOOL_NAME,
        tools::JIRA_GET_SERVICE_DESK_QUEUES_TOOL_NAME,
        tools::JIRA_GET_QUEUE_ISSUES_TOOL_NAME,
        tools::JIRA_GET_ISSUE_PROFORMA_FORMS_TOOL_NAME,
        tools::JIRA_GET_PROFORMA_FORM_DETAILS_TOOL_NAME,
        tools::JIRA_UPDATE_PROFORMA_FORM_ANSWERS_TOOL_NAME,
        tools::JIRA_GET_ISSUE_DATES_TOOL_NAME,
        tools::JIRA_GET_ISSUE_SLA_TOOL_NAME,
        tools::JIRA_GET_ISSUE_DEVELOPMENT_INFO_TOOL_NAME,
        tools::JIRA_GET_ISSUES_DEVELOPMENT_INFO_TOOL_NAME,
    ]
}

pub(super) fn jira_product_extension_write_tool_names() -> Vec<&'static str> {
    jira_product_extension_tool_names()
        .into_iter()
        .filter(|name| {
            tool_registry::metadata_for(name)
                .is_some_and(|metadata| metadata.access == ToolAccess::Write)
        })
        .collect()
}

pub(super) fn expected_jira_core_default_tools() -> Vec<String> {
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
    ]
}

pub(super) fn all_jira_tool_names() -> Vec<String> {
    let mut names = expected_jira_core_default_tools();
    names.extend(
        tools::JIRA_EXTENSION_TOOL_NAMES
            .iter()
            .map(|name| (*name).to_string()),
    );
    names
}

pub(super) fn all_confluence_tool_names() -> Vec<String> {
    confluence_tools::CONFLUENCE_TOOL_NAMES
        .iter()
        .map(|name| (*name).to_string())
        .collect()
}

pub(super) fn support_matrix_tool_names() -> BTreeSet<String> {
    include_str!("../../../docs/support-matrix.md")
        .lines()
        .filter_map(|line| line.strip_prefix("| `"))
        .filter_map(|line| line.split_once('`').map(|(name, _)| name.to_string()))
        .filter(|name| name.starts_with("jira_") || name.starts_with("confluence_"))
        .collect()
}

#[derive(Clone, Debug)]
pub(super) struct RecordedRequest {
    pub(super) method: Method,
    pub(super) path: String,
    pub(super) body: Value,
}

#[derive(Clone)]
pub(super) struct MockJiraState {
    pub(super) requests: Arc<Mutex<Vec<RecordedRequest>>>,
}

#[derive(Clone)]
pub(super) struct MockConfluenceState {
    pub(super) requests: Arc<Mutex<Vec<RecordedRequest>>>,
}

pub(super) async fn mock_jira_handler(
    State(state): State<MockJiraState>,
    method: Method,
    headers: HeaderMap,
    uri: axum::http::Uri,
    body: Bytes,
) -> Response {
    let parsed_body = if body.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&body)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&body).to_string()))
    };
    let path = uri
        .path_and_query()
        .map(ToString::to_string)
        .unwrap_or_else(|| uri.path().to_string());
    state.requests.lock().await.push(RecordedRequest {
        method: method.clone(),
        path: path.clone(),
        body: parsed_body.clone(),
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

    let path_only = uri.path();
    if method == Method::GET && path_only == "/secure/attachment/1/file.png" {
        return (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "image/png")],
            "image-bytes",
        )
            .into_response();
    }
    if method == Method::GET && path_only == "/secure/attachment/2/notes.txt" {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "errorMessages": [
                    "failed /secure/attachment/2/notes.txt?token=secret&client=abc"
                ]
            })),
        )
            .into_response();
    }

    if method == Method::GET
        && (path == "/rest/api/2/issue/ABC-1" || path.starts_with("/rest/api/2/issue/ABC-1?"))
    {
        return (
            StatusCode::OK,
            Json(json!({
                "id": "10001",
                "key": "ABC-1",
                "fields": {
                    "summary": "Mock issue",
                    "created": "2026-01-01T00:00:00.000+0000",
                    "updated": "2026-01-02T00:00:00.000+0000",
                    "duedate": "2026-01-10",
                    "resolutiondate": "2026-01-03T00:00:00.000+0000",
                    "status": {
                        "id": "3",
                        "name": "Done",
                        "statusCategory": {"name": "Done"}
                    },
                    "customfield_sla": {
                        "name": "Time to resolution SLA",
                        "ongoingCycle": {
                            "breached": false,
                            "elapsedTime": {"millis": 60000},
                            "remainingTime": {"millis": 120000},
                            "startTime": "2026-01-01T00:00:00.000+0000"
                        }
                    },
                    "attachment": [
                        {
                            "id": "1",
                            "filename": "file.png",
                            "mimeType": "image/png",
                            "size": 11,
                            "content": "/secure/attachment/1/file.png?token=secret"
                        },
                        {
                            "id": "2",
                            "filename": "notes.txt",
                            "mimeType": "text/plain",
                            "size": 42,
                            "content": "/secure/attachment/2/notes.txt?token=secret&client=abc"
                        }
                    ]
                },
                "changelog": {
                    "histories": [
                        {
                            "id": "h1",
                            "created": "2026-01-01T01:00:00.000+0000",
                            "items": [{
                                "field": "status",
                                "fieldId": "status",
                                "from": "1",
                                "fromString": "To Do",
                                "to": "2",
                                "toString": "In Progress"
                            }]
                        },
                        {
                            "id": "h2",
                            "created": "2026-01-02T01:00:00.000+0000",
                            "items": [{
                                "field": "status",
                                "fieldId": "status",
                                "from": "2",
                                "fromString": "In Progress",
                                "to": "3",
                                "toString": "Done"
                            }]
                        }
                    ]
                }
            })),
        )
            .into_response();
    }
    if method == Method::GET
        && (path == "/rest/api/2/issue/TXT-1" || path.starts_with("/rest/api/2/issue/TXT-1?"))
    {
        return (
            StatusCode::OK,
            Json(json!({
                "id": "20001",
                "key": "TXT-1",
                "fields": {
                    "summary": "Text only",
                    "attachment": [
                        {
                            "id": "2",
                            "filename": "notes.txt",
                            "mimeType": "text/plain",
                            "size": 42,
                            "content": "/secure/attachment/2/notes.txt?token=secret&client=abc"
                        }
                    ]
                }
            })),
        )
            .into_response();
    }
    if method == Method::GET && path == "/rest/api/2/issue/ABC-1/watchers" {
        return (
            StatusCode::OK,
            Json(json!({
                "watchCount": 1,
                "isWatching": false,
                "watchers": [
                    {"accountId": "account-1", "displayName": "Ada Lovelace", "active": true}
                ]
            })),
        )
            .into_response();
    }
    if method == Method::POST && path == "/rest/api/2/issue/ABC-1/watchers" {
        return StatusCode::NO_CONTENT.into_response();
    }
    if method == Method::DELETE && path == "/rest/api/2/issue/ABC-1/watchers?username=ada" {
        return StatusCode::NO_CONTENT.into_response();
    }
    if method == Method::GET && path == "/rest/api/2/issue/ABC-1/worklog?startAt=0&maxResults=10" {
        return (
            StatusCode::OK,
            Json(json!({
                "startAt": 0,
                "maxResults": 10,
                "total": 2,
                "worklogs": [
                    {
                        "id": "100",
                        "timeSpent": "1h",
                        "started": "2026-01-01T00:00:00.000+0000",
                        "author": {"displayName": "Ada Lovelace"}
                    },
                    {
                        "id": "101",
                        "timeSpent": "30m"
                    }
                ]
            })),
        )
            .into_response();
    }
    if method == Method::POST
        && path == "/rest/api/2/issue/ABC-1/worklog?adjustEstimate=new&newEstimate=2h"
    {
        return (
            StatusCode::OK,
            Json(json!({
                "id": "300",
                "timeSpent": parsed_body["timeSpent"],
                "started": parsed_body["started"]
            })),
        )
            .into_response();
    }
    if method == Method::PUT && path.starts_with("/rest/api/2/issue/ABC-1") {
        return StatusCode::NO_CONTENT.into_response();
    }
    if method == Method::DELETE && path.starts_with("/rest/api/2/issue/ABC-1") {
        return StatusCode::NO_CONTENT.into_response();
    }
    if method == Method::POST && path == "/rest/api/2/issue" {
        return (
            StatusCode::OK,
            Json(json!({
                "id": "10002",
                "key": "ABC-2",
                "fields": {
                    "summary": "Created issue",
                    "project": {"key": "ABC", "name": "Demo"},
                    "issuetype": {"name": "Task"}
                }
            })),
        )
            .into_response();
    }
    if method == Method::POST && path == "/rest/api/2/issue/bulk" {
        return (
                StatusCode::OK,
                Json(json!({
                    "issues": [{"id": "10003", "key": "ABC-3", "self": "https://jira.example/rest/api/2/issue/10003"}],
                    "errors": [{"failedElementNumber": 1, "message": "validation failed"}]
                })),
            )
                .into_response();
    }
    if method == Method::POST && path == "/rest/api/3/changelog/bulkfetch" {
        return (
                StatusCode::OK,
                Json(json!({
                    "issueChangeLogs": [
                        {
                            "issueId": "10001",
                            "changeHistories": [
                                {
                                    "id": "20001",
                                    "items": [{"field": "status", "fromString": "Open", "toString": "Done"}]
                                }
                            ]
                        }
                    ],
                    "nextPageToken": "next-token"
                })),
            )
                .into_response();
    }
    if method == Method::GET && path == "/rest/api/2/project?includeArchived=false" {
        return (
            StatusCode::OK,
            Json(json!([
                {"id": "10000", "key": "ABC", "name": "Allowed"},
                {"id": "10001", "key": "XYZ", "name": "Filtered"}
            ])),
        )
            .into_response();
    }
    if method == Method::GET && path == "/rest/api/2/project/ABC/versions" {
        return (
            StatusCode::OK,
            Json(json!([
                {"id": "1", "name": "v1"},
                {"name": "unnumbered"}
            ])),
        )
            .into_response();
    }
    if method == Method::GET && path == "/rest/api/2/project/ABC/components" {
        return (
            StatusCode::OK,
            Json(json!([
                {"id": "10", "name": "API"},
                {}
            ])),
        )
            .into_response();
    }
    if method == Method::POST && path == "/rest/api/2/version" {
        if parsed_body["name"] == json!("bad") {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"errorMessages": ["bad version"]})),
            )
                .into_response();
        }
        return (
            StatusCode::OK,
            Json(json!({
                "id": "20000",
                "name": parsed_body["name"],
                "project": parsed_body["project"],
                "released": parsed_body.get("released").cloned().unwrap_or(Value::Bool(false))
            })),
        )
            .into_response();
    }
    if method == Method::GET && path == "/rest/api/2/user?username=ada" {
        return (
            StatusCode::OK,
            Json(json!({
                "accountId": "account-1",
                "name": "ada",
                "displayName": "Ada Lovelace",
                "active": true
            })),
        )
            .into_response();
    }
    if method == Method::GET && path == "/rest/api/2/user?accountId=account-1" {
        return (
            StatusCode::OK,
            Json(json!({
                "accountId": "account-1",
                "displayName": "Ada Lovelace",
                "active": true
            })),
        )
            .into_response();
    }
    if method == Method::GET && path == "/rest/api/2/issueLinkType" {
        return (
            StatusCode::OK,
            Json(json!({
                "issueLinkTypes": [
                    {
                        "id": "10000",
                        "name": "Blocks",
                        "inward": "is blocked by",
                        "outward": "blocks"
                    },
                    {
                        "id": "10001",
                        "name": "Relates"
                    }
                ]
            })),
        )
            .into_response();
    }
    if method == Method::POST && path == "/rest/api/2/issueLink" {
        return (
            StatusCode::CREATED,
            Json(json!({"id": "200", "type": parsed_body["type"]})),
        )
            .into_response();
    }
    if method == Method::POST && path == "/rest/api/2/issue/ABC-1/remotelink" {
        return (
            StatusCode::CREATED,
            Json(json!({"id": "300", "object": parsed_body["object"]})),
        )
            .into_response();
    }
    if method == Method::DELETE && path == "/rest/api/2/issueLink/200" {
        return StatusCode::NO_CONTENT.into_response();
    }
    if method == Method::GET
        && path.starts_with("/rest/agile/1.0/board?")
        && path.contains("projectKeyOrId=NOAGILE")
    {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"errorMessages": ["Jira Software is not available"]})),
        )
            .into_response();
    }
    if method == Method::GET && path.starts_with("/rest/agile/1.0/board?") {
        return (
            StatusCode::OK,
            Json(json!({
                "startAt": 0,
                "maxResults": 2,
                "total": 1,
                "isLast": true,
                "values": [
                    {"id": 1, "name": "Alpha board", "type": "scrum"}
                ]
            })),
        )
            .into_response();
    }
    if method == Method::GET && path.starts_with("/rest/agile/1.0/board/1/issue?") {
        return (
            StatusCode::OK,
            Json(json!({
                "startAt": 0,
                "maxResults": 2,
                "total": 1,
                "issues": [
                    {"id": "10001", "key": "ABC-1", "fields": {"summary": "Sprint issue"}}
                ]
            })),
        )
            .into_response();
    }
    if method == Method::GET && path.starts_with("/rest/agile/1.0/board/1/sprint?") {
        return (
            StatusCode::OK,
            Json(json!({
                "startAt": 0,
                "maxResults": 2,
                "total": 1,
                "isLast": true,
                "values": [
                    {"id": 2, "name": "Sprint 2", "state": "active"}
                ]
            })),
        )
            .into_response();
    }
    if method == Method::GET && path.starts_with("/rest/agile/1.0/sprint/2/issue?") {
        return (
            StatusCode::OK,
            Json(json!({
                "startAt": 0,
                "maxResults": 2,
                "total": 1,
                "issues": [
                    {"id": "10001", "key": "ABC-1", "fields": {"summary": "Sprint issue"}}
                ]
            })),
        )
            .into_response();
    }
    if method == Method::POST && path == "/rest/agile/1.0/sprint" {
        return (
            StatusCode::OK,
            Json(json!({
                "id": 2,
                "name": parsed_body["name"],
                "originBoardId": parsed_body["originBoardId"],
                "state": "future"
            })),
        )
            .into_response();
    }
    if method == Method::PUT && path == "/rest/agile/1.0/sprint/2" {
        return (
            StatusCode::OK,
            Json(json!({
                "id": 2,
                "name": parsed_body["name"],
                "state": parsed_body["state"],
                "goal": parsed_body["goal"]
            })),
        )
            .into_response();
    }
    if method == Method::POST && path == "/rest/agile/1.0/sprint/2/issue" {
        return StatusCode::NO_CONTENT.into_response();
    }
    if method == Method::GET && path_only.starts_with("/jsm-down/rest/servicedeskapi") {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"errorMessages": ["Jira Service Management is not available"]})),
        )
            .into_response();
    }
    if method == Method::GET && path == "/rest/servicedeskapi/servicedesk" {
        return (
            StatusCode::OK,
            Json(json!({
                "size": 2,
                "values": [
                    {"id": "4", "projectKey": "ABC", "serviceDeskName": "Support"},
                    {"id": "5", "projectKey": "XYZ", "serviceDeskName": "Other"}
                ]
            })),
        )
            .into_response();
    }
    if method == Method::GET && path == "/rest/servicedeskapi/servicedesk/4/queue?start=0&limit=50"
    {
        return (
            StatusCode::OK,
            Json(json!({
                "start": 0,
                "limit": 50,
                "size": 1,
                "values": [
                    {"id": "47", "name": "Open requests"}
                ]
            })),
        )
            .into_response();
    }
    if method == Method::GET
        && path == "/rest/servicedeskapi/servicedesk/4/queue/47/issue?start=0&limit=2"
    {
        return (
            StatusCode::OK,
            Json(json!({
                "start": 0,
                "limit": 2,
                "size": 1,
                "values": [
                    {"id": "10001", "key": "ABC-1", "fields": {"summary": "Customer request"}}
                ]
            })),
        )
            .into_response();
    }
    if method == Method::GET && path == "/jira/forms/cloud/cloud-123/issue/ABC-1/form" {
        return (
            StatusCode::OK,
            Json(json!({
                "forms": [
                    {
                        "id": "form-1",
                        "name": "Request form",
                        "state": {"status": "o"},
                        "submitted": false
                    }
                ]
            })),
        )
            .into_response();
    }
    if method == Method::GET && path == "/jira/forms/cloud/cloud-123/issue/ABC-1/form/form-1" {
        return (
            StatusCode::OK,
            Json(json!({
                "id": "form-1",
                "name": "Request form",
                "state": {"status": "o"},
                "design": {"content": []},
                "answers": {"q1": {"text": "Existing"}}
            })),
        )
            .into_response();
    }
    if method == Method::PUT && path == "/jira/forms/cloud/cloud-123/issue/ABC-1/form/form-1" {
        return (
            StatusCode::OK,
            Json(json!({
                "id": "form-1",
                "updated": true,
                "answers": parsed_body["answers"]
            })),
        )
            .into_response();
    }
    if method == Method::GET && path_only.starts_with("/jira/forms/cloud/forms-down/") {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"errorMessages": ["Jira Forms is not available"]})),
        )
            .into_response();
    }
    if method == Method::GET && path_only.starts_with("/dev-down/rest/dev-status") {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"errorMessages": ["Jira development status is not available"]})),
        )
            .into_response();
    }
    if method == Method::GET && path.starts_with("/rest/dev-status/1.0/issue/detail?") {
        return (
            StatusCode::OK,
            Json(json!({
                "detail": [
                    {
                        "applicationType": "github",
                        "dataType": "pullrequest",
                        "branches": [{"name": "main"}],
                        "pullRequests": [{"id": "pr-1", "name": "Fix bug"}],
                        "commits": [{"id": "commit-1", "displayId": "abc123"}]
                    }
                ]
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

pub(super) async fn mock_confluence_handler(
    State(state): State<MockConfluenceState>,
    method: Method,
    headers: HeaderMap,
    uri: axum::http::Uri,
    body: Bytes,
) -> Response {
    let parsed_body = if body.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&body)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&body).to_string()))
    };
    let path = uri
        .path_and_query()
        .map(ToString::to_string)
        .unwrap_or_else(|| uri.path().to_string());
    state.requests.lock().await.push(RecordedRequest {
        method: method.clone(),
        path: path.clone(),
        body: parsed_body.clone(),
    });

    let authorization = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok());
    let expected_pat_header = format!("Bearer {}", "test-pat-value");
    if authorization != Some(expected_pat_header.as_str())
        && !authorization.is_some_and(|value| value.starts_with("Basic "))
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"errorMessages": ["auth"]})),
        )
            .into_response();
    }

    let path_only = uri.path();
    if method == Method::GET && path_only == "/rest/api/content/123" {
        if let Some(version) = query_value(&path, "version") {
            let (title, storage_value) = match version.as_str() {
                "1" => ("Roadmap", "<h1>Roadmap</h1><p>Hello team</p>"),
                "2" => ("Roadmap", "<h1>Roadmap</h1><p>Hello team and partners</p>"),
                _ => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(json!({"errorMessages": ["historical version not found"]})),
                    )
                        .into_response();
                }
            };
            return (
                StatusCode::OK,
                Json(json!({
                    "id": "123",
                    "title": title,
                    "type": "page",
                    "status": "historical",
                    "space": {"key": "ENG", "name": "Engineering"},
                    "body": {"storage": {"value": storage_value}},
                    "version": {"number": version.parse::<u64>().unwrap()},
                    "ancestors": [{"id": "100", "title": "Home"}],
                    "_links": {"webui": "/spaces/ENG/pages/123/Roadmap"}
                })),
            )
                .into_response();
        }

        return (
            StatusCode::OK,
            Json(json!({
                "id": "123",
                "title": "Roadmap",
                "type": "page",
                "status": "current",
                "space": {"key": "ENG", "name": "Engineering"},
                "body": {"storage": {"value": "<h1>Roadmap</h1><p>Hello &amp; welcome</p>"}},
                "version": {"number": 7, "message": "Updated"},
                "ancestors": [{"id": "100", "title": "Home"}],
                "metadata": {"labels": {"results": [{"name": "planning"}]}},
                "_links": {"webui": "/spaces/ENG/pages/123/Roadmap"}
            })),
        )
            .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/123/child/comment" {
        return (
            StatusCode::OK,
            Json(json!({
                "results": [
                    {
                        "id": "c-1",
                        "title": "Roadmap",
                        "type": "comment",
                        "body": {"storage": {"value": "<p>First comment</p>"}},
                        "version": {"number": 2, "by": {"displayName": "Ada"}},
                        "container": {"id": "123", "type": "page", "title": "Roadmap"},
                        "extensions": {"location": "footer"},
                        "_links": {"webui": "/spaces/ENG/pages/123?focusedCommentId=c-1"}
                    },
                    {
                        "id": "c-2",
                        "type": "comment",
                        "body": {"storage": {"value": "<p>Reply</p>"}},
                        "version": {"number": 1, "by": {"displayName": "Lin"}},
                        "container": {"id": "c-1", "type": "comment", "title": "Roadmap"}
                    }
                ],
                "start": 0,
                "limit": 25,
                "size": 2,
                "_links": {}
            })),
        )
            .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/empty/child/comment" {
        return (
            StatusCode::OK,
            Json(json!({
                "results": [],
                "start": 0,
                "limit": 25,
                "size": 0,
                "_links": {}
            })),
        )
            .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/123/label" {
        return (
                StatusCode::OK,
                Json(json!({
                    "results": [
                        {"id": "label-1", "name": "draft", "prefix": "global", "label": "draft", "type": "label"},
                        {"id": "label-2", "name": "team", "prefix": "my", "label": "team", "type": "label"}
                    ],
                    "start": 0,
                    "limit": 200,
                    "size": 2,
                    "_links": {}
                })),
            )
                .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/empty-labels/label" {
        return (
            StatusCode::OK,
            Json(json!({
                "results": [],
                "start": 0,
                "limit": 200,
                "size": 0,
                "_links": {}
            })),
        )
            .into_response();
    }
    if method == Method::POST && path_only == "/rest/api/content/123/label" {
        return StatusCode::NO_CONTENT.into_response();
    }
    if method == Method::POST && path_only == "/rest/api/content/label-error/label" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"errorMessages": ["label failed"]})),
        )
            .into_response();
    }
    if method == Method::POST && path_only == "/rest/api/content" {
        if parsed_body["type"] == json!("comment") {
            let container_id = parsed_body["container"]["id"].as_str().unwrap_or("");
            if container_id == "comment-error" || container_id == "reply-error" {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"errorMessages": ["comment failed"]})),
                )
                    .into_response();
            }
            let is_reply = parsed_body["container"]["type"] == json!("comment");
            let comment_id = if is_reply { "c-2" } else { "c-1" };
            let display_name = if is_reply { "Lin" } else { "Ada" };
            return (
                StatusCode::OK,
                Json(json!({
                    "id": comment_id,
                    "title": "Roadmap",
                    "type": "comment",
                    "body": parsed_body["body"],
                    "version": {"number": 1, "by": {"displayName": display_name}},
                    "container": parsed_body["container"],
                    "extensions": {"location": "footer"},
                    "_links": {"webui": "/spaces/ENG/pages/123?focusedCommentId=c-1"}
                })),
            )
                .into_response();
        }

        return (
            StatusCode::OK,
            Json(json!({
                "id": "900",
                "title": parsed_body["title"],
                "type": "page",
                "status": "current",
                "space": parsed_body["space"],
                "body": parsed_body["body"],
                "version": {"number": 1},
                "ancestors": parsed_body.get("ancestors").cloned().unwrap_or(Value::Array(vec![]))
            })),
        )
            .into_response();
    }
    if method == Method::PUT
        && (path_only == "/rest/api/content/900/property/emoji-title-published"
            || path_only == "/rest/api/content/123/property/emoji-title-published")
    {
        if parsed_body["value"] == json!("fail") {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"errorMessages": ["emoji failed token=secret"]})),
            )
                .into_response();
        }
        return (
            StatusCode::OK,
            Json(json!({
                "key": "emoji-title-published",
                "value": parsed_body["value"]
            })),
        )
            .into_response();
    }
    if method == Method::PUT && path_only == "/rest/api/content/123" {
        return (
            StatusCode::OK,
            Json(json!({
                "id": "123",
                "title": parsed_body["title"],
                "type": "page",
                "status": "current",
                "space": parsed_body["space"],
                "body": parsed_body["body"],
                "version": parsed_body["version"],
                "ancestors": parsed_body.get("ancestors").cloned().unwrap_or(Value::Array(vec![]))
            })),
        )
            .into_response();
    }
    if method == Method::PUT && path_only == "/rest/api/content/123/move/above/999" {
        return StatusCode::NO_CONTENT.into_response();
    }
    if method == Method::DELETE && path_only == "/rest/api/content/123" {
        return StatusCode::NO_CONTENT.into_response();
    }
    if method == Method::DELETE && path_only == "/rest/api/content/delete-error" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"errorMessages": ["delete failed"]})),
        )
            .into_response();
    }
    if method == Method::PUT && path_only == "/rest/api/content/123/child/attachment" {
        if headers
            .get("x-atlassian-token")
            .and_then(|value| value.to_str().ok())
            != Some("nocheck")
        {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"errorMessages": ["missing attachment upload token"]})),
            )
                .into_response();
        }

        let body_text = parsed_body.as_str().unwrap_or("");
        let title = if body_text.contains("batch-1.txt") {
            "batch-1.txt"
        } else if body_text.contains("upload.txt") {
            "upload.txt"
        } else {
            "uploaded.bin"
        };
        return (
            StatusCode::OK,
            Json(json!({
                "results": [{
                    "id": format!("uploaded-{title}"),
                    "type": "attachment",
                    "title": title,
                    "status": "current",
                    "extensions": {"mediaType": "application/octet-stream", "fileSize": 5},
                    "_links": {"download": format!("/download/attachments/uploaded/{title}")}
                }]
            })),
        )
            .into_response();
    }
    if method == Method::PUT && path_only == "/rest/api/content/upload-error/child/attachment" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"errorMessages": ["upload failed"]})),
        )
            .into_response();
    }
    if method == Method::DELETE && path_only == "/rest/api/content/att-1" {
        return StatusCode::NO_CONTENT.into_response();
    }
    if method == Method::DELETE && path_only == "/rest/api/content/att-delete-error" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"errorMessages": ["delete attachment failed"]})),
        )
            .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/missing" {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"errorMessages": ["page not found"]})),
        )
            .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/att-1" {
        return (
            StatusCode::OK,
            Json(json!({
                "id": "att-1",
                "type": "attachment",
                "title": "file.png",
                "status": "current",
                "extensions": {"mediaType": "image/png", "fileSize": 11},
                "_links": {"download": "/download/attachments/att-1/file.png?token=secret"}
            })),
        )
            .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/att-no-url" {
        return (
            StatusCode::OK,
            Json(json!({
                "id": "att-no-url",
                "type": "attachment",
                "title": "missing.bin",
                "extensions": {"mediaType": "application/octet-stream", "fileSize": 12}
            })),
        )
            .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/att-large" {
        return (
            StatusCode::OK,
            Json(json!({
                "id": "att-large",
                "type": "attachment",
                "title": "large.bin",
                "extensions": {
                    "mediaType": "application/octet-stream",
                    "fileSize": crate::confluence::client::DEFAULT_ATTACHMENT_MAX_BYTES + 1
                },
                "_links": {"download": "/download/attachments/att-large/large.bin"}
            })),
        )
            .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/att-stream-large" {
        return (
            StatusCode::OK,
            Json(json!({
                "id": "att-stream-large",
                "type": "attachment",
                "title": "large-stream.bin",
                "extensions": {"mediaType": "application/octet-stream"},
                "_links": {"download": "/download/attachments/att-stream-large/large.bin"}
            })),
        )
            .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/att-cross" {
        return (
            StatusCode::OK,
            Json(json!({
                "id": "att-cross",
                "type": "attachment",
                "title": "cross.png",
                "extensions": {"mediaType": "image/png", "fileSize": 11},
                "_links": {"download": "https://other.example/download/cross.png?token=secret"}
            })),
        )
            .into_response();
    }
    if method == Method::GET && path_only == "/download/attachments/att-1/file.png" {
        return Bytes::from_static(b"image-bytes").into_response();
    }
    if method == Method::GET && path_only == "/download/attachments/att-page-1/file-1.txt" {
        return Bytes::from_static(b"page-one").into_response();
    }
    if method == Method::GET && path_only == "/download/attachments/att-page-2/file-2.txt" {
        return Bytes::from_static(b"page-two").into_response();
    }
    if method == Method::GET && path_only == "/download/attachments/att-octet-image/photo.jpg" {
        return Bytes::from_static(b"photo-bytes").into_response();
    }
    if method == Method::GET && path_only == "/download/attachments/att-stream-large/large.bin" {
        let bytes =
            vec![b'x'; crate::confluence::client::DEFAULT_ATTACHMENT_MAX_BYTES as usize + 1];
        return bytes.into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/123/child/page" {
        return (
            StatusCode::OK,
            Json(json!({
                "results": [{
                    "id": "201",
                    "title": "Child page",
                    "type": "page",
                    "status": "current",
                    "space": {"key": "ENG", "name": "Engineering"},
                    "body": {"storage": {"value": "<p>Child body</p>"}},
                    "version": {"number": 1}
                }],
                "start": 0,
                "limit": 2,
                "size": 1
            })),
        )
            .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/123/child/folder" {
        return (
            StatusCode::OK,
            Json(json!({
                "results": [{
                    "id": "301",
                    "title": "Folder",
                    "type": "folder",
                    "status": "current",
                    "space": {"key": "ENG", "name": "Engineering"}
                }],
                "start": 0,
                "limit": 2,
                "size": 1
            })),
        )
            .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/123/child/attachment" {
        return (
                StatusCode::OK,
                Json(json!({
                    "results": [
                        {
                            "id": "att-1",
                            "type": "attachment",
                            "title": "file.png",
                            "status": "current",
                            "extensions": {"mediaType": "image/png", "fileSize": 42},
                            "_links": {"download": "/download/attachments/att-1/file.png"}
                        },
                        {
                            "id": "att-2",
                            "type": "attachment",
                            "title": "notes.txt",
                            "metadata": {"mediaType": "text/plain", "fileSize": 12},
                            "_links": {"download": "/download/attachments/att-2/notes.txt"}
                        }
                    ],
                    "start": query_value(&path, "start").and_then(|value| value.parse::<u64>().ok()).unwrap_or(0),
                    "limit": query_value(&path, "limit").and_then(|value| value.parse::<u64>().ok()).unwrap_or(50),
                    "size": 2,
                    "_links": {"next": "/rest/api/content/123/child/attachment?start=2"}
                })),
            )
                .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/empty-attachments/child/attachment"
    {
        return (
            StatusCode::OK,
            Json(json!({
                "results": [],
                "start": 0,
                "limit": 50,
                "size": 0,
                "_links": {}
            })),
        )
            .into_response();
    }
    if method == Method::GET
        && path_only == "/rest/api/content/missing-attachment-fields/child/attachment"
    {
        return (
            StatusCode::OK,
            Json(json!({
                "results": [{"id": "att-min"}],
                "start": 0,
                "limit": 50,
                "size": 1,
                "_links": {}
            })),
        )
            .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/download-batch/child/attachment" {
        return (
            StatusCode::OK,
            Json(json!({
                "results": [
                    {
                        "id": "att-1",
                        "type": "attachment",
                        "title": "file.png",
                        "extensions": {"mediaType": "image/png", "fileSize": 11},
                        "_links": {"download": "/download/attachments/att-1/file.png?token=secret"}
                    },
                    {
                        "id": "att-no-url",
                        "type": "attachment",
                        "title": "missing.bin",
                        "extensions": {"mediaType": "application/octet-stream", "fileSize": 12}
                    },
                    {
                        "id": "att-large",
                        "type": "attachment",
                        "title": "large.bin",
                        "extensions": {
                            "mediaType": "application/octet-stream",
                            "fileSize": crate::confluence::client::DEFAULT_ATTACHMENT_MAX_BYTES + 1
                        },
                        "_links": {"download": "/download/attachments/att-large/large.bin"}
                    }
                ],
                "start": 0,
                "limit": 100,
                "size": 3,
                "_links": {}
            })),
        )
            .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/download-paged/child/attachment" {
        let start = query_value(&path, "start")
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(0);
        let (attachment_id, title, links) = if start == 0 {
            (
                "att-page-1",
                "file-1.txt",
                json!({"next": "/rest/api/content/download-paged/child/attachment?start=1"}),
            )
        } else {
            ("att-page-2", "file-2.txt", json!({}))
        };
        return (
            StatusCode::OK,
            Json(json!({
                "results": [{
                    "id": attachment_id,
                    "type": "attachment",
                    "title": title,
                    "extensions": {"mediaType": "text/plain", "fileSize": 8},
                    "_links": {"download": format!("/download/attachments/{attachment_id}/{title}")}
                }],
                "start": start,
                "limit": 100,
                "size": 1,
                "_links": links
            })),
        )
            .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/download-capped/child/attachment" {
        let start = query_value(&path, "start")
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(0);
        return (
                StatusCode::OK,
                Json(json!({
                    "results": [{
                        "id": format!("att-capped-{start}"),
                        "type": "attachment",
                        "title": format!("capped-{start}.bin"),
                        "extensions": {"mediaType": "application/octet-stream", "fileSize": 1}
                    }],
                    "start": start,
                    "limit": 100,
                    "size": 1,
                    "_links": {"next": format!("/rest/api/content/download-capped/child/attachment?start={}", start + 1)}
                })),
            )
                .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content/images/child/attachment" {
        return (
            StatusCode::OK,
            Json(json!({
                "results": [
                    {
                        "id": "att-1",
                        "type": "attachment",
                        "title": "file.png",
                        "extensions": {"mediaType": "image/png", "fileSize": 11},
                        "_links": {"download": "/download/attachments/att-1/file.png?token=secret"}
                    },
                    {
                        "id": "att-octet-image",
                        "type": "attachment",
                        "title": "photo.jpg",
                        "extensions": {"mediaType": "application/octet-stream", "fileSize": 11},
                        "_links": {"download": "/download/attachments/att-octet-image/photo.jpg"}
                    },
                    {
                        "id": "att-2",
                        "type": "attachment",
                        "title": "notes.txt",
                        "metadata": {"mediaType": "text/plain", "fileSize": 12},
                        "_links": {"download": "/download/attachments/att-2/notes.txt"}
                    }
                ],
                "start": 0,
                "limit": 100,
                "size": 3,
                "_links": {}
            })),
        )
            .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/content" {
        if query_value(&path, "type").as_deref() == Some("page") {
            let limit = query_value(&path, "limit");
            if limit.as_deref() == Some("1") {
                return (
                    StatusCode::OK,
                    Json(json!({
                        "results": [{
                            "id": "100",
                            "title": "Home",
                            "type": "page",
                            "ancestors": [],
                            "extensions": {"position": 0}
                        }],
                        "start": 0,
                        "limit": 1,
                        "size": 1,
                        "_links": {"next": "/rest/api/content?start=1"}
                    })),
                )
                    .into_response();
            }

            return (
                StatusCode::OK,
                Json(json!({
                    "results": [
                        {
                            "id": "200",
                            "title": "Child",
                            "type": "page",
                            "ancestors": [{"id": "100", "title": "Home"}],
                            "extensions": {"position": 1}
                        },
                        {
                            "id": "100",
                            "title": "Home",
                            "type": "page",
                            "ancestors": [],
                            "extensions": {"position": 0}
                        }
                    ],
                    "start": 0,
                    "limit": 2,
                    "size": 2,
                    "_links": {}
                })),
            )
                .into_response();
        }

        let title = query_value(&path, "title");
        let space_key = query_value(&path, "spaceKey");
        if title.as_deref() == Some("Roadmap") && space_key.as_deref() == Some("ENG") {
            return (
                StatusCode::OK,
                Json(json!({
                    "results": [{
                        "id": "123",
                        "title": "Roadmap",
                        "type": "page",
                        "status": "current",
                        "space": {"key": "ENG", "name": "Engineering"},
                        "body": {"storage": {"value": "<p>Raw storage</p>"}},
                        "version": {"number": 7}
                    }],
                    "start": 0,
                    "limit": 1,
                    "size": 1
                })),
            )
                .into_response();
        }

        return (
            StatusCode::OK,
            Json(json!({"results": [], "start": 0, "limit": 1, "size": 0})),
        )
            .into_response();
    }

    if method == Method::GET && path.starts_with("/rest/api/content/search?") {
        return (
            StatusCode::OK,
            Json(json!({
                "results": [
                    {
                        "id": "123",
                        "title": "Roadmap",
                        "excerpt": "<p>Planning</p>",
                        "content": {
                            "id": "123",
                            "title": "Roadmap",
                            "type": "page",
                            "space": {"key": "ENG", "name": "Engineering"}
                        },
                        "space": {"key": "ENG", "name": "Engineering"}
                    }
                ],
                "start": 0,
                "limit": 10,
                "size": 1
            })),
        )
            .into_response();
    }
    if method == Method::GET && path.starts_with("/rest/api/search/user?") {
        return (
            StatusCode::OK,
            Json(json!({
                "results": [{
                    "title": "Ada Lovelace",
                    "entityType": "user",
                    "score": 0.9,
                    "user": {
                        "accountId": "abc",
                        "displayName": "Ada Lovelace",
                        "email": "ada@example.com",
                        "accountStatus": "active",
                        "profilePicture": {"path": "/avatar/ada.png"}
                    }
                }],
                "start": 0,
                "limit": 5,
                "totalSize": 1,
                "cqlQuery": query_value(&path, "cql").unwrap_or_default(),
                "searchDuration": 7
            })),
        )
            .into_response();
    }
    if method == Method::GET
        && (path_only == "/rest/api/group/confluence-users/member"
            || path_only == "/rest/api/group/confluence%20users/member")
    {
        return (
                StatusCode::OK,
                Json(json!({
                    "results": [
                        {"username": "ada", "displayName": "Ada Lovelace", "email": "ada@example.com"},
                        {"username": "grace", "displayName": "Grace Hopper", "email": "grace@example.com"}
                    ],
                    "start": 0,
                    "limit": 200,
                    "size": 2,
                    "_links": {}
                })),
            )
                .into_response();
    }
    if method == Method::GET && path_only == "/rest/api/analytics/content/123/views" {
        return (
            StatusCode::OK,
            Json(json!({
                "count": 42,
                "lastSeen": "2026-06-04T12:00:00Z",
                "uniqueViewers": 7
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

pub(super) async fn mock_jira_server() -> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
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

pub(super) async fn mock_confluence_server() -> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let app = Router::new()
        .fallback(any(mock_confluence_handler))
        .with_state(MockConfluenceState {
            requests: requests.clone(),
        });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://{address}"), requests)
}

pub(super) fn temp_confluence_upload_file(filename: &str, content: &[u8]) -> String {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("mcp-atlassian-rs-{nonce}"));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(filename);
    std::fs::write(&path, content).unwrap();
    path.to_string_lossy().into_owned()
}

pub(super) fn oversized_temp_confluence_upload_file(filename: &str) -> String {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("mcp-atlassian-rs-oversized-{nonce}"));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(filename);
    let file = std::fs::File::create(&path).unwrap();
    file.set_len(crate::confluence::client::DEFAULT_UPLOAD_ATTACHMENT_MAX_BYTES + 1)
        .unwrap();
    path.to_string_lossy().into_owned()
}

pub(super) fn remove_temp_confluence_upload_file(file_path: &str) {
    let path = std::path::Path::new(file_path);
    let parent = path.parent().map(ToOwned::to_owned);
    let _ = std::fs::remove_file(path);
    if let Some(parent) = parent {
        let _ = std::fs::remove_dir(parent);
    }
}
