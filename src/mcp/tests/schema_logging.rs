use super::support::*;
use super::*;
use serde_json::Map;

#[test]
fn tool_log_arguments_redact_sensitive_fields_and_truncate_large_values() {
    let long_value = "x".repeat(TOOL_LOG_MAX_STRING_CHARS + 1);
    let mut nested = Map::new();
    nested.insert("password".to_string(), json!("secret-password"));
    let mut arguments = Map::new();
    arguments.insert("jql".to_string(), json!("project = ABC"));
    arguments.insert("page_token".to_string(), json!("visible-page-token"));
    arguments.insert("api_token".to_string(), json!("test-api-token"));
    arguments.insert("nested".to_string(), Value::Object(nested));
    arguments.insert("description".to_string(), Value::String(long_value));
    arguments.insert(
        "callback".to_string(),
        json!("Authorization Bearer callback-secret /path?token=query-secret&client=abc"),
    );

    let sanitized = sanitize_tool_log_arguments(Some(&arguments));

    assert_eq!(sanitized["jql"], "project = ABC");
    assert_eq!(sanitized["page_token"], "visible-page-token");
    assert_eq!(sanitized["api_token"], TOOL_LOG_REDACTED);
    assert_eq!(sanitized["nested"]["password"], TOOL_LOG_REDACTED);
    let description = sanitized["description"].as_str().unwrap();
    assert!(description.ends_with(TOOL_LOG_TRUNCATED));
    let callback = sanitized["callback"].as_str().unwrap();
    assert!(callback.contains("Bearer <redacted>"));
    assert!(callback.contains("token=<redacted>"));
    assert!(!sanitized.to_string().contains("test-api-token"));
    assert!(!sanitized.to_string().contains("secret-password"));
    assert!(!sanitized.to_string().contains("callback-secret"));
    assert!(!sanitized.to_string().contains("query-secret"));
}

#[test]
fn atlassian_error_preserves_http_status_and_redacts_sensitive_values() {
    for status in [401, 403, 404, 429] {
        let error = atlassian_error(AtlassianError::HttpStatus {
                status,
                message: "failed https://example.atlassian.net/rest/api?token=secret with Authorization: Bearer abc123"
                    .to_string(),
            });

        assert!(error.message.contains("category=http_status"));
        assert!(error.message.contains(&format!("status={status}")));
        assert!(!error.message.contains("token=secret"));
        assert!(error.message.contains("token=<redacted>"));
        assert!(!error.message.contains("abc123"));
        assert!(error.message.contains("Bearer <redacted>"));
    }
}

#[test]
fn atlassian_error_reports_non_http_categories() {
    let invalid_base = atlassian_error(AtlassianError::invalid_base_url(
        "bad base https://example.atlassian.net?password=secret",
    ));
    let json_decode = atlassian_error(AtlassianError::JsonDecode {
        message: "expected JSON object".to_string(),
    });
    let unexpected_shape = atlassian_error(AtlassianError::unexpected_shape(
        "missing field customfield_10000",
    ));

    assert!(invalid_base.message.contains("category=invalid_base_url"));
    assert!(!invalid_base.message.contains("password=secret"));
    assert!(invalid_base.message.contains("password=<redacted>"));
    assert!(json_decode.message.contains("category=json_decode"));
    assert!(json_decode.message.contains("expected JSON object"));
    assert!(
        unexpected_shape
            .message
            .contains("category=unexpected_shape")
    );
    assert!(
        unexpected_shape
            .message
            .contains("missing field customfield_10000")
    );
}

#[test]
fn default_jira_tool_schemas_are_client_compatible() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from([
            "jira_issue_read".to_string(),
            "jira_issue_write".to_string(),
            "jira_fields_read".to_string(),
            "jira_comments_write".to_string(),
            "jira_workflow_read".to_string(),
            "jira_workflow_write".to_string(),
        ]),
        ..runtime_config()
    });
    let tools = server.current_tools_result().tools;
    let names = tool_names(tools.clone());

    assert!(names.contains(&tools::JIRA_GET_ISSUE_TOOL_NAME.to_string()));
    assert!(names.contains(&tools::JIRA_SEARCH_TOOL_NAME.to_string()));
    assert_client_compatible_tool_schemas(&tools);
}

#[test]
fn all_jira_tool_schemas_are_client_compatible() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config()
    });
    let tools = server.current_tools_result().tools;
    let names = tool_names(tools.clone());

    assert!(names.contains(&tools::JIRA_GET_ISSUE_TOOL_NAME.to_string()));
    assert!(names.contains(&tools::JIRA_GET_ISSUE_SLA_TOOL_NAME.to_string()));
    assert_client_compatible_tool_schemas(&tools);
    assert_tool_schema_lacks_property(&tools, tools::JIRA_SEARCH_FIELDS_TOOL_NAME, "refresh");
    assert_tool_schema_lacks_property(
        &tools,
        tools::JIRA_GET_ISSUE_SLA_TOOL_NAME,
        concat!("working_hours", "_only"),
    );
}

#[test]
fn confluence_content_toolsets_have_client_compatible_schemas() {
    let read_write = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_toolsets: BTreeSet::from([
            "confluence_content_read".to_string(),
            "confluence_content_write".to_string(),
            "confluence_comments_read".to_string(),
            "confluence_comments_write".to_string(),
        ]),
        ..runtime_config()
    });
    let read_write_tools = read_write.current_tools_result().tools;
    let read_write_names = tool_names(read_write_tools.clone());

    assert!(read_write_names.contains(&confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME.to_string()));
    assert!(
        read_write_names.contains(&confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME.to_string())
    );
    assert!(
        read_write_names.contains(&confluence_tools::CONFLUENCE_ADD_COMMENT_TOOL_NAME.to_string())
    );
    assert_client_compatible_tool_schemas(&read_write_tools);
    assert_tool_schema_lacks_property(
        &read_write_tools,
        confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME,
        concat!("enable_heading", "_anchors"),
    );
    assert_tool_schema_lacks_property(
        &read_write_tools,
        confluence_tools::CONFLUENCE_UPDATE_PAGE_TOOL_NAME,
        concat!("enable_heading", "_anchors"),
    );
}
