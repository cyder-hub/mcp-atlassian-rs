use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::State,
    http::{
        HeaderMap, HeaderValue, Method, StatusCode, Uri,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
    response::{IntoResponse, Response},
    routing::any,
};
use clap::{Args, ValueEnum};
use reqwest::header::HeaderMap as ReqwestHeaderMap;
use serde_json::{Value, json};
use tokio::{sync::Mutex, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::XtaskResult;

type EnvMap = BTreeMap<String, String>;

const JIRA_TOKEN: &str = "test-smoke-token";
const CONFLUENCE_TOKEN: &str = "test-confluence-smoke-token";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmokeCommand {
    service: SmokeService,
    mode: SmokeMode,
    port: Option<u16>,
    path: Option<String>,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct SmokeArgs {
    #[arg(value_enum, default_value_t = SmokeMode::All)]
    pub mode: SmokeMode,
    #[arg(long)]
    pub port: Option<u16>,
    #[arg(long)]
    pub path: Option<String>,
}

impl SmokeArgs {
    pub fn into_command(self, service: SmokeService) -> SmokeCommand {
        SmokeCommand {
            service,
            mode: self.mode,
            port: self.port,
            path: self.path,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmokeService {
    Jira,
    Confluence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SmokeMode {
    All,
    Stdio,
    Http,
    Restricted,
}

#[derive(Debug, Clone)]
struct RecordedRequest {
    method: Method,
    path: String,
}

struct MockState {
    service: SmokeService,
    requests: Mutex<Vec<RecordedRequest>>,
}

struct MockServer {
    url: String,
    state: std::sync::Arc<MockState>,
    shutdown: CancellationToken,
    task: JoinHandle<std::io::Result<()>>,
}

pub async fn run(command: SmokeCommand) -> XtaskResult<i32> {
    match run_inner(command).await {
        Ok(()) => Ok(0),
        Err(error) => {
            eprintln!("smoke failed: {error}");
            Ok(1)
        }
    }
}

pub async fn run_all() -> XtaskResult<i32> {
    let commands = [
        SmokeCommand {
            service: SmokeService::Jira,
            mode: SmokeMode::Stdio,
            port: None,
            path: None,
        },
        SmokeCommand {
            service: SmokeService::Jira,
            mode: SmokeMode::Http,
            port: None,
            path: None,
        },
        SmokeCommand {
            service: SmokeService::Jira,
            mode: SmokeMode::Restricted,
            port: None,
            path: None,
        },
        SmokeCommand {
            service: SmokeService::Confluence,
            mode: SmokeMode::All,
            port: None,
            path: None,
        },
    ];

    for command in commands {
        let exit_code = run(command).await?;
        if exit_code != 0 {
            return Ok(exit_code);
        }
    }

    Ok(0)
}

async fn run_inner(command: SmokeCommand) -> Result<(), String> {
    let binary = build_mcp_binary()?;
    let server = MockServer::start(command.service).await?;
    let url = server.url.clone();
    let result = async {
        match command.mode {
            SmokeMode::All => {
                run_stdio(command.service, &binary, &url).await?;
                run_http(
                    command.service,
                    &binary,
                    &url,
                    command.port,
                    command.path.as_deref(),
                )
                .await?;
                run_restricted(command.service, &binary, &url, &server).await
            }
            SmokeMode::Stdio => run_stdio(command.service, &binary, &url).await,
            SmokeMode::Http => {
                run_http(
                    command.service,
                    &binary,
                    &url,
                    command.port,
                    command.path.as_deref(),
                )
                .await
            }
            SmokeMode::Restricted => run_restricted(command.service, &binary, &url, &server).await,
        }
    }
    .await;
    server.shutdown().await;
    result
}

fn build_mcp_binary() -> Result<PathBuf, String> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "xtask manifest directory has no parent".to_string())?
        .to_path_buf();
    let status = Command::new("cargo")
        .args(["build", "--quiet", "--bin", "mcp-atlassian-rs"])
        .current_dir(&workspace_root)
        .status()
        .map_err(|error| format!("failed to run cargo build: {error}"))?;
    if !status.success() {
        return Err(format!("cargo build failed with status {status}"));
    }

    let binary_name = if cfg!(windows) {
        "mcp-atlassian-rs.exe"
    } else {
        "mcp-atlassian-rs"
    };
    Ok(workspace_root
        .join("target")
        .join("debug")
        .join(binary_name))
}

impl MockServer {
    async fn start(service: SmokeService) -> Result<Self, String> {
        let state = std::sync::Arc::new(MockState {
            service,
            requests: Mutex::new(Vec::new()),
        });
        let app = Router::new()
            .fallback(any(mock_handler))
            .with_state(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|error| format!("mock bind failed: {error}"))?;
        let address = listener
            .local_addr()
            .map_err(|error| format!("mock local_addr failed: {error}"))?;
        let shutdown = CancellationToken::new();
        let server_shutdown = shutdown.clone();
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(server_shutdown.cancelled_owned())
                .await
        });

        Ok(Self {
            url: format!("http://{address}"),
            state,
            shutdown,
            task,
        })
    }

    async fn requests(&self) -> Vec<RecordedRequest> {
        self.state.requests.lock().await.clone()
    }

    async fn shutdown(self) {
        self.shutdown.cancel();
        let _ = self.task.await;
    }
}

async fn mock_handler(
    State(state): State<std::sync::Arc<MockState>>,
    method: Method,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response {
    let path_and_query = uri
        .path_and_query()
        .map(ToString::to_string)
        .unwrap_or_else(|| uri.path().to_string());
    state.requests.lock().await.push(RecordedRequest {
        method: method.clone(),
        path: path_and_query,
    });

    if !authorized(state.service, &headers) {
        return json_response(
            StatusCode::UNAUTHORIZED,
            json!({"errorMessages": ["mock auth failed"]}),
        );
    }

    let body = match read_body(body).await {
        Ok(body) => body,
        Err(error) => return json_response(StatusCode::BAD_REQUEST, json!({"error": error})),
    };
    match state.service {
        SmokeService::Jira => mock_jira_response(method, uri.path(), body),
        SmokeService::Confluence => mock_confluence_response(method, uri.path(), body),
    }
}

fn authorized(service: SmokeService, headers: &HeaderMap) -> bool {
    let expected = match service {
        SmokeService::Jira => format!("Bearer {JIRA_TOKEN}"),
        SmokeService::Confluence => format!("Bearer {CONFLUENCE_TOKEN}"),
    };
    headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        == Some(expected.as_str())
}

async fn read_body(body: Body) -> Result<Option<Value>, String> {
    let bytes = to_bytes(body, 1024 * 1024)
        .await
        .map_err(|error| format!("body read failed: {error}"))?;
    if bytes.is_empty() {
        return Ok(None);
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|error| format!("invalid JSON body: {error}"))
}

fn mock_jira_response(method: Method, path: &str, body: Option<Value>) -> Response {
    match (method, path) {
        (Method::GET, "/rest/api/2/issue/ABC-1/transitions") => json_response(
            StatusCode::OK,
            json!({
                "transitions": [{
                    "id": "31",
                    "name": "Done",
                    "to": {"id": "3", "name": "Done"}
                }]
            }),
        ),
        (Method::GET, "/rest/api/2/issue/ABC-1") => json_response(
            StatusCode::OK,
            json!({
                "id": "10001",
                "key": "ABC-1",
                "fields": {
                    "summary": "Smoke issue",
                    "created": "2026-01-01T00:00:00.000+0000",
                    "updated": "2026-01-02T00:00:00.000+0000",
                    "duedate": "2026-01-10",
                    "resolutiondate": "2026-01-03T00:00:00.000+0000",
                    "status": {"id": "3", "name": "Done"},
                    "project": {"id": "10000", "key": "ABC", "name": "ABC"},
                    "customfield_sla": {
                        "name": "Time to resolution SLA",
                        "ongoingCycle": {
                            "breached": false,
                            "elapsedTime": {"millis": 60000},
                            "remainingTime": {"millis": 120000},
                            "startTime": "2026-01-01T00:00:00.000+0000"
                        }
                    }
                },
                "changelog": {
                    "histories": [{
                        "id": "h1",
                        "created": "2026-01-02T00:00:00.000+0000",
                        "items": [{
                            "field": "status",
                            "fieldId": "status",
                            "from": "2",
                            "fromString": "In Progress",
                            "to": "3",
                            "toString": "Done"
                        }]
                    }]
                }
            }),
        ),
        (Method::GET, "/rest/api/2/issue/ABC-1/worklog") => json_response(
            StatusCode::OK,
            json!({
                "startAt": 0,
                "maxResults": 1,
                "total": 1,
                "worklogs": [{
                    "id": "20001",
                    "timeSpent": "1h",
                    "author": {"displayName": "Smoke User"}
                }]
            }),
        ),
        (Method::GET, "/rest/agile/1.0/board") => json_response(
            StatusCode::OK,
            json!({
                "startAt": 0,
                "maxResults": 1,
                "total": 1,
                "values": [{
                    "id": 7,
                    "name": "Smoke board",
                    "type": "scrum",
                    "location": {"projectKey": "ABC"}
                }]
            }),
        ),
        (Method::GET, "/rest/api/2/field") => json_response(
            StatusCode::OK,
            json!([
                {"id": "summary", "name": "Summary"},
                {"id": "customfield_10001", "name": "Customer Impact"}
            ]),
        ),
        (Method::POST, "/rest/api/2/search") | (Method::POST, "/rest/api/3/search/jql") => {
            json_response(
                StatusCode::OK,
                json!({
                    "issues": [{
                        "id": "10001",
                        "key": "ABC-1",
                        "fields": {"summary": "Smoke issue"}
                    }],
                    "total": 1,
                    "startAt": 0,
                    "maxResults": 1
                }),
            )
        }
        (Method::POST, "/rest/api/2/issue/ABC-1/comment") => json_response(
            StatusCode::OK,
            json!({
                "id": "10",
                "body": body.and_then(|body| body.get("body").cloned()).unwrap_or(Value::Null),
                "author": {"displayName": "Smoke User"}
            }),
        ),
        (Method::POST, "/rest/api/2/issue/ABC-1/transitions") => {
            StatusCode::NO_CONTENT.into_response()
        }
        (Method::PUT, "/rest/api/2/issue/ABC-1/comment/10") => json_response(
            StatusCode::OK,
            json!({
                "id": "10",
                "body": body.and_then(|body| body.get("body").cloned()).unwrap_or(Value::Null),
                "author": {"displayName": "Smoke User"}
            }),
        ),
        _ => json_response(
            StatusCode::NOT_FOUND,
            json!({"errorMessages": ["mock path not found"]}),
        ),
    }
}

fn mock_confluence_response(method: Method, path: &str, _body: Option<Value>) -> Response {
    match (method, path) {
        (Method::GET, "/rest/api/content/search") => json_response(
            StatusCode::OK,
            json!({
                "results": [{
                    "id": "123",
                    "title": "Roadmap",
                    "type": "page",
                    "content": {
                        "id": "123",
                        "title": "Roadmap",
                        "type": "page",
                        "space": {"key": "ENG", "name": "Engineering"},
                        "_links": {"webui": "/spaces/ENG/pages/123/Roadmap"}
                    },
                    "space": {"key": "ENG", "name": "Engineering"},
                    "url": "/spaces/ENG/pages/123/Roadmap",
                    "excerpt": "Smoke page"
                }],
                "start": 0,
                "limit": 10,
                "size": 1,
                "_links": {}
            }),
        ),
        (Method::GET, "/rest/api/content/123") => json_response(
            StatusCode::OK,
            json!({
                "id": "123",
                "title": "Roadmap",
                "type": "page",
                "status": "current",
                "space": {"key": "ENG", "name": "Engineering"},
                "body": {
                    "storage": {
                        "value": "<h1>Roadmap</h1><p>Smoke page</p>",
                        "representation": "storage"
                    }
                },
                "version": {"number": 3},
                "ancestors": [{"id": "100", "title": "Home"}],
                "metadata": {"labels": {"results": [{"name": "smoke"}]}},
                "_links": {"webui": "/spaces/ENG/pages/123/Roadmap"}
            }),
        ),
        _ => json_response(
            StatusCode::NOT_FOUND,
            json!({"errorMessages": ["mock path not found"]}),
        ),
    }
}

fn json_response(status: StatusCode, payload: Value) -> Response {
    (status, Json(payload)).into_response()
}

async fn run_stdio(service: SmokeService, binary: &PathBuf, url: &str) -> Result<(), String> {
    let mut session = StdioSession::start(binary, smoke_env(service, url, false))?;
    session.initialize(service.stdio_client_name())?;
    let names = session.list_tools()?;
    assert_required_tools(service, &names, false, "stdio")?;

    match service {
        SmokeService::Jira => {
            assert_jira_issue_result(&session.call_tool(
                3,
                "jira_get_issue",
                json!({"issue_key": "ABC-1", "fields": ["summary", "status"]}),
            )?)?;
            assert_worklog_result(&session.call_tool(
                4,
                "jira_get_worklog",
                json!({"issue_key": "ABC-1", "limit": 1}),
            )?)?;
            assert_agile_boards_result(&session.call_tool(
                5,
                "jira_get_agile_boards",
                json!({"project_key": "ABC", "board_type": "scrum", "limit": 1}),
            )?)?;
            assert_issue_dates_result(&session.call_tool(
                6,
                "jira_get_issue_dates",
                json!({
                    "issue_key": "ABC-1",
                    "include_status_changes": true,
                    "include_status_summary": true
                }),
            )?)?;
            assert_issue_sla_result(&session.call_tool(
                7,
                "jira_get_issue_sla",
                json!({"issue_key": "ABC-1", "include_raw_dates": true}),
            )?)?;
            println!(
                "stdio smoke passed: representative Jira tools, status_summary, and SLA parsing_limitations are discoverable and callable with mock Jira"
            );
        }
        SmokeService::Confluence => {
            assert_search_result(&session.call_tool(
                3,
                "confluence_search",
                json!({"query": "project docs", "limit": 10, "spaces_filter": "ENG"}),
            )?)?;
            assert_page_result(&session.call_tool(
                4,
                "confluence_get_page",
                json!({"page_id": "123", "include_metadata": true, "convert_to_markdown": true}),
            )?)?;
            println!(
                "Confluence stdio smoke passed: search and get_page work with mock Confluence"
            );
        }
    }
    Ok(())
}

async fn run_http(
    service: SmokeService,
    binary: &PathBuf,
    url: &str,
    requested_port: Option<u16>,
    requested_path: Option<&str>,
) -> Result<(), String> {
    let port = match requested_port {
        Some(port) => port,
        None => free_port()?,
    };
    let path = normalize_path(requested_path.unwrap_or(service.default_path()));
    let mut child = Command::new(binary)
        .arg("streamhttp")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .arg("--path")
        .arg(&path)
        .env_clear()
        .envs(smoke_env(service, url, false))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to start HTTP server: {error}"))?;

    let result = run_http_inner(service, port, &path).await;
    let _ = child.kill();
    let _ = child.wait();
    result
}

async fn run_http_inner(service: SmokeService, port: u16, path: &str) -> Result<(), String> {
    wait_health(port).await?;
    let client = reqwest::Client::new();
    let (headers, body) = post_mcp(
        &client,
        port,
        path,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {"name": service.http_client_name(), "version": "0.1.0"}
            }
        }),
        None,
    )
    .await?;
    let init = expect_rpc(&body, 1)?;
    if init.get("result").is_none() {
        return Err(format!("HTTP initialize failed: {init}"));
    }
    let session_id = headers
        .get("mcp-session-id")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| "Mcp-Session-Id header missing".to_string())?
        .to_string();
    let _ = post_mcp(
        &client,
        port,
        path,
        json!({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}),
        Some(&session_id),
    )
    .await?;

    let (_, body) = post_mcp(
        &client,
        port,
        path,
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
        Some(&session_id),
    )
    .await?;
    let tools_message = expect_rpc(&body, 2)?;
    let names = tool_names(&tools_message);
    assert_required_tools(service, &names, false, "HTTP")?;

    match service {
        SmokeService::Jira => {
            assert_jira_issue_result(
                &http_call(
                    &client,
                    port,
                    path,
                    &session_id,
                    3,
                    "jira_get_issue",
                    json!({"issue_key": "ABC-1", "fields": ["summary", "status"]}),
                )
                .await?,
            )?;
            assert_worklog_result(
                &http_call(
                    &client,
                    port,
                    path,
                    &session_id,
                    4,
                    "jira_get_worklog",
                    json!({"issue_key": "ABC-1", "limit": 1}),
                )
                .await?,
            )?;
            assert_agile_boards_result(
                &http_call(
                    &client,
                    port,
                    path,
                    &session_id,
                    5,
                    "jira_get_agile_boards",
                    json!({"project_key": "ABC", "board_type": "scrum", "limit": 1}),
                )
                .await?,
            )?;
            assert_issue_dates_result(
                &http_call(
                    &client,
                    port,
                    path,
                    &session_id,
                    6,
                    "jira_get_issue_dates",
                    json!({
                        "issue_key": "ABC-1",
                        "include_status_changes": true,
                        "include_status_summary": true
                    }),
                )
                .await?,
            )?;
            assert_issue_sla_result(
                &http_call(
                    &client,
                    port,
                    path,
                    &session_id,
                    7,
                    "jira_get_issue_sla",
                    json!({"issue_key": "ABC-1", "include_raw_dates": true}),
                )
                .await?,
            )?;
            println!(
                "HTTP smoke passed: /healthz ok and representative Jira tools, status_summary, and SLA parsing_limitations are callable with mock Jira"
            );
        }
        SmokeService::Confluence => {
            assert_search_result(
                &http_call(
                    &client,
                    port,
                    path,
                    &session_id,
                    3,
                    "confluence_search",
                    json!({"query": "project docs", "limit": 10, "spaces_filter": "ENG"}),
                )
                .await?,
            )?;
            assert_page_result(
                &http_call(
                    &client,
                    port,
                    path,
                    &session_id,
                    4,
                    "confluence_get_page",
                    json!({"page_id": "123", "include_metadata": true, "convert_to_markdown": true}),
                )
                .await?,
            )?;
            println!(
                "Confluence HTTP smoke passed: /healthz ok and search/get_page work with mock Confluence"
            );
        }
    }

    Ok(())
}

async fn run_restricted(
    service: SmokeService,
    binary: &PathBuf,
    url: &str,
    server: &MockServer,
) -> Result<(), String> {
    let mut session = StdioSession::start(binary, smoke_env(service, url, true))?;
    session.initialize(service.restricted_client_name())?;
    let names = session.list_tools()?;
    assert_required_tools(service, &names, true, "restricted")?;

    match service {
        SmokeService::Jira => {
            let response = session.call_tool(
                3,
                "jira_create_issue",
                json!({
                    "project_key": "ABC",
                    "summary": "blocked by restricted smoke",
                    "issue_type": "Task"
                }),
            )?;
            assert_restricted_error(&response, "jira_create_issue")?;
            assert_no_request(server, Method::POST, "/rest/api/2/issue").await?;
            println!(
                "Jira restricted smoke passed: selected reads stay visible and jira_create_issue is blocked before HTTP"
            );
        }
        SmokeService::Confluence => {
            let response = session.call_tool(
                3,
                "confluence_create_page",
                json!({
                    "space_key": "ENG",
                    "title": "Blocked smoke page",
                    "content": "blocked by restricted smoke",
                    "content_format": "markdown"
                }),
            )?;
            assert_restricted_error(&response, "confluence_create_page")?;
            assert_no_request(server, Method::POST, "/rest/api/content").await?;
            println!(
                "Confluence restricted smoke passed: reads stay visible and confluence_create_page is blocked before HTTP"
            );
        }
    }

    Ok(())
}

fn smoke_env(service: SmokeService, url: &str, restricted: bool) -> EnvMap {
    let mut env = EnvMap::new();
    match service {
        SmokeService::Jira => {
            env.insert("JIRA_URL".to_string(), url.to_string());
            env.insert("JIRA_PERSONAL_TOKEN".to_string(), JIRA_TOKEN.to_string());
            env.insert("JIRA_SSL_VERIFY".to_string(), "false".to_string());
            env.insert(
                "TOOLSETS".to_string(),
                "jira_worklog,jira_agile_read,jira_metrics_read".to_string(),
            );
            if restricted {
                env.insert(
                    "DISABLED_TOOLS".to_string(),
                    "jira_create_issue".to_string(),
                );
            }
            env.insert(
                "ATLASSIAN_OAUTH_CLOUD_ID".to_string(),
                "cloud-smoke".to_string(),
            );
        }
        SmokeService::Confluence => {
            env.insert("CONFLUENCE_URL".to_string(), url.to_string());
            env.insert(
                "CONFLUENCE_PERSONAL_TOKEN".to_string(),
                CONFLUENCE_TOKEN.to_string(),
            );
            env.insert("CONFLUENCE_SSL_VERIFY".to_string(), "false".to_string());
            env.insert(
                "TOOLSETS".to_string(),
                "confluence_content_write".to_string(),
            );
            if restricted {
                env.insert(
                    "DISABLED_TOOLS".to_string(),
                    "confluence_create_page".to_string(),
                );
            }
        }
    }
    env
}

fn assert_required_tools(
    service: SmokeService,
    names: &[String],
    restricted: bool,
    transport: &str,
) -> Result<(), String> {
    let required = match service {
        SmokeService::Jira => vec![
            "jira_get_issue",
            "jira_get_worklog",
            "jira_get_agile_boards",
            "jira_get_issue_dates",
            "jira_get_issue_sla",
        ],
        SmokeService::Confluence => vec!["confluence_search", "confluence_get_page"],
    };
    let missing = required
        .into_iter()
        .filter(|tool| !names.iter().any(|name| name == tool))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "{transport} tools/list missing {missing:?}: {names:?}"
        ));
    }

    let write_tool = match service {
        SmokeService::Jira => "jira_create_issue",
        SmokeService::Confluence => "confluence_create_page",
    };
    let has_write_tool = names.iter().any(|name| name == write_tool);
    if restricted && has_write_tool {
        return Err(format!(
            "{write_tool} should be hidden in restricted smoke mode: {names:?}"
        ));
    }
    if !restricted && !has_write_tool {
        return Err(format!(
            "{transport} tools/list missing write sentinel {write_tool}: {names:?}"
        ));
    }

    Ok(())
}

async fn assert_no_request(
    server: &MockServer,
    method: Method,
    path_prefix: &str,
) -> Result<(), String> {
    let requests = server.requests().await;
    let blocked = requests
        .iter()
        .filter(|request| {
            request.method == method && request.path.split('?').next() == Some(path_prefix)
        })
        .collect::<Vec<_>>();
    if blocked.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "restricted write tool reached mock service: {blocked:?}"
        ))
    }
}

fn assert_jira_issue_result(response: &Value) -> Result<(), String> {
    let structured = structured(response, "jira_get_issue")?;
    if structured.get("key").and_then(Value::as_str) == Some("ABC-1") {
        Ok(())
    } else {
        Err(format!(
            "jira_get_issue did not return mock issue: {response}"
        ))
    }
}

fn assert_worklog_result(response: &Value) -> Result<(), String> {
    let structured = structured(response, "jira_get_worklog")?;
    let id = structured
        .get("worklogs")
        .and_then(Value::as_array)
        .and_then(|worklogs| worklogs.first())
        .and_then(|worklog| worklog.get("id"))
        .and_then(Value::as_str);
    if id == Some("20001") {
        Ok(())
    } else {
        Err(format!(
            "jira_get_worklog did not return mock worklog: {response}"
        ))
    }
}

fn assert_agile_boards_result(response: &Value) -> Result<(), String> {
    let structured = structured(response, "jira_get_agile_boards")?;
    let name = structured
        .get("values")
        .and_then(Value::as_array)
        .and_then(|boards| boards.first())
        .and_then(|board| board.get("name"))
        .and_then(Value::as_str);
    if name == Some("Smoke board") {
        Ok(())
    } else {
        Err(format!(
            "jira_get_agile_boards did not return mock board: {response}"
        ))
    }
}

fn assert_issue_dates_result(response: &Value) -> Result<(), String> {
    let structured = structured(response, "jira_get_issue_dates")?;
    let summary = structured
        .get("status_summary")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("jira_get_issue_dates did not return status_summary: {response}"))?;
    let transition_count = summary.get("transition_count").and_then(Value::as_u64);
    let status_name = summary
        .get("current_status")
        .and_then(Value::as_object)
        .and_then(|status| status.get("name"))
        .and_then(Value::as_str);
    if transition_count == Some(1) && status_name == Some("Done") {
        Ok(())
    } else {
        Err(format!(
            "jira_get_issue_dates did not return current status summary: {response}"
        ))
    }
}

fn assert_issue_sla_result(response: &Value) -> Result<(), String> {
    let structured = structured(response, "jira_get_issue_sla")?;
    let limitation = structured
        .get("parsing_limitations")
        .and_then(Value::as_object)
        .and_then(|limitations| limitations.get("working_hours_filtering"))
        .and_then(Value::as_str);
    if limitation == Some("not_supported") {
        Ok(())
    } else {
        Err(format!(
            "jira_get_issue_sla did not return parsing_limitations: {response}"
        ))
    }
}

fn assert_search_result(response: &Value) -> Result<(), String> {
    let structured = structured(response, "confluence_search")?;
    let title = structured
        .get("results")
        .and_then(Value::as_array)
        .and_then(|results| results.first())
        .and_then(|result| result.get("title"))
        .and_then(Value::as_str);
    if title == Some("Roadmap") {
        Ok(())
    } else {
        Err(format!(
            "confluence_search did not return mock page: {response}"
        ))
    }
}

fn assert_page_result(response: &Value) -> Result<(), String> {
    let structured = structured(response, "confluence_get_page")?;
    let metadata = structured
        .get("metadata")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("confluence_get_page missing metadata: {response}"))?;
    let id = metadata.get("id").and_then(Value::as_str);
    let title = metadata.get("title").and_then(Value::as_str);
    if id == Some("123") && title == Some("Roadmap") {
        Ok(())
    } else {
        Err(format!(
            "confluence_get_page did not return mock page: {response}"
        ))
    }
}

fn assert_restricted_error(response: &Value, tool: &str) -> Result<(), String> {
    let text = serde_json::to_string(response).unwrap_or_default();
    if text.contains("tool not available") {
        Ok(())
    } else {
        Err(format!(
            "{tool} was not blocked by restricted tool config: {response}"
        ))
    }
}

fn structured<'a>(
    response: &'a Value,
    tool_name: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    if let Some(error) = response.get("error") {
        return Err(format!("{tool_name} returned JSON-RPC error: {error}"));
    }
    let result = response.get("result").unwrap_or(&Value::Null);
    if result.get("isError").and_then(Value::as_bool) == Some(true) {
        return Err(format!("{tool_name} returned tool error: {response}"));
    }
    result
        .get("structuredContent")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{tool_name} returned no structuredContent: {response}"))
}

struct StdioSession {
    child: Child,
    stdin: std::process::ChildStdin,
    rx: mpsc::Receiver<Result<String, String>>,
}

impl StdioSession {
    fn start(binary: &PathBuf, env: EnvMap) -> Result<Self, String> {
        let mut child = Command::new(binary)
            .arg("stdio")
            .env_clear()
            .envs(env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("failed to start stdio server: {error}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "failed to open stdio server stdin".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "failed to open stdio server stdout".to_string())?;
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                if tx
                    .send(line.map_err(|error| format!("stdio read error: {error}")))
                    .is_err()
                {
                    break;
                }
            }
        });

        Ok(Self { child, stdin, rx })
    }

    fn initialize(&mut self, client_name: &str) -> Result<(), String> {
        self.send(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {"name": client_name, "version": "0.1.0"},
            }
        }))?;
        let response = self.read_response(1)?;
        if response.get("result").is_none() {
            return Err(format!("initialize failed: {response}"));
        }
        self.send(json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        }))
    }

    fn list_tools(&mut self) -> Result<Vec<String>, String> {
        self.send(json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }))?;
        let response = self.read_response(2)?;
        Ok(tool_names(&response))
    }

    fn call_tool(
        &mut self,
        request_id: i64,
        name: &str,
        arguments: Value,
    ) -> Result<Value, String> {
        self.send(json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments,
            }
        }))?;
        self.read_response(request_id)
    }

    fn send(&mut self, message: Value) -> Result<(), String> {
        let encoded = serde_json::to_string(&message).map_err(|error| error.to_string())?;
        writeln!(self.stdin, "{encoded}").map_err(|error| format!("stdio write error: {error}"))?;
        self.stdin
            .flush()
            .map_err(|error| format!("stdio flush error: {error}"))
    }

    fn read_response(&mut self, expected_id: i64) -> Result<Value, String> {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            let now = Instant::now();
            if now >= deadline {
                return Err(format!(
                    "timed out waiting for JSON-RPC response id {expected_id}"
                ));
            }
            let line = self
                .rx
                .recv_timeout(deadline.saturating_duration_since(now))
                .map_err(|_| format!("timed out waiting for JSON-RPC response id {expected_id}"))?
                .map_err(|error| error.to_string())?;
            let message: Value =
                serde_json::from_str(&line).map_err(|error| format!("invalid JSON: {error}"))?;
            if message.get("id").and_then(Value::as_i64) == Some(expected_id) {
                return Ok(message);
            }
        }
    }
}

impl Drop for StdioSession {
    fn drop(&mut self) {
        let _ = self.stdin.flush();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

async fn http_call(
    client: &reqwest::Client,
    port: u16,
    path: &str,
    session_id: &str,
    request_id: i64,
    name: &str,
    arguments: Value,
) -> Result<Value, String> {
    let (_, body) = post_mcp(
        client,
        port,
        path,
        json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments,
            }
        }),
        Some(session_id),
    )
    .await?;
    expect_rpc(&body, request_id)
}

async fn wait_health(port: u16) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}/healthz");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if Instant::now() >= deadline {
            return Err("health endpoint did not become ready".to_string());
        }
        if let Ok(response) = client.get(&url).send().await
            && let Ok(payload) = response.json::<Value>().await
            && payload == json!({"status": "ok"})
        {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn post_mcp(
    client: &reqwest::Client,
    port: u16,
    path: &str,
    message: Value,
    session_id: Option<&str>,
) -> Result<(ReqwestHeaderMap, String), String> {
    let url = format!("http://127.0.0.1:{port}{path}");
    let mut request = client
        .post(url)
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .header(
            ACCEPT,
            HeaderValue::from_static("application/json, text/event-stream"),
        )
        .body(serde_json::to_vec(&message).map_err(|error| error.to_string())?);
    if let Some(session_id) = session_id {
        request = request.header("Mcp-Session-Id", session_id);
    }
    let response = request
        .send()
        .await
        .map_err(|error| format!("MCP HTTP request failed: {error}"))?;
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .text()
        .await
        .map_err(|error| format!("MCP HTTP response read failed: {error}"))?;
    if status.is_client_error() || status.is_server_error() {
        return Err(format!("MCP HTTP {status}: {body}"));
    }
    Ok((headers, body))
}

fn expect_rpc(body: &str, expected_id: i64) -> Result<Value, String> {
    for message in parse_sse_messages(body)? {
        if message.get("id").and_then(Value::as_i64) == Some(expected_id) {
            if message.get("result").is_none() && message.get("error").is_none() {
                return Err(format!(
                    "expected JSON-RPC result id {expected_id}: {message}"
                ));
            }
            return Ok(message);
        }
    }
    Err(format!("missing JSON-RPC response id {expected_id}"))
}

fn parse_sse_messages(body: &str) -> Result<Vec<Value>, String> {
    let stripped = body.trim_start();
    if stripped.starts_with('{') {
        return serde_json::from_str(stripped)
            .map(|message| vec![message])
            .map_err(|error| format!("invalid JSON response: {error}"));
    }

    let mut messages = Vec::new();
    for line in body.lines() {
        let Some(payload) = line.strip_prefix("data:") else {
            continue;
        };
        let payload = payload.trim();
        if payload.is_empty() {
            continue;
        }
        messages.push(
            serde_json::from_str(payload).map_err(|error| format!("invalid SSE JSON: {error}"))?,
        );
    }
    Ok(messages)
}

fn tool_names(response: &Value) -> Vec<String> {
    response
        .get("result")
        .and_then(|result| result.get("tools"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|tool| tool.get("name").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect()
}

fn free_port() -> Result<u16, String> {
    let listener =
        TcpListener::bind("127.0.0.1:0").map_err(|error| format!("bind failed: {error}"))?;
    listener
        .local_addr()
        .map(|address| address.port())
        .map_err(|error| format!("local_addr failed: {error}"))
}

fn normalize_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return "/mcp".to_string();
    }
    if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    }
}

impl SmokeService {
    fn default_path(self) -> &'static str {
        "/mcp"
    }

    fn stdio_client_name(self) -> &'static str {
        match self {
            Self::Jira => "jira-stdio-smoke",
            Self::Confluence => "confluence-stdio-smoke",
        }
    }

    fn http_client_name(self) -> &'static str {
        match self {
            Self::Jira => "jira-http-smoke",
            Self::Confluence => "confluence-http-smoke",
        }
    }

    fn restricted_client_name(self) -> &'static str {
        match self {
            Self::Jira => "jira-restricted-smoke",
            Self::Confluence => "confluence-restricted-smoke",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_smoke_command_from_clap_args() {
        let args = SmokeArgs {
            mode: SmokeMode::Http,
            port: Some(9000),
            path: Some("mcp".to_string()),
        };
        assert_eq!(
            args.into_command(SmokeService::Jira),
            SmokeCommand {
                service: SmokeService::Jira,
                mode: SmokeMode::Http,
                port: Some(9000),
                path: Some("mcp".to_string()),
            }
        );
    }

    #[test]
    fn normalizes_paths() {
        assert_eq!(normalize_path("mcp"), "/mcp");
        assert_eq!(normalize_path("/custom-mcp"), "/custom-mcp");
        assert_eq!(normalize_path(""), "/mcp");
    }
}
