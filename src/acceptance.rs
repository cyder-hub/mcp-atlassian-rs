use std::{
    collections::BTreeMap,
    env,
    fs::File,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::{Value, json};

use crate::{
    AppResult,
    atlassian::redaction::{env_secret_values_from_pairs, redact_text_with_secrets},
};

const BLOCKED: i32 = 2;
const FAILED: i32 = 1;
const ENV_FILE_VAR: &str = "ACCEPTANCE_ENV_FILE";
const LEGACY_ENV_FILE_VAR: &str = "STAGE5_ENV_FILE";
const SECRET_KEYS: &[&str] = &[
    "JIRA_API_TOKEN",
    "JIRA_PERSONAL_TOKEN",
    "CONFLUENCE_API_TOKEN",
    "CONFLUENCE_PERSONAL_TOKEN",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceptanceMode {
    Jira,
    Confluence,
    Mcp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptanceCommand {
    pub mode: AcceptanceMode,
    pub action: AcceptanceAction,
    pub env_file: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceptanceAction {
    Preflight,
    Run { binary: PathBuf },
}

#[derive(Debug)]
struct Row {
    toolset: &'static str,
    tool: &'static str,
    path: &'static str,
    required_env: &'static [&'static str],
    arguments: Value,
    read_only: bool,
    expect_blocked: bool,
}

#[derive(Debug)]
struct McpRow {
    toolset: &'static str,
    tool: &'static str,
    path: &'static str,
    required_env: &'static [&'static str],
    arguments: Option<Value>,
    transport: McpTransport,
}

#[derive(Debug, Clone, Copy)]
enum McpTransport {
    StdioList,
    Stdio,
    Http,
}

type EnvMap = BTreeMap<String, String>;

pub fn parse_acceptance_args(args: &[String]) -> Result<AcceptanceCommand, String> {
    let Some(mode) = args.first() else {
        return Err("acceptance requires a mode: jira, confluence, or mcp".to_string());
    };
    let mode = match mode.as_str() {
        "jira" => AcceptanceMode::Jira,
        "confluence" => AcceptanceMode::Confluence,
        "mcp" => AcceptanceMode::Mcp,
        other => return Err(format!("unknown acceptance mode `{other}`")),
    };

    let mut env_file = None;
    let mut action = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--preflight" => {
                if action.replace(AcceptanceAction::Preflight).is_some() {
                    return Err("acceptance accepts only one action".to_string());
                }
            }
            "--run" => {
                index += 1;
                let binary = args
                    .get(index)
                    .ok_or_else(|| "--run requires a binary path".to_string())?;
                if action
                    .replace(AcceptanceAction::Run {
                        binary: PathBuf::from(binary),
                    })
                    .is_some()
                {
                    return Err("acceptance accepts only one action".to_string());
                }
            }
            "--env-file" => {
                index += 1;
                let path = args
                    .get(index)
                    .ok_or_else(|| "--env-file requires a path".to_string())?;
                env_file = Some(PathBuf::from(path));
            }
            arg if arg.starts_with("--env-file=") => {
                env_file = Some(PathBuf::from(
                    arg.strip_prefix("--env-file=")
                        .expect("prefix was just checked"),
                ));
            }
            arg => return Err(format!("unexpected acceptance argument `{arg}`")),
        }
        index += 1;
    }

    Ok(AcceptanceCommand {
        mode,
        action: action.ok_or_else(|| "acceptance requires --preflight or --run".to_string())?,
        env_file,
    })
}

pub async fn run(command: AcceptanceCommand) -> AppResult<i32> {
    let env = match load_env(command.env_file.as_deref()) {
        Ok(env) => env,
        Err(error) => {
            print_header();
            print_row(
                command.mode.toolset(),
                command.mode.command_name(),
                "preflight",
                "failed",
                "env_file_error",
                &error,
            );
            return Ok(FAILED);
        }
    };

    let base_status = preflight(&command.mode, &env);
    if matches!(command.action, AcceptanceAction::Preflight) || base_status != 0 {
        return Ok(base_status);
    }

    let AcceptanceAction::Run { binary } = command.action else {
        unreachable!("preflight action returned above");
    };

    let status = match command.mode {
        AcceptanceMode::Jira => run_rows(&binary, &jira_rows(&env), &env),
        AcceptanceMode::Confluence => run_rows(&binary, &confluence_rows(&env), &env),
        AcceptanceMode::Mcp => run_mcp(&binary, &env).await,
    };

    Ok(status)
}

impl AcceptanceMode {
    fn toolset(&self) -> &'static str {
        match self {
            Self::Jira => "jira",
            Self::Confluence => "confluence",
            Self::Mcp => "mcp",
        }
    }

    fn command_name(&self) -> &'static str {
        match self {
            Self::Jira => "acceptance-jira",
            Self::Confluence => "acceptance-confluence",
            Self::Mcp => "acceptance-mcp",
        }
    }
}

fn load_env(env_file: Option<&Path>) -> Result<EnvMap, String> {
    let mut loaded = env::vars().collect::<EnvMap>();
    let path = env_file
        .map(Path::to_path_buf)
        .or_else(|| loaded.get(ENV_FILE_VAR).map(PathBuf::from))
        .or_else(|| loaded.get(LEGACY_ENV_FILE_VAR).map(PathBuf::from))
        .unwrap_or_else(default_env_file);
    let explicit = env_file.is_some()
        || loaded.contains_key(ENV_FILE_VAR)
        || loaded.contains_key(LEGACY_ENV_FILE_VAR);

    if !path.exists() {
        if explicit {
            return Err(format!("env file not found: {}", path.display()));
        }
        return Ok(loaded);
    }

    let file = File::open(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    for (line_number, line) in BufReader::new(file).lines().enumerate() {
        let line = line.map_err(|error| format!("{}: {error}", path.display()))?;
        let Some((key, value)) = parse_env_line(&line, &path, line_number + 1)? else {
            continue;
        };
        if loaded.get(&key).is_none_or(String::is_empty) {
            loaded.insert(key, value);
        }
    }

    Ok(loaded)
}

fn default_env_file() -> PathBuf {
    env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".env")
}

fn parse_env_line(
    line: &str,
    path: &Path,
    line_number: usize,
) -> Result<Option<(String, String)>, String> {
    let mut line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    if let Some(rest) = line.strip_prefix("export ") {
        line = rest.trim_start();
    }
    let Some((key, raw_value)) = line.split_once('=') else {
        return Err(format!(
            "{}:{line_number}: expected KEY=VALUE",
            path.display()
        ));
    };
    let key = key.trim();
    if !is_valid_env_key(key) {
        return Err(format!("{}:{line_number}: invalid env key", path.display()));
    }
    Ok(Some((key.to_string(), parse_env_value(raw_value))))
}

fn is_valid_env_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn parse_env_value(raw_value: &str) -> String {
    let value = strip_inline_comment(raw_value.trim()).trim().to_string();
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\'')
            || (bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"'))
    {
        let inner = &value[1..value.len() - 1];
        if bytes[0] == b'"' {
            inner
                .replace("\\n", "\n")
                .replace("\\r", "\r")
                .replace("\\t", "\t")
                .replace("\\\"", "\"")
                .replace("\\\\", "\\")
        } else {
            inner.to_string()
        }
    } else {
        value
    }
}

fn strip_inline_comment(value: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    for (index, ch) in value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && in_double {
            escaped = true;
            continue;
        }
        if ch == '\'' && !in_double {
            in_single = !in_single;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            continue;
        }
        if ch == '#'
            && !in_single
            && !in_double
            && (index == 0 || value[..index].ends_with(char::is_whitespace))
        {
            return value[..index].trim_end();
        }
    }

    value
}

fn preflight(mode: &AcceptanceMode, env: &EnvMap) -> i32 {
    let missing = missing_base_env(mode, env);
    if missing.is_empty() {
        return 0;
    }

    print_header();
    print_row(
        mode.toolset(),
        mode.command_name(),
        "preflight",
        "blocked",
        &format!("blocked:missing_env:{}", missing.join(",")),
        "not_run",
    );
    BLOCKED
}

fn missing_base_env(mode: &AcceptanceMode, env: &EnvMap) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if matches!(mode, AcceptanceMode::Jira | AcceptanceMode::Mcp) {
        if env_value(env, "JIRA_URL").is_empty() {
            missing.push("JIRA_URL");
        }
        if !has_jira_auth(env) {
            missing.push("JIRA_AUTH");
        }
    }
    if matches!(mode, AcceptanceMode::Confluence | AcceptanceMode::Mcp) {
        if env_value(env, "CONFLUENCE_URL").is_empty() {
            missing.push("CONFLUENCE_URL");
        }
        if !has_confluence_auth(env) {
            missing.push("CONFLUENCE_AUTH");
        }
    }
    missing
}

fn has_jira_auth(env: &EnvMap) -> bool {
    !env_value(env, "JIRA_PERSONAL_TOKEN").is_empty()
        || (!env_value(env, "JIRA_USERNAME").is_empty()
            && !env_value(env, "JIRA_API_TOKEN").is_empty())
}

fn has_confluence_auth(env: &EnvMap) -> bool {
    !env_value(env, "CONFLUENCE_PERSONAL_TOKEN").is_empty()
        || (!env_value(env, "CONFLUENCE_USERNAME").is_empty()
            && !env_value(env, "CONFLUENCE_API_TOKEN").is_empty())
}

fn env_value<'a>(env: &'a EnvMap, key: &str) -> &'a str {
    let value = env.get(key).map(String::as_str).unwrap_or_default().trim();
    if !value.is_empty() {
        return value;
    }

    legacy_env_key(key)
        .and_then(|legacy| env.get(legacy))
        .map(String::as_str)
        .unwrap_or_default()
        .trim()
}

fn legacy_env_key(key: &str) -> Option<&'static str> {
    match key {
        "JIRA_READ_ISSUE" => Some("STAGE5_JIRA_READ_ISSUE"),
        "JIRA_PROJECT_KEY" => Some("STAGE5_JIRA_PROJECT_KEY"),
        "JIRA_FIELD_ID" => Some("STAGE5_JIRA_FIELD_ID"),
        "JIRA_FIELD_CONTEXT_ID" => Some("STAGE5_JIRA_FIELD_CONTEXT_ID"),
        "JIRA_WATCHER_USER" => Some("STAGE5_JIRA_WATCHER_USER"),
        "JIRA_SERVICE_DESK_ID" => Some("STAGE5_JIRA_SERVICE_DESK_ID"),
        "JIRA_QUEUE_ID" => Some("STAGE5_JIRA_QUEUE_ID"),
        "JIRA_FORM_ID" => Some("STAGE5_JIRA_FORM_ID"),
        "CONFLUENCE_SEARCH_QUERY" => Some("STAGE5_CONFLUENCE_SEARCH_QUERY"),
        "CONFLUENCE_PAGE_ID" => Some("STAGE5_CONFLUENCE_PAGE_ID"),
        "CONFLUENCE_SPACE_KEY" => Some("STAGE5_CONFLUENCE_SPACE_KEY"),
        "CONFLUENCE_TEST_PAGE_PREFIX" => Some("STAGE5_CONFLUENCE_TEST_PAGE_PREFIX"),
        "CONFLUENCE_MUTATION_PAGE_ID" => Some("STAGE5_CONFLUENCE_MUTATION_PAGE_ID"),
        "CONFLUENCE_ATTACHMENT_ID" => Some("STAGE5_CONFLUENCE_ATTACHMENT_ID"),
        "CONFLUENCE_ATTACHMENT_FILE" => Some("STAGE5_CONFLUENCE_ATTACHMENT_FILE"),
        "CONFLUENCE_COMMENT_ID" => Some("STAGE5_CONFLUENCE_COMMENT_ID"),
        "CONFLUENCE_LABEL_NAME" => Some("STAGE5_CONFLUENCE_LABEL_NAME"),
        _ => None,
    }
}

fn print_header() {
    println!("toolset\ttool\tpath\tstatus\tblocker\tevidence");
}

fn print_row(toolset: &str, tool: &str, path: &str, status: &str, blocker: &str, evidence: &str) {
    println!("{toolset}\t{tool}\t{path}\t{status}\t{blocker}\t{evidence}");
}

fn blocked_row(toolset: &str, tool: &str, path: &str, missing: &[&str]) {
    print_row(
        toolset,
        tool,
        path,
        "blocked",
        &format!("blocked:missing_env:{}", missing.join(",")),
        "not_run",
    );
}

fn run_rows(binary: &Path, rows: &[Row], env: &EnvMap) -> i32 {
    print_header();
    let mut exit_code = 0;

    for row in rows {
        let missing = missing_required_env(row.required_env, env);
        if !missing.is_empty() {
            blocked_row(row.toolset, row.tool, row.path, &missing);
            exit_code = exit_code.max(BLOCKED);
            continue;
        }

        let (status, blocker, evidence) =
            match call_stdio_tool(binary, env, row.tool, &row.arguments, row.read_only) {
                Ok(response) => classify_response(&response, row.expect_blocked, env),
                Err(error) => (
                    "failed".to_string(),
                    "runner_error".to_string(),
                    redact_text(&error, env).chars().take(180).collect(),
                ),
            };

        print_row(
            row.toolset,
            row.tool,
            row.path,
            &status,
            &blocker,
            &evidence,
        );
        if status == "blocked" {
            exit_code = exit_code.max(BLOCKED);
        } else if status != "ok" {
            exit_code = exit_code.max(FAILED);
        }
    }

    exit_code
}

fn missing_required_env<'a>(required: &'static [&'static str], env: &EnvMap) -> Vec<&'a str> {
    required
        .iter()
        .copied()
        .filter(|key| env_value(env, key).is_empty())
        .collect()
}

fn jira_rows(env: &EnvMap) -> Vec<Row> {
    let issue = env_value(env, "JIRA_READ_ISSUE");
    let project = env_value(env, "JIRA_PROJECT_KEY");
    let field = env_value(env, "JIRA_FIELD_ID");
    let field_context = env_value(env, "JIRA_FIELD_CONTEXT_ID");
    let watcher = env_value(env, "JIRA_WATCHER_USER");
    let service_desk = env_value(env, "JIRA_SERVICE_DESK_ID");
    let queue = env_value(env, "JIRA_QUEUE_ID");
    let form = env_value(env, "JIRA_FORM_ID");

    vec![
        row(
            "jira_issues",
            "jira_get_issue",
            "jira/core/read_issue",
            &["JIRA_READ_ISSUE"],
            json!({"issue_key": issue, "fields": ["summary", "status"]}),
            false,
            false,
        ),
        row(
            "jira_issues",
            "jira_search",
            "jira/core/search",
            &["JIRA_PROJECT_KEY"],
            json!({"jql": format!("project = {project}"), "limit": 1}),
            false,
            false,
        ),
        row(
            "jira_issues",
            "jira_get_project_issues",
            "jira/core/project_issues",
            &["JIRA_PROJECT_KEY"],
            json!({"project_key": project, "limit": 1}),
            false,
            false,
        ),
        row(
            "jira_fields",
            "jira_search_fields",
            "jira/core/search_fields",
            &[],
            json!({"keyword": "summary", "limit": 10}),
            false,
            false,
        ),
        row(
            "jira_fields",
            "jira_get_field_options",
            "jira/core/field_options",
            &["JIRA_FIELD_ID", "JIRA_FIELD_CONTEXT_ID"],
            json!({"field_id": field, "context_id": field_context, "return_limit": 10}),
            false,
            false,
        ),
        row(
            "jira_watchers",
            "jira_get_issue_watchers",
            "jira/core/watchers",
            &["JIRA_READ_ISSUE"],
            json!({"issue_key": issue}),
            false,
            false,
        ),
        row(
            "jira_watchers",
            "jira_remove_watcher",
            "jira/core/read_only_watcher_remove",
            &["JIRA_READ_ISSUE", "JIRA_WATCHER_USER"],
            json!({"issue_key": issue, "user_identifier": watcher}),
            true,
            true,
        ),
        row(
            "jira_issues",
            "jira_create_issue",
            "jira/core/read_only_create_issue",
            &["JIRA_PROJECT_KEY"],
            json!({"project_key": project, "summary": "Acceptance read-only guard probe", "issue_type": "Task"}),
            true,
            true,
        ),
        row(
            "jira_agile",
            "jira_get_agile_boards",
            "jira/product/agile_boards",
            &["JIRA_PROJECT_KEY"],
            json!({"project_key": project, "limit": 5}),
            false,
            false,
        ),
        row(
            "jira_service_desk",
            "jira_get_service_desk_for_project",
            "jira/product/jsm_service_desk",
            &["JIRA_PROJECT_KEY"],
            json!({"project_key": project}),
            false,
            false,
        ),
        row(
            "jira_service_desk",
            "jira_get_service_desk_queues",
            "jira/product/jsm_queues",
            &["JIRA_SERVICE_DESK_ID"],
            json!({"service_desk_id": service_desk, "limit": 5}),
            false,
            false,
        ),
        row(
            "jira_service_desk",
            "jira_get_queue_issues",
            "jira/product/jsm_queue_issues",
            &["JIRA_SERVICE_DESK_ID", "JIRA_QUEUE_ID"],
            json!({"service_desk_id": service_desk, "queue_id": queue, "limit": 5}),
            false,
            false,
        ),
        row(
            "jira_forms",
            "jira_get_issue_proforma_forms",
            "jira/product/forms",
            &["JIRA_READ_ISSUE"],
            json!({"issue_key": issue}),
            false,
            false,
        ),
        row(
            "jira_forms",
            "jira_get_proforma_form_details",
            "jira/product/form_details",
            &["JIRA_READ_ISSUE", "JIRA_FORM_ID"],
            json!({"issue_key": issue, "form_id": form}),
            false,
            false,
        ),
        row(
            "jira_metrics",
            "jira_get_issue_sla",
            "jira/product/sla",
            &["JIRA_READ_ISSUE"],
            json!({"issue_key": issue, "include_raw_dates": false}),
            false,
            false,
        ),
        row(
            "jira_development",
            "jira_get_issue_development_info",
            "jira/product/dev_status_single",
            &["JIRA_READ_ISSUE"],
            json!({"issue_key": issue, "application_type": "bitbucket", "data_type": "repository"}),
            false,
            false,
        ),
        row(
            "jira_development",
            "jira_get_issues_development_info",
            "jira/product/dev_status_batch",
            &["JIRA_READ_ISSUE"],
            json!({"issue_keys": [issue], "application_type": "bitbucket", "data_type": "repository"}),
            false,
            false,
        ),
    ]
}

fn confluence_rows(env: &EnvMap) -> Vec<Row> {
    let query = env_value(env, "CONFLUENCE_SEARCH_QUERY");
    let page = env_value(env, "CONFLUENCE_PAGE_ID");
    let space = env_value(env, "CONFLUENCE_SPACE_KEY");
    let prefix = env_value(env, "CONFLUENCE_TEST_PAGE_PREFIX");
    let mutation_page = env_value(env, "CONFLUENCE_MUTATION_PAGE_ID");
    let attachment = env_value(env, "CONFLUENCE_ATTACHMENT_ID");
    let file_path = env_value(env, "CONFLUENCE_ATTACHMENT_FILE");
    let comment = env_value(env, "CONFLUENCE_COMMENT_ID");
    let label = env_value(env, "CONFLUENCE_LABEL_NAME");
    let label = if label.is_empty() {
        "acceptance"
    } else {
        label
    };
    let create_title = format!("{prefix} create {}", acceptance_run_suffix());

    vec![
        row(
            "confluence_pages",
            "confluence_search",
            "confluence/core/search",
            &["CONFLUENCE_SEARCH_QUERY"],
            json!({"query": query, "limit": 5}),
            false,
            false,
        ),
        row(
            "confluence_pages",
            "confluence_get_page",
            "confluence/core/get_page",
            &["CONFLUENCE_PAGE_ID"],
            json!({"page_id": page, "include_metadata": true, "convert_to_markdown": true}),
            false,
            false,
        ),
        row(
            "confluence_pages",
            "confluence_get_page_children",
            "confluence/core/page_children",
            &["CONFLUENCE_PAGE_ID"],
            json!({"parent_id": page, "limit": 10, "convert_to_markdown": false}),
            false,
            false,
        ),
        row(
            "confluence_pages",
            "confluence_get_space_page_tree",
            "confluence/core/page_tree",
            &["CONFLUENCE_SPACE_KEY"],
            json!({"space_key": space, "limit": 25}),
            false,
            false,
        ),
        row(
            "confluence_comments",
            "confluence_get_comments",
            "confluence/core/comments",
            &["CONFLUENCE_PAGE_ID"],
            json!({"page_id": page}),
            false,
            false,
        ),
        row(
            "confluence_pages",
            "confluence_create_page",
            "confluence/core/read_only_create_page",
            &["CONFLUENCE_SPACE_KEY", "CONFLUENCE_TEST_PAGE_PREFIX"],
            json!({"space_key": space, "title": format!("{prefix} read-only probe"), "content": "Acceptance read-only guard probe", "content_format": "markdown"}),
            true,
            true,
        ),
        row(
            "confluence_pages",
            "confluence_create_page",
            "confluence/write/create_page",
            &["CONFLUENCE_SPACE_KEY", "CONFLUENCE_TEST_PAGE_PREFIX"],
            json!({"space_key": space, "title": create_title, "content": "Acceptance create probe", "content_format": "markdown"}),
            false,
            false,
        ),
        row(
            "confluence_pages",
            "confluence_update_page",
            "confluence/write/update_page",
            &["CONFLUENCE_MUTATION_PAGE_ID", "CONFLUENCE_TEST_PAGE_PREFIX"],
            json!({"page_id": mutation_page, "title": format!("{prefix} update"), "content": "Acceptance update probe", "content_format": "markdown"}),
            false,
            false,
        ),
        row(
            "confluence_pages",
            "confluence_delete_page",
            "confluence/write/read_only_delete_page",
            &["CONFLUENCE_MUTATION_PAGE_ID"],
            json!({"page_id": mutation_page}),
            true,
            true,
        ),
        row(
            "confluence_pages",
            "confluence_move_page",
            "confluence/write/read_only_move_page",
            &["CONFLUENCE_MUTATION_PAGE_ID", "CONFLUENCE_PAGE_ID"],
            json!({"page_id": mutation_page, "target_parent_id": page, "position": "append"}),
            true,
            true,
        ),
        row(
            "confluence_comments",
            "confluence_add_comment",
            "confluence/write/add_comment",
            &["CONFLUENCE_PAGE_ID"],
            json!({"page_id": page, "body": "Acceptance comment probe"}),
            false,
            false,
        ),
        row(
            "confluence_comments",
            "confluence_reply_to_comment",
            "confluence/write/reply_to_comment",
            &["CONFLUENCE_COMMENT_ID"],
            json!({"comment_id": comment, "body": "Acceptance reply probe"}),
            false,
            false,
        ),
        row(
            "confluence_labels",
            "confluence_get_labels",
            "confluence/write/get_labels",
            &["CONFLUENCE_PAGE_ID"],
            json!({"page_id": page}),
            false,
            false,
        ),
        row(
            "confluence_labels",
            "confluence_add_label",
            "confluence/write/add_label",
            &["CONFLUENCE_PAGE_ID"],
            json!({"page_id": page, "name": label}),
            false,
            false,
        ),
        row(
            "confluence_analytics",
            "confluence_get_page_views",
            "confluence/product/cloud_page_views",
            &["CONFLUENCE_PAGE_ID"],
            json!({"page_id": page, "include_title": true}),
            false,
            false,
        ),
        row(
            "confluence_attachments",
            "confluence_get_attachments",
            "confluence/attachments/list",
            &["CONFLUENCE_PAGE_ID"],
            json!({"content_id": page, "limit": 25}),
            false,
            false,
        ),
        row(
            "confluence_attachments",
            "confluence_download_attachment",
            "confluence/attachments/download_one",
            &["CONFLUENCE_ATTACHMENT_ID"],
            json!({"attachment_id": attachment}),
            false,
            false,
        ),
        row(
            "confluence_attachments",
            "confluence_download_content_attachments",
            "confluence/attachments/download_content",
            &["CONFLUENCE_PAGE_ID"],
            json!({"content_id": page}),
            false,
            false,
        ),
        row(
            "confluence_attachments",
            "confluence_get_page_images",
            "confluence/attachments/page_images",
            &["CONFLUENCE_PAGE_ID"],
            json!({"content_id": page}),
            false,
            false,
        ),
        row(
            "confluence_attachments",
            "confluence_upload_attachment",
            "confluence/attachments/upload",
            &["CONFLUENCE_PAGE_ID", "CONFLUENCE_ATTACHMENT_FILE"],
            json!({"content_id": page, "file_path": file_path, "comment": "Acceptance upload probe"}),
            false,
            false,
        ),
        row(
            "confluence_attachments",
            "confluence_upload_attachments",
            "confluence/attachments/upload_batch",
            &["CONFLUENCE_PAGE_ID", "CONFLUENCE_ATTACHMENT_FILE"],
            json!({"content_id": page, "file_paths": file_path, "comment": "Acceptance batch upload probe"}),
            false,
            false,
        ),
        row(
            "confluence_attachments",
            "confluence_delete_attachment",
            "confluence/attachments/read_only_delete",
            &["CONFLUENCE_ATTACHMENT_ID"],
            json!({"attachment_id": attachment}),
            true,
            true,
        ),
    ]
}

fn acceptance_run_suffix() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("{}-{millis}", std::process::id())
}

fn row(
    toolset: &'static str,
    tool: &'static str,
    path: &'static str,
    required_env: &'static [&'static str],
    arguments: Value,
    read_only: bool,
    expect_blocked: bool,
) -> Row {
    Row {
        toolset,
        tool,
        path,
        required_env,
        arguments,
        read_only,
        expect_blocked,
    }
}

async fn run_mcp(binary: &Path, env: &EnvMap) -> i32 {
    let issue = env_value(env, "JIRA_READ_ISSUE");
    let page = env_value(env, "CONFLUENCE_PAGE_ID");
    let rows = vec![
        McpRow {
            toolset: "mcp",
            tool: "tools/list",
            path: "mcp/stdio/tools_list",
            required_env: &[],
            arguments: None,
            transport: McpTransport::StdioList,
        },
        mcp_row(
            "jira_issues",
            "jira_get_issue",
            "mcp/stdio/jira_get_issue",
            &["JIRA_READ_ISSUE"],
            json!({"issue_key": issue, "fields": ["summary", "status"]}),
            McpTransport::Stdio,
        ),
        mcp_row(
            "confluence_pages",
            "confluence_get_page",
            "mcp/stdio/confluence_get_page",
            &["CONFLUENCE_PAGE_ID"],
            json!({"page_id": page, "include_metadata": true, "convert_to_markdown": true}),
            McpTransport::Stdio,
        ),
        mcp_row(
            "jira_issues",
            "jira_get_issue",
            "mcp/http/jira_get_issue",
            &["JIRA_READ_ISSUE"],
            json!({"issue_key": issue, "fields": ["summary", "status"]}),
            McpTransport::Http,
        ),
        mcp_row(
            "confluence_pages",
            "confluence_get_page",
            "mcp/http/confluence_get_page",
            &["CONFLUENCE_PAGE_ID"],
            json!({"page_id": page, "include_metadata": true, "convert_to_markdown": true}),
            McpTransport::Http,
        ),
    ];

    print_header();
    let mut exit_code = 0;
    for row in rows {
        let missing = missing_required_env(row.required_env, env);
        if !missing.is_empty() {
            blocked_row(row.toolset, row.tool, row.path, &missing);
            exit_code = exit_code.max(BLOCKED);
            continue;
        }

        let (status, blocker, evidence) = match row.transport {
            McpTransport::StdioList => match list_stdio_tools(binary, env) {
                Ok(names) => {
                    let missing_tools = ["jira_get_issue", "confluence_get_page"]
                        .into_iter()
                        .filter(|tool| !names.iter().any(|name| name == tool))
                        .collect::<Vec<_>>();
                    if missing_tools.is_empty() {
                        (
                            "ok".to_string(),
                            "none".to_string(),
                            "jira_get_issue,confluence_get_page".to_string(),
                        )
                    } else {
                        (
                            "failed".to_string(),
                            "tool_discovery_missing".to_string(),
                            missing_tools.join(","),
                        )
                    }
                }
                Err(error) => (
                    "failed".to_string(),
                    "runner_error".to_string(),
                    redact_text(&error, env).chars().take(180).collect(),
                ),
            },
            McpTransport::Stdio => match call_stdio_tool(
                binary,
                env,
                row.tool,
                row.arguments.as_ref().expect("stdio rows have arguments"),
                false,
            ) {
                Ok(response) => classify_response(&response, false, env),
                Err(error) => (
                    "failed".to_string(),
                    "runner_error".to_string(),
                    redact_text(&error, env).chars().take(180).collect(),
                ),
            },
            McpTransport::Http => match run_http_call(
                binary,
                env,
                row.tool,
                row.arguments.as_ref().expect("http rows have arguments"),
            )
            .await
            {
                Ok(response) => classify_response(&response, false, env),
                Err(error) => (
                    "failed".to_string(),
                    "runner_error".to_string(),
                    redact_text(&error, env).chars().take(180).collect(),
                ),
            },
        };

        print_row(
            row.toolset,
            row.tool,
            row.path,
            &status,
            &blocker,
            &evidence,
        );
        if status == "blocked" {
            exit_code = exit_code.max(BLOCKED);
        } else if status != "ok" {
            exit_code = exit_code.max(FAILED);
        }
    }

    exit_code
}

fn mcp_row(
    toolset: &'static str,
    tool: &'static str,
    path: &'static str,
    required_env: &'static [&'static str],
    arguments: Value,
    transport: McpTransport,
) -> McpRow {
    McpRow {
        toolset,
        tool,
        path,
        required_env,
        arguments: Some(arguments),
        transport,
    }
}

fn clean_env(env: &EnvMap, read_only: bool) -> EnvMap {
    let mut clean = env.clone();
    clean.remove("ENABLED_TOOLS");
    clean.insert("TOOLSETS".to_string(), "all".to_string());
    clean.insert(
        "READ_ONLY_MODE".to_string(),
        if read_only { "true" } else { "false" }.to_string(),
    );
    clean
}

fn call_stdio_tool(
    binary: &Path,
    env: &EnvMap,
    tool: &str,
    arguments: &Value,
    read_only: bool,
) -> Result<Value, String> {
    let mut session = StdioSession::start(binary, clean_env(env, read_only))?;
    session.initialize("acceptance")?;
    session.send(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": tool,
            "arguments": arguments,
        }
    }))?;
    session.read_response(2)
}

fn list_stdio_tools(binary: &Path, env: &EnvMap) -> Result<Vec<String>, String> {
    let mut session = StdioSession::start(binary, clean_env(env, false))?;
    session.initialize("acceptance-list")?;
    session.send(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    }))?;
    let response = session.read_response(2)?;
    Ok(response
        .get("result")
        .and_then(|result| result.get("tools"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|tool| tool.get("name").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect())
}

struct StdioSession {
    child: Child,
    stdin: std::process::ChildStdin,
    rx: mpsc::Receiver<Result<String, String>>,
}

impl StdioSession {
    fn start(binary: &Path, env: EnvMap) -> Result<Self, String> {
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
            return Err("initialize did not return result".to_string());
        }
        self.send(json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        }))
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

async fn run_http_call(
    binary: &Path,
    env: &EnvMap,
    tool: &str,
    arguments: &Value,
) -> Result<Value, String> {
    let port = free_port()?;
    let path = "/mcp";
    let env = clean_env(env, false);
    let mut child = Command::new(binary)
        .arg("streamhttp")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .arg("--path")
        .arg(path)
        .env_clear()
        .envs(env)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to start HTTP server: {error}"))?;

    let result = async {
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
                    "clientInfo": {"name": "http-acceptance", "version": "0.1.0"},
                }
            }),
            None,
        )
        .await?;
        let init = expect_rpc(&body, 1)?;
        if init.get("result").is_none() {
            return Err("HTTP initialize failed".to_string());
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
            json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized",
                "params": {}
            }),
            Some(&session_id),
        )
        .await?;
        let (_, body) = post_mcp(
            &client,
            port,
            path,
            json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": tool,
                    "arguments": arguments,
                }
            }),
            Some(&session_id),
        )
        .await?;
        expect_rpc(&body, 2)
    }
    .await;

    let _ = child.kill();
    let _ = child.wait();

    result
}

fn free_port() -> Result<u16, String> {
    let listener =
        TcpListener::bind("127.0.0.1:0").map_err(|error| format!("bind failed: {error}"))?;
    listener
        .local_addr()
        .map(|address| address.port())
        .map_err(|error| format!("local_addr failed: {error}"))
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
) -> Result<(HeaderMap, String), String> {
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
        return Err(format!("MCP HTTP {status}"));
    }
    Ok((headers, body))
}

fn expect_rpc(body: &str, expected_id: i64) -> Result<Value, String> {
    for message in parse_sse_messages(body)? {
        if message.get("id").and_then(Value::as_i64) == Some(expected_id) {
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

fn classify_response(
    response: &Value,
    expect_blocked: bool,
    env: &EnvMap,
) -> (String, String, String) {
    let response_text = serde_json::to_string(response).unwrap_or_default();
    if expect_blocked {
        if response_text.contains("read-only mode")
            || response_text.contains("disabled in read-only")
        {
            return (
                "ok".to_string(),
                "none".to_string(),
                "read_only_blocked_before_http".to_string(),
            );
        }
        return (
            "failed".to_string(),
            "read_only_not_blocked".to_string(),
            compact_error(response, env),
        );
    }
    if let Some(error) = response.get("error") {
        return (
            "failed".to_string(),
            "json_rpc_error".to_string(),
            compact_error(error, env),
        );
    }
    let result = response.get("result").unwrap_or(&Value::Null);
    if result.get("isError").and_then(Value::as_bool) == Some(true) {
        return (
            "failed".to_string(),
            "tool_error".to_string(),
            compact_error(result, env),
        );
    }
    if product_dependency_unavailable(response) {
        return (
            "blocked".to_string(),
            "blocked:product_unavailable".to_string(),
            format!("structured:{}", structured_keys(response)),
        );
    }
    (
        "ok".to_string(),
        "none".to_string(),
        format!("structured:{}", structured_keys(response)),
    )
}

fn product_dependency_unavailable(response: &Value) -> bool {
    let Some(structured) = response
        .get("result")
        .and_then(|result| result.get("structuredContent"))
        .and_then(Value::as_object)
    else {
        return false;
    };

    if structured
        .get("product_dependency")
        .and_then(Value::as_object)
        .and_then(|product| product.get("available"))
        .and_then(Value::as_bool)
        == Some(false)
    {
        return true;
    }
    structured.get("available").and_then(Value::as_bool) == Some(false)
        && structured.contains_key("product")
}

fn structured_keys(response: &Value) -> String {
    let structured = response
        .get("result")
        .and_then(|result| result.get("structuredContent"));
    match structured {
        Some(Value::Object(map)) => {
            let keys = map.keys().take(8).cloned().collect::<Vec<_>>();
            if keys.is_empty() {
                "empty".to_string()
            } else {
                keys.join(",")
            }
        }
        Some(value) => value_type(value).to_string(),
        None => "null".to_string(),
    }
}

fn value_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn compact_error(value: &Value, env: &EnvMap) -> String {
    let text = serde_json::to_string(value).unwrap_or_default();
    redact_text(&text, env).chars().take(180).collect()
}

fn redact_text(text: &str, env_map: &EnvMap) -> String {
    let file_secrets =
        env_secret_values_from_pairs(env_map.iter().map(|(key, value)| (key.as_str(), value)));
    let process_secrets = env_secret_values_from_pairs(
        SECRET_KEYS
            .iter()
            .filter_map(|key| env::var(key).ok().map(|value| (*key, value))),
    );

    redact_text_with_secrets(text, file_secrets.into_iter().chain(process_secrets))
}

#[cfg(test)]
mod tests;
