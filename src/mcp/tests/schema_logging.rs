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
fn upstream_error_preserves_http_status_and_redacts_sensitive_values() {
    for status in [401, 403, 404, 429] {
        let error = upstream_error(UpstreamError::HttpStatus {
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
fn upstream_error_reports_non_http_categories() {
    let invalid_base = upstream_error(UpstreamError::invalid_base_url(
        "bad base https://example.atlassian.net?password=secret",
    ));
    let json_decode = upstream_error(UpstreamError::JsonDecode {
        message: "expected JSON object".to_string(),
    });
    let unexpected_shape = upstream_error(UpstreamError::unexpected_shape(
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
fn high_risk_schema_baseline_covers_existing_tools_fields_and_payloads() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        confluence: Some(confluence_config()),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config()
    });
    let tools = server.current_tools_result().tools;
    let names = tool_names(tools.clone())
        .into_iter()
        .collect::<BTreeSet<_>>();

    assert!(high_risk_schema_tool_names().len() >= 40);
    for name in high_risk_schema_tool_names() {
        assert!(
            names.contains(name),
            "{name} should be in the high-risk tool baseline"
        );
    }

    assert!(high_risk_input_fields().len() >= 20);
    for field in high_risk_input_fields() {
        let tool = tools
            .iter()
            .find(|tool| tool.name == field.tool_name)
            .unwrap_or_else(|| panic!("{} should be discoverable", field.tool_name));
        let properties = tool
            .input_schema
            .get("properties")
            .and_then(Value::as_object)
            .unwrap_or_else(|| panic!("{} should expose an object schema", field.tool_name));

        assert!(
            properties.contains_key(field.field_name),
            "{} should expose high-risk field {} ({})",
            field.tool_name,
            field.field_name,
            field.reason
        );
        let description = properties
            .get(field.field_name)
            .and_then(|schema| schema.get("description"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim();
        assert!(
            !description.is_empty(),
            "{} high-risk field {} should expose a schema description ({})",
            field.tool_name,
            field.field_name,
            field.reason
        );
        assert!(!field.reason.is_empty());
    }

    assert!(high_risk_output_tools().len() >= 10);
    for output in high_risk_output_tools() {
        let tool = tools
            .iter()
            .find(|tool| tool.name == output.tool_name)
            .unwrap_or_else(|| panic!("{} should be discoverable", output.tool_name));
        let output_schema = tool
            .output_schema
            .as_ref()
            .unwrap_or_else(|| panic!("{} should expose output schema", output.tool_name));
        assert_eq!(
            output_schema.get("type").and_then(Value::as_str),
            Some("object"),
            "{} output schema should describe structuredContent root object",
            output.tool_name
        );
        assert!(
            output_schema
                .get("description")
                .and_then(Value::as_str)
                .is_some_and(|description| !description.trim().is_empty()),
            "{} output schema should have a description",
            output.tool_name
        );
        assert!(
            output_schema
                .get("properties")
                .and_then(Value::as_object)
                .is_some_and(|properties| !properties.is_empty()),
            "{} output schema should describe stable payload keys",
            output.tool_name
        );
        assert!(
            names.contains(output.tool_name),
            "{} should be in the high-risk output baseline",
            output.tool_name
        );
        assert!(!output.reason.is_empty());
    }
}

#[test]
fn high_risk_output_schemas_declare_representative_payload_keys() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        confluence: Some(confluence_config()),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config()
    });
    let discovered_tools = server.current_tools_result().tools;

    for (tool_name, expected_properties) in [
        (tools::JIRA_CREATE_ISSUES_TOOL_NAME, vec!["success", "data"]),
        (
            tools::JIRA_CREATE_PROJECT_VERSIONS_TOOL_NAME,
            vec!["versions"],
        ),
        (tools::JIRA_UPDATE_ISSUE_TOOL_NAME, vec!["success", "data"]),
        (tools::JIRA_DELETE_ISSUE_TOOL_NAME, vec!["success", "data"]),
        (
            tools::JIRA_GET_ISSUE_ATTACHMENTS_TOOL_NAME,
            vec!["issue_key", "count", "attachments"],
        ),
        (
            tools::JIRA_GET_ISSUE_IMAGES_TOOL_NAME,
            vec!["issue_key", "count", "images_only", "attachments"],
        ),
        (
            tools::JIRA_LIST_AGILE_BOARDS_TOOL_NAME,
            vec!["success", "product_dependency"],
        ),
        (
            confluence_tools::CONFLUENCE_UPLOAD_CONTENT_ATTACHMENTS_TOOL_NAME,
            vec![
                "success",
                "partial_success",
                "summary",
                "attachments",
                "failed",
            ],
        ),
        (
            confluence_tools::CONFLUENCE_DOWNLOAD_ATTACHMENT_TOOL_NAME,
            vec!["success", "attachment", "error"],
        ),
        (
            confluence_tools::CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_TOOL_NAME,
            vec!["success", "summary", "attachments", "failed"],
        ),
        (
            confluence_tools::CONFLUENCE_GET_PAGE_VIEW_ANALYTICS_TOOL_NAME,
            vec!["success", "available", "page_id", "total_views", "error"],
        ),
    ] {
        let tool = discovered_tools
            .iter()
            .find(|tool| tool.name == tool_name)
            .unwrap_or_else(|| panic!("{tool_name} should be discoverable"));
        let properties = tool
            .output_schema
            .as_ref()
            .unwrap_or_else(|| panic!("{tool_name} should expose output schema"))
            .get("properties")
            .and_then(Value::as_object)
            .unwrap_or_else(|| panic!("{tool_name} output schema should expose properties"));

        for property in expected_properties {
            assert!(
                properties.contains_key(property),
                "{tool_name} output schema should declare {property}"
            );
        }
    }
}

#[test]
fn output_schema_sanitizer_preserves_boolean_const_literals() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config()
    });
    let discovered_tools = server.current_tools_result().tools;

    for (tool_name, expected_const) in [
        (tools::JIRA_GET_ISSUE_ATTACHMENTS_TOOL_NAME, false),
        (tools::JIRA_GET_ISSUE_IMAGES_TOOL_NAME, true),
    ] {
        let tool = discovered_tools
            .iter()
            .find(|tool| tool.name == tool_name)
            .unwrap_or_else(|| panic!("{tool_name} should be discoverable"));
        let images_only = tool
            .output_schema
            .as_ref()
            .unwrap_or_else(|| panic!("{tool_name} should expose output schema"))
            .get("properties")
            .and_then(Value::as_object)
            .and_then(|properties| properties.get("images_only"))
            .and_then(Value::as_object)
            .unwrap_or_else(|| panic!("{tool_name} should describe images_only"));

        assert_eq!(
            images_only.get("type").and_then(Value::as_str),
            Some("boolean")
        );
        assert_eq!(images_only.get("const"), Some(&json!(expected_const)));
    }
}

#[test]
fn jira_mutation_output_schema_allows_no_content_data_null() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config()
    });
    let discovered_tools = server.current_tools_result().tools;

    for tool_name in [
        tools::JIRA_UPDATE_ISSUE_TOOL_NAME,
        tools::JIRA_DELETE_ISSUE_TOOL_NAME,
    ] {
        let tool = discovered_tools
            .iter()
            .find(|tool| tool.name == tool_name)
            .unwrap_or_else(|| panic!("{tool_name} should be discoverable"));
        let any_of = tool
            .output_schema
            .as_ref()
            .unwrap_or_else(|| panic!("{tool_name} should expose output schema"))
            .get("properties")
            .and_then(Value::as_object)
            .and_then(|properties| properties.get("data"))
            .and_then(|data| data.get("anyOf"))
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("{tool_name} data schema should expose anyOf"));

        assert!(
            any_of
                .iter()
                .any(|schema| schema.get("type").and_then(Value::as_str) == Some("null")),
            "{tool_name} data schema should allow null for no-content responses"
        );
    }
}

#[test]
fn default_jira_tool_schemas_are_client_compatible() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        mcp_enabled_toolsets: BTreeSet::from([
            "jira_issues_read".to_string(),
            "jira_issues_write".to_string(),
            "jira_fields_read".to_string(),
            "jira_issue_comments_write".to_string(),
            "jira_issue_workflows_read".to_string(),
            "jira_issue_workflows_write".to_string(),
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
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config()
    });
    let tools = server.current_tools_result().tools;
    let names = tool_names(tools.clone());

    assert!(names.contains(&tools::JIRA_GET_ISSUE_TOOL_NAME.to_string()));
    assert!(names.contains(&tools::JIRA_GET_ISSUE_SLA_METRICS_TOOL_NAME.to_string()));
    assert_client_compatible_tool_schemas(&tools);
    assert_tool_schema_lacks_property(&tools, tools::JIRA_SEARCH_FIELDS_TOOL_NAME, "refresh");
    assert_tool_schema_lacks_property(
        &tools,
        tools::JIRA_GET_ISSUE_SLA_METRICS_TOOL_NAME,
        concat!("working_hours", "_only"),
    );
}

#[test]
fn confluence_content_toolsets_have_client_compatible_schemas() {
    let read_write = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config()),
        mcp_enabled_toolsets: BTreeSet::from([
            "confluence_content_read".to_string(),
            "confluence_content_write".to_string(),
            "confluence_content_update".to_string(),
            "confluence_page_comments_read".to_string(),
            "confluence_page_comments_write".to_string(),
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
