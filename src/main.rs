mod acceptance;
mod atlassian;
mod config;
mod confluence;
mod context;
mod error;
mod jira;
mod mcp;
mod mcp_confluence_helpers;
mod mcp_errors;
mod smoke;
mod tool_registry;

use std::{net::SocketAddr, sync::Arc};

use atlassian::redaction::redact_text;
use axum::{
    Json, Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use config::HttpConfigOverrides;
use context::AppContext;
use mcp::{AtlassianMcpServer, RequestAuthSessionStore};
use rmcp::{
    ServiceExt,
    transport::{
        stdio,
        streamable_http_server::{
            StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
        },
    },
};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::EnvFilter;

type AppResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const ENV_RUST_LOG: &str = "RUST_LOG";
const ENV_TOOL_CALL_DEBUG: &str = "MCP_TOOL_CALL_DEBUG";
const DEFAULT_RUST_LOG: &str = "info";
const TOOL_CALL_DEBUG_RUST_LOG: &str =
    "mcp_atlassian_rs::mcp=debug,mcp_atlassian_rs=info,rmcp=info";

const USAGE: &str = "\
Usage:
  mcp-atlassian-rs [stdio]
  mcp-atlassian-rs streamhttp [--host <host>] [--port <port>] [--path <path>] [--env-file <path>]
  mcp-atlassian-rs acceptance <jira|confluence|mcp> (--preflight | --run <binary>) [--env-file <path>]
  mcp-atlassian-rs smoke <jira|confluence> [all|stdio|http|read-only] [--port <port>] [--path <path>]

Commands:
  stdio       Run the MCP server over standard input/output.
  streamhttp  Run the MCP server over streamable HTTP.
  acceptance  Run Stage 5 acceptance checks from the Rust binary.
  smoke       Run local smoke checks against Rust mock Atlassian services.

Options:
  --env-file <path>  Load environment variables from the specified file (streamhttp and acceptance only).
                     Alternatively, set the ENV_FILE environment variable.

Defaults:
  host  127.0.0.1
  port  8000
  path  /mcp
";

#[derive(Debug, Clone, PartialEq, Eq)]
enum RunMode {
    Stdio,
    StreamHttp(HttpConfigOverrides, Option<String>),
    Acceptance(acceptance::AcceptanceCommand),
    Smoke(smoke::SmokeCommand),
}

#[derive(Debug, serde::Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Clone)]
struct StreamHttpAuthState {
    context: Arc<AppContext>,
    session_auth_store: RequestAuthSessionStore,
}

#[tokio::main]
async fn main() -> AppResult<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        println!("{USAGE}");
        return Ok(());
    }

    let mode = match parse_args(args) {
        Ok(mode) => mode,
        Err(message) => {
            eprintln!("{message}\n\n{USAGE}");
            std::process::exit(2);
        }
    };

    if let RunMode::Acceptance(command) = mode {
        let exit_code = acceptance::run(command).await?;
        if exit_code != 0 {
            std::process::exit(exit_code);
        }
        return Ok(());
    }

    if let RunMode::Smoke(command) = mode {
        let exit_code = smoke::run(command).await?;
        if exit_code != 0 {
            std::process::exit(exit_code);
        }
        return Ok(());
    }

    if let RunMode::StreamHttp(_, ref env_file_path) = mode {
        let env_path = env_file_path
            .clone()
            .or_else(|| std::env::var("ENV_FILE").ok());
        if let Some(path) = env_path {
            if let Err(error) = dotenvy::from_filename(&path) {
                eprintln!("Failed to load env file {}: {}", path, error);
                std::process::exit(1);
            }
            eprintln!("Loaded environment variables from: {}", path);
        } else if let Ok(path) = dotenvy::dotenv() {
            eprintln!("Loaded environment variables from: {}", path.display());
        }
    }

    let runtime_config = match mode.http_overrides() {
        Some(overrides) => config::RuntimeConfig::from_env_with_http_overrides(overrides)?,
        None => config::RuntimeConfig::from_env()?,
    };
    let context = Arc::new(AppContext::from_config(&runtime_config));

    init_tracing()?;
    log_runtime_context(&context);

    match mode {
        RunMode::Stdio => run_stdio(context).await?,
        RunMode::StreamHttp(_, _) => run_streamhttp(runtime_config.http, context).await?,
        RunMode::Acceptance(_) => unreachable!("acceptance mode returns before runtime startup"),
        RunMode::Smoke(_) => unreachable!("smoke mode returns before runtime startup"),
    }

    Ok(())
}

fn log_runtime_context(context: &AppContext) {
    let enabled_tools = context.enabled_tools().map(|tools| tools.len());
    let availability = context.service_availability();

    tracing::info!(
        read_only = context.read_only(),
        enabled_tools,
        enabled_toolsets = context.enabled_toolsets().len(),
        jira_configured = availability.jira,
        confluence_configured = availability.confluence,
        "loaded MCP runtime control plane"
    );
}

impl RunMode {
    fn http_overrides(&self) -> Option<HttpConfigOverrides> {
        match self {
            Self::Stdio => None,
            Self::StreamHttp(overrides, _) => Some(overrides.clone()),
            Self::Acceptance(_) => None,
            Self::Smoke(_) => None,
        }
    }
}

async fn run_stdio(context: Arc<AppContext>) -> AppResult<()> {
    tracing::info!(server = mcp::SERVER_NAME, "starting MCP stdio server");

    let service = AtlassianMcpServer::new(context)
        .serve(stdio())
        .await
        .inspect_err(|error| {
            tracing::error!(?error, "MCP stdio server error");
        })?;

    service.waiting().await?;
    Ok(())
}

async fn run_streamhttp(config: config::HttpConfig, context: Arc<AppContext>) -> AppResult<()> {
    let address: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let cancellation = CancellationToken::new();
    let server_context = context.clone();
    let session_auth_store = RequestAuthSessionStore::default();
    let service_session_auth_store = session_auth_store.clone();
    let service = StreamableHttpService::new(
        move || {
            Ok(AtlassianMcpServer::with_session_auth_store(
                server_context.clone(),
                service_session_auth_store.clone(),
            ))
        },
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default().with_cancellation_token(cancellation.child_token()),
    );
    let auth_state = StreamHttpAuthState {
        context,
        session_auth_store,
    };
    let app = Router::new()
        .nest_service(config.path.as_str(), service)
        .route_layer(middleware::from_fn_with_state(
            auth_state,
            streamhttp_request_auth,
        ))
        .route("/healthz", get(healthz));
    let listener = tokio::net::TcpListener::bind(address).await?;

    tracing::info!(
        server = mcp::SERVER_NAME,
        address = %address,
        endpoint = %config.path,
        "starting MCP streamable HTTP server"
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            if let Err(error) = tokio::signal::ctrl_c().await {
                tracing::error!(?error, "failed to listen for shutdown signal");
            }
            cancellation.cancel();
        })
        .await?;

    Ok(())
}

async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn streamhttp_request_auth(
    State(state): State<StreamHttpAuthState>,
    request: Request,
    next: Next,
) -> Response {
    let fingerprint = match state
        .session_auth_store
        .parse_and_enforce_headers(request.headers(), &state.context)
    {
        Ok(fingerprint) => fingerprint,
        Err(error) => return request_auth_error_response(error),
    };

    let response = next.run(request).await;
    if let Err(error) = state
        .session_auth_store
        .bind_response_headers(response.headers(), &fingerprint)
    {
        return request_auth_error_response(error);
    }

    response
}

fn request_auth_error_response(error: rmcp::ErrorData) -> Response {
    (StatusCode::BAD_REQUEST, redact_text(error.message.as_ref())).into_response()
}

fn init_tracing() -> AppResult<()> {
    let filter_spec = tracing_filter_spec_from_env(|key| std::env::var(key));
    let filter =
        EnvFilter::try_new(&filter_spec).unwrap_or_else(|_| EnvFilter::new(DEFAULT_RUST_LOG));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .try_init()?;

    Ok(())
}

fn tracing_filter_spec_from_env<F, E>(mut get_var: F) -> String
where
    F: FnMut(&str) -> Result<String, E>,
{
    if let Some(rust_log) = get_var(ENV_RUST_LOG)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return rust_log;
    }

    if config::parse_extended_truthy(get_var(ENV_TOOL_CALL_DEBUG).ok().as_deref()) {
        return TOOL_CALL_DEBUG_RUST_LOG.to_string();
    }

    DEFAULT_RUST_LOG.to_string()
}

fn parse_args<I, S>(args: I) -> Result<RunMode, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    match args.first().map(String::as_str) {
        None => Ok(RunMode::Stdio),
        Some("stdio") if args.len() <= 1 => Ok(RunMode::Stdio),
        Some("stdio") => Err(format!("unexpected argument for stdio: `{}`", args[1])),
        Some("streamhttp") => {
            let (overrides, env_file) = parse_streamhttp_args(&args[1..])?;
            Ok(RunMode::StreamHttp(overrides, env_file))
        }
        Some("acceptance") => {
            acceptance::parse_acceptance_args(&args[1..]).map(RunMode::Acceptance)
        }
        Some("smoke") => smoke::parse_smoke_args(&args[1..]).map(RunMode::Smoke),
        Some(command) => Err(format!("unknown command `{command}`")),
    }
}

fn parse_streamhttp_args(args: &[String]) -> Result<(HttpConfigOverrides, Option<String>), String> {
    let mut overrides = HttpConfigOverrides::default();
    let mut env_file = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--host" => {
                index += 1;
                overrides.host = Some(
                    args.get(index)
                        .ok_or_else(|| "--host requires a value".to_string())?
                        .clone(),
                );
            }
            "--port" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--port requires a value".to_string())?;
                overrides.port = Some(parse_cli_port(value)?);
            }
            "--path" => {
                index += 1;
                overrides.path = Some(
                    args.get(index)
                        .ok_or_else(|| "--path requires a value".to_string())?
                        .clone(),
                );
            }
            "--env-file" => {
                index += 1;
                env_file = Some(
                    args.get(index)
                        .ok_or_else(|| "--env-file requires a path".to_string())?
                        .clone(),
                );
            }
            arg if arg.starts_with("--host=") => {
                overrides.host = Some(
                    arg.strip_prefix("--host=")
                        .expect("prefix was just checked")
                        .to_string(),
                );
            }
            arg if arg.starts_with("--port=") => {
                let value = arg
                    .strip_prefix("--port=")
                    .expect("prefix was just checked");
                overrides.port = Some(parse_cli_port(value)?);
            }
            arg if arg.starts_with("--path=") => {
                overrides.path = Some(
                    arg.strip_prefix("--path=")
                        .expect("prefix was just checked")
                        .to_string(),
                );
            }
            arg if arg.starts_with("--env-file=") => {
                env_file = Some(
                    arg.strip_prefix("--env-file=")
                        .expect("prefix was just checked")
                        .to_string(),
                );
            }
            arg => return Err(format!("unexpected streamhttp argument `{arg}`")),
        }
        index += 1;
    }

    Ok((overrides, env_file))
}

fn parse_cli_port(value: &str) -> Result<u16, String> {
    value
        .parse()
        .map_err(|_| format!("invalid --port value `{value}`"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DEFAULT_HTTP_HOST, DEFAULT_HTTP_PATH, DEFAULT_HTTP_PORT};

    fn env_provider<'a>(
        pairs: &'a [(&'a str, &'a str)],
    ) -> impl FnMut(&str) -> Result<String, ()> + 'a {
        move |key| {
            pairs
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).to_string())
                .ok_or(())
        }
    }

    fn merge_http(overrides: HttpConfigOverrides) -> config::HttpConfig {
        config::HttpConfig::from_var_provider(|_| Err(()), overrides).unwrap()
    }

    #[test]
    fn parse_args_defaults_to_stdio() {
        assert_eq!(parse_args(Vec::<String>::new()).unwrap(), RunMode::Stdio);
        assert_eq!(parse_args(["stdio"]).unwrap(), RunMode::Stdio);
    }

    #[test]
    fn parse_args_accepts_streamhttp_env_defaults() {
        assert_eq!(
            parse_args(["streamhttp"]).unwrap(),
            RunMode::StreamHttp(HttpConfigOverrides::default(), None)
        );
        assert_eq!(
            merge_http(HttpConfigOverrides::default()),
            config::HttpConfig {
                host: DEFAULT_HTTP_HOST.to_string(),
                port: DEFAULT_HTTP_PORT,
                path: DEFAULT_HTTP_PATH.to_string(),
            }
        );
    }

    #[test]
    fn parse_args_accepts_streamhttp_host_port_and_path() {
        let mode = parse_args([
            "streamhttp",
            "--host",
            "0.0.0.0",
            "--port=9000",
            "--path",
            "alt-mcp",
        ])
        .unwrap();

        assert_eq!(
            mode,
            RunMode::StreamHttp(
                HttpConfigOverrides {
                    host: Some("0.0.0.0".to_string()),
                    port: Some(9000),
                    path: Some("alt-mcp".to_string()),
                },
                None
            )
        );

        assert_eq!(
            merge_http(match mode {
                RunMode::StreamHttp(overrides, _) => overrides,
                RunMode::Stdio => unreachable!("test parsed streamhttp"),
                RunMode::Acceptance(_) => unreachable!("test parsed streamhttp"),
                RunMode::Smoke(_) => unreachable!("test parsed streamhttp"),
            }),
            config::HttpConfig {
                host: "0.0.0.0".to_string(),
                port: 9000,
                path: "/alt-mcp".to_string(),
            }
        );
    }

    #[test]
    fn parse_args_rejects_invalid_streamhttp_port() {
        assert!(parse_args(["streamhttp", "--port", "bad"]).is_err());
        assert!(parse_args(["streamhttp", "--port=bad"]).is_err());
    }

    #[test]
    fn parse_args_rejects_unknown_command() {
        assert!(parse_args(["http"]).is_err());
    }

    #[test]
    fn parse_args_rejects_sse_transport() {
        let error = parse_args(["sse"]).unwrap_err();

        assert_eq!(error, "unknown command `sse`");
    }

    #[test]
    fn tracing_filter_defaults_to_info() {
        assert_eq!(
            tracing_filter_spec_from_env(env_provider(&[])),
            DEFAULT_RUST_LOG
        );
    }

    #[test]
    fn tracing_filter_enables_tool_call_debug_when_requested() {
        for value in ["true", "1", "yes", "y", "on"] {
            assert_eq!(
                tracing_filter_spec_from_env(env_provider(&[(ENV_TOOL_CALL_DEBUG, value)])),
                TOOL_CALL_DEBUG_RUST_LOG
            );
        }
    }

    #[test]
    fn tracing_filter_respects_rust_log_over_tool_call_debug() {
        assert_eq!(
            tracing_filter_spec_from_env(env_provider(&[
                (ENV_RUST_LOG, "warn,rmcp=debug"),
                (ENV_TOOL_CALL_DEBUG, "true"),
            ])),
            "warn,rmcp=debug"
        );
    }

    #[test]
    fn tracing_filter_treats_empty_rust_log_as_unset() {
        assert_eq!(
            tracing_filter_spec_from_env(env_provider(&[
                (ENV_RUST_LOG, "   "),
                (ENV_TOOL_CALL_DEBUG, "true"),
            ])),
            TOOL_CALL_DEBUG_RUST_LOG
        );
    }

    #[tokio::test]
    async fn healthz_reports_ok_without_runtime_details() {
        let Json(response) = healthz().await;

        assert_eq!(response.status, "ok");
    }
}
