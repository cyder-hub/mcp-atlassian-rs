use super::support::*;
use super::*;

#[test]
fn list_tools_uses_request_scoped_service_headers_without_mutating_global_context() {
    let server = server_with_config(runtime_config());
    let headers = request_service_headers();

    let scoped_server = server.scoped_for_request_headers(&headers).unwrap();
    let names = current_tool_names(&scoped_server);

    assert!(names.contains(&tools::JIRA_GET_ISSUE_TOOL_NAME.to_string()));
    assert!(names.contains(&confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME.to_string()));
    assert!(current_tool_names(&server).is_empty());
}

#[test]
fn session_auth_fingerprint_allows_stable_request_auth() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        ..runtime_config()
    });
    let headers = header_map(&[
        ("Mcp-Session-Id", "session-1"),
        ("Authorization", "Bearer stable-request-token"),
    ]);

    server.scoped_for_request_headers(&headers).unwrap();
    let scoped_server = server.scoped_for_request_headers(&headers).unwrap();

    assert!(
        current_tool_names(&scoped_server).contains(&tools::JIRA_GET_ISSUE_TOOL_NAME.to_string())
    );
}

#[test]
fn session_auth_fingerprint_rejects_changed_request_auth() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        ..runtime_config()
    });
    let first = header_map(&[
        ("Mcp-Session-Id", "session-1"),
        ("Authorization", "Bearer first-request-token"),
    ]);
    let changed = header_map(&[
        ("Mcp-Session-Id", "session-1"),
        ("Authorization", "Bearer second-request-token"),
    ]);
    let different_session = header_map(&[
        ("Mcp-Session-Id", "session-2"),
        ("Authorization", "Bearer second-request-token"),
    ]);

    server.scoped_for_request_headers(&first).unwrap();
    let error = match server.scoped_for_request_headers(&changed) {
        Ok(_) => panic!("changed request auth should be rejected"),
        Err(error) => error,
    };

    assert_eq!(
        error.message.as_ref(),
        "per-request authentication changed for MCP session"
    );
    assert!(!error.message.contains("first-request-token"));
    assert!(!error.message.contains("second-request-token"));
    server
        .scoped_for_request_headers(&different_session)
        .unwrap();
}

#[test]
fn request_auth_session_fingerprint_rejects_changed_token_type() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        atlassian_oauth_enabled: true,
        ..runtime_config()
    });
    let bearer = header_map(&[
        ("Mcp-Session-Id", "session-token-type"),
        ("Authorization", "Bearer shared-request-secret"),
    ]);
    let token = header_map(&[
        ("Mcp-Session-Id", "session-token-type"),
        ("Authorization", "Token shared-request-secret"),
    ]);

    server.scoped_for_request_headers(&bearer).unwrap();
    let error = match server.scoped_for_request_headers(&token) {
        Ok(_) => panic!("changed request auth token type should be rejected"),
        Err(error) => error,
    };

    assert_eq!(
        error.message.as_ref(),
        "per-request authentication changed for MCP session"
    );
    assert!(!error.message.contains("shared-request-secret"));
}

#[test]
fn session_auth_store_binds_initialize_response_fingerprint() {
    let context = AppContext::from_config(&RuntimeConfig {
        jira: Some(jira_config()),
        ..runtime_config()
    });
    let store = RequestAuthSessionStore::default();
    let init_headers = header_map(&[("Authorization", "Bearer init-request-token")]);
    let response_headers = header_map(&[("Mcp-Session-Id", "session-from-init")]);

    let fingerprint = store
        .parse_and_enforce_headers(&init_headers, &context)
        .unwrap();
    store
        .bind_response_headers(&response_headers, &fingerprint)
        .unwrap();

    let stable_headers = header_map(&[
        ("Mcp-Session-Id", "session-from-init"),
        ("Authorization", "Bearer init-request-token"),
    ]);
    store
        .parse_and_enforce_headers(&stable_headers, &context)
        .unwrap();

    let changed_headers = header_map(&[
        ("Mcp-Session-Id", "session-from-init"),
        ("Authorization", "Bearer changed-request-token"),
    ]);
    let error = store
        .parse_and_enforce_headers(&changed_headers, &context)
        .unwrap_err();

    assert_eq!(
        error.message.as_ref(),
        "per-request authentication changed for MCP session"
    );
    assert!(!error.message.contains("init-request-token"));
    assert!(!error.message.contains("changed-request-token"));
}

#[test]
fn request_auth_matrix_accepts_basic_token_and_bearer_at_mcp_boundary() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        confluence: Some(confluence_config()),
        ..runtime_config()
    });

    let basic = server
        .scoped_for_request_headers(&header_map(&[(
            "Authorization",
            "Basic dXNlckBleGFtcGxlLmNvbTphcGktdG9rZW4=",
        )]))
        .unwrap();
    assert_eq!(
        basic.context.jira_config().unwrap().auth,
        AtlassianAuth::Basic {
            username: "user@example.com".to_string(),
            api_token: "api-token".to_string(),
        }
    );
    assert_eq!(
        basic.context.confluence_config().unwrap().auth,
        AtlassianAuth::Basic {
            username: "user@example.com".to_string(),
            api_token: "api-token".to_string(),
        }
    );

    for (scheme, token) in [
        ("Token request-token-value", "request-token-value"),
        ("Bearer request-bearer-value", "request-bearer-value"),
    ] {
        let scoped = server
            .scoped_for_request_headers(&header_map(&[("Authorization", scheme)]))
            .unwrap();
        assert_eq!(
            scoped.context.jira_config().unwrap().auth,
            AtlassianAuth::Pat {
                personal_token: token.to_string(),
            }
        );
    }
}

#[test]
fn request_auth_matrix_rejects_bad_headers_without_echoing_credentials() {
    let server = server_with_config(runtime_config());

    let missing_pair = match server.scoped_for_request_headers(&header_map(&[(
        "X-Atlassian-Jira-Url",
        "https://example.com",
    )])) {
        Ok(_) => panic!("missing Jira token should be rejected"),
        Err(error) => error,
    };
    assert_eq!(
        missing_pair.message.as_ref(),
        "missing Jira URL/token header pair"
    );

    let unsupported = match server.scoped_for_request_headers(&header_map(&[(
        "Authorization",
        "Digest reject-this-secret",
    )])) {
        Ok(_) => panic!("unsupported Authorization scheme should be rejected"),
        Err(error) => error,
    };
    assert_eq!(
        unsupported.message.as_ref(),
        "unsupported Authorization scheme `Digest`"
    );
    assert!(!unsupported.message.contains("reject-this-secret"));
}

#[test]
fn request_auth_matrix_respects_ignore_header_auth_and_control_plane() {
    let ignore_headers = server_with_config(RuntimeConfig {
        ignore_header_auth: true,
        ..runtime_config()
    });
    let ignored = ignore_headers
        .scoped_for_request_headers(&request_service_headers())
        .unwrap();
    assert!(current_tool_names(&ignored).is_empty());

    let basic = server_with_config(RuntimeConfig { ..runtime_config() });
    let basic = basic
        .scoped_for_request_headers(&request_service_headers())
        .unwrap();
    let basic_names = current_tool_names(&basic);
    assert!(basic_names.contains(&tools::JIRA_GET_ISSUE_TOOL_NAME.to_string()));
    assert!(basic_names.contains(&tools::JIRA_CREATE_ISSUE_TOOL_NAME.to_string()));
    assert!(basic_names.contains(&confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME.to_string()));
    assert!(!basic_names.contains(&confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME.to_string()));

    let filtered = server_with_config(RuntimeConfig {
        enabled_tools: Some(BTreeSet::from(
            [tools::JIRA_GET_ISSUE_TOOL_NAME.to_string()],
        )),
        enabled_toolsets: BTreeSet::new(),
        ..runtime_config()
    });
    let filtered = filtered
        .scoped_for_request_headers(&request_service_headers())
        .unwrap();
    assert_eq!(
        current_tool_names(&filtered),
        vec![tools::JIRA_GET_ISSUE_TOOL_NAME.to_string()]
    );
}

#[test]
fn request_auth_byot_matrix_preserves_profile_filters_and_service_availability() {
    let byot_headers = header_map(&[
        ("Authorization", "Bearer request-access-token"),
        ("X-Atlassian-Cloud-Id", "cloud-123"),
    ]);
    let basic = server_with_config(RuntimeConfig {
        jira: Some(jira_cloud_config_with_base_url(
            "https://example.atlassian.net".to_string(),
        )),
        confluence: Some(confluence_cloud_config_with_base_url(
            "https://example.atlassian.net/wiki".to_string(),
        )),
        ..runtime_config()
    });
    let basic = basic.scoped_for_request_headers(&byot_headers).unwrap();
    let basic_names = current_tool_names(&basic);

    assert_eq!(
        basic.context.jira_config().unwrap().auth,
        AtlassianAuth::OAuthAccessToken {
            access_token: "request-access-token".to_string(),
        }
    );
    assert_eq!(
        basic.context.jira_config().unwrap().base_url,
        "https://api.atlassian.com/ex/jira/cloud-123"
    );
    assert_eq!(
        basic.context.confluence_config().unwrap().base_url,
        "https://api.atlassian.com/ex/confluence/cloud-123/wiki"
    );
    assert!(basic_names.contains(&tools::JIRA_GET_ISSUE_TOOL_NAME.to_string()));
    assert!(basic_names.contains(&tools::JIRA_CREATE_ISSUE_TOOL_NAME.to_string()));
    assert!(basic_names.contains(&confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME.to_string()));
    assert!(!basic_names.contains(&confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME.to_string()));

    let filtered = server_with_config(RuntimeConfig {
        jira: Some(jira_cloud_config_with_base_url(
            "https://example.atlassian.net".to_string(),
        )),
        confluence: Some(confluence_cloud_config_with_base_url(
            "https://example.atlassian.net/wiki".to_string(),
        )),
        enabled_tools: Some(BTreeSet::from(
            [tools::JIRA_GET_ISSUE_TOOL_NAME.to_string()],
        )),
        enabled_toolsets: BTreeSet::new(),
        ..runtime_config()
    });
    let filtered = filtered.scoped_for_request_headers(&byot_headers).unwrap();

    assert_eq!(
        current_tool_names(&filtered),
        vec![tools::JIRA_GET_ISSUE_TOOL_NAME.to_string()]
    );
}

#[test]
fn request_auth_byot_matrix_keeps_token_scheme_as_pat_when_oauth_enabled() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        confluence: Some(confluence_config()),
        atlassian_oauth_enabled: true,
        ..runtime_config()
    });
    let scoped = server
        .scoped_for_request_headers(&header_map(&[("Authorization", "Token request-pat-token")]))
        .unwrap();

    assert_eq!(
        scoped.context.jira_config().unwrap().auth,
        AtlassianAuth::Pat {
            personal_token: "request-pat-token".to_string(),
        }
    );
    assert_eq!(
        scoped.context.confluence_config().unwrap().auth,
        AtlassianAuth::Pat {
            personal_token: "request-pat-token".to_string(),
        }
    );
}
