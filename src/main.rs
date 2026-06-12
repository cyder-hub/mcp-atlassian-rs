mod atlassian;
mod cli;
mod config;
mod confluence;
mod context;
mod env_loader;
mod error;
mod gitlab;
mod jira;
mod mcp;
mod mcp_confluence_helpers;
mod mcp_errors;
mod operations;
mod tool_registry;
mod upstream;

use std::{net::SocketAddr, sync::Arc};

use axum::{Json, Router, routing::get};
use clap::{Args, Parser, Subcommand};
use config::HttpConfigOverrides;
use context::AppContext;
use mcp::WorkhubMcpServer;
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
const TOOL_CALL_DEBUG_RUST_LOG: &str = "workhub_rs::mcp=debug,workhub_rs=info,rmcp=info";

const USAGE: &str = "\
Usage:
  workhub -v
  workhub [stdio]
  workhub streamhttp [--host <host>] [--port <port>] [--path <path>] [--env-file <path>]
  workhub cli [--env-file <path>] [--json] [--pretty] <provider> <resource> <action> ...

Commands:
  stdio       Run the MCP server over standard input/output.
  streamhttp  Run the MCP server over streamable HTTP.
  cli         Run a resource-oriented Workhub CLI command.

Options:
  -v                 Print the package version.
  --env-file <path>  Load environment variables from the specified file (streamhttp and cli only).
                     Alternatively, set the ENV_FILE environment variable.

Defaults:
  host  127.0.0.1
  port  8000
  path  /mcp
";

#[derive(Debug, Clone, PartialEq, Eq)]
enum RunMode {
    Version,
    Stdio,
    StreamHttp {
        overrides: HttpConfigOverrides,
        env_file: Option<String>,
    },
    Cli(Box<cli::CliArgs>),
}

#[derive(Debug, serde::Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Debug, Parser)]
#[command(name = "workhub", version, disable_help_subcommand = true)]
struct RootArgs {
    #[command(subcommand)]
    command: RootCommand,
}

#[derive(Debug, Subcommand)]
enum RootCommand {
    Streamhttp(StreamHttpArgs),
    Cli(cli::CliArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
struct StreamHttpArgs {
    #[arg(long)]
    host: Option<String>,
    #[arg(long)]
    port: Option<u16>,
    #[arg(long)]
    path: Option<String>,
    #[arg(long, value_name = "path")]
    env_file: Option<String>,
}

#[derive(Debug)]
enum ParseArgsError {
    Usage(String),
    Clap(clap::Error),
}

#[tokio::main]
async fn main() -> AppResult<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mode = match parse_args(args) {
        Ok(mode) => mode,
        Err(ParseArgsError::Usage(message)) => {
            eprintln!("{message}\n\n{USAGE}");
            std::process::exit(2);
        }
        Err(ParseArgsError::Clap(error)) => error.exit(),
    };

    if mode == RunMode::Version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if mode.loads_dotenv() {
        match env_loader::load_dotenv(mode.explicit_env_file()) {
            Ok(Some(path)) if mode.reports_dotenv_success() => {
                eprintln!("Loaded environment variables from: {}", path.display())
            }
            Ok(Some(_)) => {}
            Ok(None) => {}
            Err(error) => {
                if let RunMode::Cli(args) = &mode {
                    exit_cli_error(cli::render_config_error(args, error.to_string()));
                }
                eprintln!("{error}");
                std::process::exit(3);
            }
        }
    }

    let runtime_config = match &mode {
        RunMode::Cli(args) => config::RuntimeConfig::from_env_for_cli().unwrap_or_else(|error| {
            exit_cli_error(cli::render_config_error(args, error.to_string()))
        }),
        RunMode::StreamHttp { overrides, .. } => {
            config::RuntimeConfig::from_env_with_http_overrides(overrides.clone())?
        }
        RunMode::Version | RunMode::Stdio => config::RuntimeConfig::from_env()?,
    };
    let context = Arc::new(AppContext::from_config(&runtime_config));

    if !matches!(mode, RunMode::Cli(_)) {
        init_tracing()?;
        log_runtime_context(&context);
    }

    match mode {
        RunMode::Version => unreachable!("version mode returned before runtime startup"),
        RunMode::Stdio => run_stdio(context).await?,
        RunMode::StreamHttp { .. } => run_streamhttp(runtime_config.http, context).await?,
        RunMode::Cli(args) => {
            if let Err(error) = cli::run(*args, context).await {
                exit_cli_error(error);
            }
        }
    }

    Ok(())
}

fn exit_cli_error(error: cli::CliRunError) -> ! {
    eprintln!("{error}");
    std::process::exit(error.exit_code());
}

fn log_runtime_context(context: &AppContext) {
    let mcp_enabled_tools = context.mcp_enabled_tools().map(|tools| tools.len());
    let availability = context.service_availability();

    tracing::info!(
        mcp_enabled_tools,
        mcp_disabled_tools = context.mcp_disabled_tools().len(),
        mcp_enabled_toolsets = context.mcp_enabled_toolsets().len(),
        jira_configured = availability.jira,
        confluence_configured = availability.confluence,
        gitlab_configured = availability.gitlab,
        "loaded MCP runtime control plane"
    );
}

impl RunMode {
    fn explicit_env_file(&self) -> Option<&str> {
        match self {
            Self::StreamHttp { env_file, .. } => env_file.as_deref(),
            Self::Cli(args) => args.env_file.as_deref(),
            Self::Version | Self::Stdio => None,
        }
    }

    fn loads_dotenv(&self) -> bool {
        matches!(self, Self::StreamHttp { .. } | Self::Cli(_))
    }

    fn reports_dotenv_success(&self) -> bool {
        matches!(self, Self::StreamHttp { .. })
    }
}

async fn run_stdio(context: Arc<AppContext>) -> AppResult<()> {
    tracing::info!(server = mcp::SERVER_NAME, "starting MCP stdio server");

    let service = WorkhubMcpServer::new(context)
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
        move || Ok(WorkhubMcpServer::new(server_context.clone())),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default().with_cancellation_token(cancellation.child_token()),
    );
    let app = Router::new()
        .nest_service(config.path.as_str(), service)
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

fn parse_args<I, S>(args: I) -> Result<RunMode, ParseArgsError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    match args.first().map(String::as_str) {
        None => Ok(RunMode::Stdio),
        Some("stdio") if args.len() <= 1 => Ok(RunMode::Stdio),
        Some("stdio") => Err(ParseArgsError::Usage(format!(
            "unexpected argument for stdio: `{}`",
            args[1]
        ))),
        Some("-v") if args.len() == 1 => Ok(RunMode::Version),
        _ => {
            let argv = std::iter::once("workhub".to_string()).chain(args);
            let root = RootArgs::try_parse_from(argv).map_err(ParseArgsError::Clap)?;
            Ok(match root.command {
                RootCommand::Streamhttp(args) => RunMode::StreamHttp {
                    overrides: HttpConfigOverrides {
                        host: args.host,
                        port: args.port,
                        path: args.path,
                    },
                    env_file: args.env_file,
                },
                RootCommand::Cli(args) => RunMode::Cli(Box::new(args)),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DEFAULT_HTTP_HOST, DEFAULT_HTTP_PATH, DEFAULT_HTTP_PORT};
    use clap::error::ErrorKind;

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
        assert!(!parse_args(Vec::<String>::new()).unwrap().loads_dotenv());
        assert!(!parse_args(["stdio"]).unwrap().loads_dotenv());
    }

    #[test]
    fn parse_args_accepts_streamhttp_env_defaults() {
        assert_eq!(
            parse_args(["streamhttp"]).unwrap(),
            RunMode::StreamHttp {
                overrides: HttpConfigOverrides::default(),
                env_file: None,
            }
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
            RunMode::StreamHttp {
                overrides: HttpConfigOverrides {
                    host: Some("0.0.0.0".to_string()),
                    port: Some(9000),
                    path: Some("alt-mcp".to_string()),
                },
                env_file: None,
            }
        );

        assert_eq!(
            merge_http(match mode {
                RunMode::StreamHttp { overrides, .. } => overrides,
                _ => unreachable!("test parsed streamhttp"),
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
        for args in [
            ["streamhttp", "--port", "bad"].as_slice(),
            ["streamhttp", "--port=bad"].as_slice(),
        ] {
            let Err(ParseArgsError::Clap(error)) = parse_args(args.iter().copied()) else {
                panic!("expected clap error");
            };
            assert_eq!(error.kind(), ErrorKind::ValueValidation);
        }
    }

    #[test]
    fn parse_args_rejects_unknown_command() {
        for args in [
            ["http"].as_slice(),
            ["acceptance", "jira", "--preflight"].as_slice(),
            ["smoke", "jira", "restricted"].as_slice(),
        ] {
            let Err(ParseArgsError::Clap(error)) = parse_args(args.iter().copied()) else {
                panic!("expected clap error");
            };
            assert_eq!(error.kind(), ErrorKind::InvalidSubcommand);
        }
    }

    #[test]
    fn parse_args_rejects_sse_transport() {
        let Err(ParseArgsError::Clap(error)) = parse_args(["sse"]) else {
            panic!("expected clap error");
        };

        assert_eq!(error.kind(), ErrorKind::InvalidSubcommand);
    }

    #[test]
    fn parse_args_keeps_stdio_fast_path_errors_out_of_clap() {
        let Err(ParseArgsError::Usage(error)) = parse_args(["stdio", "--env-file", ".env"]) else {
            panic!("expected fast-path usage error");
        };

        assert_eq!(error, "unexpected argument for stdio: `--env-file`");
    }

    #[test]
    fn parse_args_accepts_cli_mode_and_env_file() {
        let mode = parse_args([
            "cli",
            "--env-file",
            "workhub.env",
            "--json",
            "jira",
            "project",
            "list",
        ])
        .unwrap();

        assert!(mode.loads_dotenv());
        assert!(!mode.reports_dotenv_success());
        assert_eq!(mode.explicit_env_file(), Some("workhub.env"));
        assert!(matches!(mode, RunMode::Cli(_)));
    }

    #[test]
    fn parse_args_reports_dotenv_success_only_for_streamhttp() {
        assert!(
            parse_args(["streamhttp", "--env-file", "workhub.env"])
                .unwrap()
                .reports_dotenv_success()
        );
        assert!(
            !parse_args([
                "cli",
                "--env-file",
                "workhub.env",
                "jira",
                "project",
                "list"
            ])
            .unwrap()
            .reports_dotenv_success()
        );
    }

    #[test]
    fn parse_args_allows_nested_cli_help_without_top_level_help_scan() {
        let Err(ParseArgsError::Clap(error)) = parse_args(["cli", "jira", "issue", "--help"])
        else {
            panic!("expected clap display help");
        };

        assert_eq!(error.kind(), ErrorKind::DisplayHelp);
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
