use super::*;

#[test]
fn parses_acceptance_preflight_args() {
    assert_eq!(
        parse_acceptance_args(&["jira".to_string(), "--preflight".to_string()]).unwrap(),
        AcceptanceCommand {
            mode: AcceptanceMode::Jira,
            action: AcceptanceAction::Preflight,
            env_file: None,
        }
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
