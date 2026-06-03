mod atlassian;
mod config;
mod context;
mod error;
mod jira;
mod mcp;
mod tool_registry;

use std::{net::SocketAddr, sync::Arc};

use axum::{Json, Router, routing::get};
use config::HttpConfigOverrides;
use context::AppContext;
use mcp::AtlassianMcpServer;
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

const USAGE: &str = "\
Usage:
  mcp-atlassian-rs [stdio]
  mcp-atlassian-rs streamhttp [--host <host>] [--port <port>] [--path <path>]

Commands:
  stdio       Run the MCP server over standard input/output.
  streamhttp  Run the MCP server over streamable HTTP.

Defaults:
  host  127.0.0.1
  port  8000
  path  /mcp
";

#[derive(Debug, Clone, PartialEq, Eq)]
enum RunMode {
    Stdio,
    StreamHttp(HttpConfigOverrides),
}

#[derive(Debug, serde::Serialize)]
struct HealthResponse {
    status: &'static str,
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

    let runtime_config = match mode.http_overrides() {
        Some(overrides) => config::RuntimeConfig::from_env_with_http_overrides(overrides)?,
        None => config::RuntimeConfig::from_env()?,
    };
    let context = Arc::new(AppContext::from_config(&runtime_config));

    init_tracing()?;
    log_runtime_context(&context);

    match mode {
        RunMode::Stdio => run_stdio(context).await?,
        RunMode::StreamHttp(_) => run_streamhttp(runtime_config.http, context).await?,
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
            Self::StreamHttp(overrides) => Some(overrides.clone()),
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
    let service = StreamableHttpService::new(
        move || Ok(AtlassianMcpServer::new(server_context.clone())),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default().with_cancellation_token(cancellation.child_token()),
    );
    let app = Router::new()
        .route("/healthz", get(healthz))
        .nest_service(config.path.as_str(), service);
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

fn init_tracing() -> AppResult<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .try_init()?;

    Ok(())
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
        Some("streamhttp") => parse_streamhttp_args(&args[1..]).map(RunMode::StreamHttp),
        Some(command) => Err(format!("unknown command `{command}`")),
    }
}

fn parse_streamhttp_args(args: &[String]) -> Result<HttpConfigOverrides, String> {
    let mut overrides = HttpConfigOverrides::default();
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
            arg => return Err(format!("unexpected streamhttp argument `{arg}`")),
        }
        index += 1;
    }

    Ok(overrides)
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
            RunMode::StreamHttp(HttpConfigOverrides::default())
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
            RunMode::StreamHttp(HttpConfigOverrides {
                host: Some("0.0.0.0".to_string()),
                port: Some(9000),
                path: Some("alt-mcp".to_string()),
            })
        );

        assert_eq!(
            merge_http(match mode {
                RunMode::StreamHttp(overrides) => overrides,
                RunMode::Stdio => unreachable!("test parsed streamhttp"),
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

    #[tokio::test]
    async fn healthz_reports_ok_without_runtime_details() {
        let Json(response) = healthz().await;

        assert_eq!(response.status, "ok");
    }
}
