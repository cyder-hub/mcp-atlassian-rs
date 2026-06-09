use super::*;
use clap::Parser;

#[derive(Parser)]
struct AcceptanceTestCli {
    #[command(flatten)]
    command: AcceptanceCommand,
}

#[test]
fn parses_acceptance_preflight_args() {
    let command = AcceptanceTestCli::try_parse_from(["acceptance", "jira", "--preflight"])
        .unwrap()
        .command;
    assert_eq!(command.mode, AcceptanceMode::Jira);
    assert_eq!(command.action(), AcceptanceAction::Preflight);
    assert_eq!(command.env_file, None);
}

#[test]
fn parses_acceptance_env_file_args() {
    let command = AcceptanceTestCli::try_parse_from([
        "acceptance",
        "confluence",
        "--env-file=.env.acceptance",
        "--preflight",
    ])
    .unwrap()
    .command;
    assert_eq!(command.mode, AcceptanceMode::Confluence);
    assert_eq!(command.action(), AcceptanceAction::Preflight);
    assert_eq!(command.env_file, Some(PathBuf::from(".env.acceptance")));
}

#[test]
fn rejects_missing_acceptance_action() {
    assert!(AcceptanceTestCli::try_parse_from(["acceptance", "jira"]).is_err());
}

#[test]
fn default_env_file_is_env_dev() {
    assert_eq!(
        default_env_file()
            .file_name()
            .and_then(|value| value.to_str()),
        Some(".env.dev")
    );
}

#[test]
fn parses_dotenv_values() {
    assert_eq!(
        parse_env_line(
            "export JIRA_URL='https://example.atlassian.net' # comment",
            Path::new(".env"),
            1
        )
        .unwrap(),
        Some((
            "JIRA_URL".to_string(),
            "https://example.atlassian.net".to_string()
        ))
    );
    assert_eq!(
        parse_env_line("SAMPLE_VALUE=\"abc#def\"", Path::new(".env"), 2).unwrap(),
        Some(("SAMPLE_VALUE".to_string(), "abc#def".to_string()))
    );
}

#[test]
fn preflight_reports_missing_base_env() {
    let env = EnvMap::new();
    assert_eq!(
        missing_base_env(&AcceptanceMode::Mcp, &env),
        vec!["JIRA_URL", "JIRA_AUTH", "CONFLUENCE_URL", "CONFLUENCE_AUTH"]
    );
}

#[test]
fn preflight_accepts_shared_basic_auth_for_both_services() {
    let mut env = EnvMap::new();
    env.insert(
        "JIRA_URL".to_string(),
        "https://example.atlassian.net".to_string(),
    );
    env.insert(
        "CONFLUENCE_URL".to_string(),
        "https://example.atlassian.net/wiki".to_string(),
    );
    env.insert(
        "ATLASSIAN_USERNAME".to_string(),
        "user@example.com".to_string(),
    );
    env.insert(
        "ATLASSIAN_API_TOKEN".to_string(),
        "shared-api-token".to_string(),
    );

    assert!(missing_base_env(&AcceptanceMode::Mcp, &env).is_empty());
}

#[test]
fn preflight_accepts_shared_token_auth_for_each_service() {
    let mut jira_env = EnvMap::new();
    jira_env.insert("JIRA_URL".to_string(), "https://jira.example".to_string());
    jira_env.insert(
        "ATLASSIAN_PERSONAL_TOKEN".to_string(),
        "shared-pat-value".to_string(),
    );
    assert!(missing_base_env(&AcceptanceMode::Jira, &jira_env).is_empty());

    let mut confluence_env = EnvMap::new();
    confluence_env.insert(
        "CONFLUENCE_URL".to_string(),
        "https://confluence.example".to_string(),
    );
    confluence_env.insert(
        "ATLASSIAN_PERSONAL_TOKEN".to_string(),
        "shared-pat-value".to_string(),
    );
    assert!(missing_base_env(&AcceptanceMode::Confluence, &confluence_env).is_empty());
}

#[test]
fn redacts_file_loaded_secrets() {
    let mut env = EnvMap::new();
    env.insert("JIRA_API_TOKEN".to_string(), "secret-token".to_string());
    assert_eq!(
        redact_text("Authorization secret-token", &env),
        "Authorization <redacted>"
    );
}

#[test]
fn compact_error_redacts_auth_fragments_and_query_tokens() {
    let env = EnvMap::new();
    let value = json!({
        "error": "Authorization Bearer bearer-secret failed /path?token=query-secret&client=abc"
    });
    let output = compact_error(&value, &env);

    assert!(output.contains("Bearer <redacted>"));
    assert!(output.contains("token=<redacted>"));
    assert!(!output.contains("bearer-secret"));
    assert!(!output.contains("query-secret"));
}
