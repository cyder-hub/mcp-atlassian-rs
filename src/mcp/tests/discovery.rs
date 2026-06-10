use super::support::*;
use super::*;

#[test]
fn server_info_advertises_tools() {
    let info = AtlassianMcpServer::default().get_info();

    assert_eq!(info.server_info.name, SERVER_NAME);
    assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));
    assert!(info.capabilities.tools.is_some());
    assert!(info.capabilities.prompts.is_none());
    assert!(info.capabilities.resources.is_none());
}

#[test]
fn server_info_uses_app_context() {
    let config = RuntimeConfig {
        ..RuntimeConfig::default()
    };
    let server = AtlassianMcpServer::new(Arc::new(AppContext::from_config(&config)));
    let info = server.get_info();
    let instructions = info.instructions.unwrap_or_default();

    assert!(instructions.contains("TOOL_PROFILE"));
    assert!(instructions.contains("88 Jira, Confluence, and GitLab business tools"));
    assert!(instructions.contains("docs/support-matrix.md"));
}

#[test]
fn tool_discovery_has_no_tools_without_service_config() {
    let server = AtlassianMcpServer::default();

    assert!(current_tool_names(&server).is_empty());
    assert!(server.get_tool(tools::JIRA_GET_ISSUE_TOOL_NAME).is_none());
}

#[test]
fn tool_discovery_uses_public_names_not_handler_method_names() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        confluence: Some(confluence_config()),
        gitlab: Some(gitlab_config()),
        enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config()
    });

    assert!(server.get_tool(tools::JIRA_SEARCH_TOOL_NAME).is_some());
    assert!(
        server
            .get_tool(confluence_tools::CONFLUENCE_LIST_PAGE_COMMENTS_TOOL_NAME)
            .is_some()
    );
    assert!(
        server
            .get_tool(gitlab_tools::GITLAB_GET_MERGE_REQUEST_TOOL_NAME)
            .is_some()
    );
    assert!(server.get_tool("search_issues").is_none());
    assert!(server.get_tool("list_page_comments").is_none());
    assert!(server.get_tool("gitlab_get_merge_request").is_some());
}

#[test]
fn tool_discovery_applies_gitlab_service_profile_and_disabled_filters() {
    let unavailable = server_with_config(RuntimeConfig {
        enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config()
    });
    assert!(
        unavailable
            .get_tool(gitlab_tools::GITLAB_GET_PROJECT_TOOL_NAME)
            .is_none()
    );
    assert!(
        unavailable
            .guard_registered_tool_call(gitlab_tools::GITLAB_GET_PROJECT_TOOL_NAME)
            .is_err()
    );

    let basic = server_with_config(RuntimeConfig {
        gitlab: Some(gitlab_config()),
        disabled_tools: BTreeSet::from([
            gitlab_tools::GITLAB_GET_MERGE_REQUEST_TOOL_NAME.to_string()
        ]),
        ..runtime_config()
    });
    let basic_names = current_tool_names(&basic);

    assert!(basic_names.contains(&gitlab_tools::GITLAB_GET_PROJECT_TOOL_NAME.to_string()));
    assert!(!basic_names.contains(&gitlab_tools::GITLAB_GET_MERGE_REQUEST_TOOL_NAME.to_string()));
    assert!(
        !basic_names.contains(&gitlab_tools::GITLAB_CREATE_MERGE_REQUEST_TOOL_NAME.to_string())
    );
    assert!(
        !basic_names.contains(&gitlab_tools::GITLAB_ACCEPT_MERGE_REQUEST_TOOL_NAME.to_string())
    );
    assert_eq!(
        basic
            .guard_registered_tool_call(gitlab_tools::GITLAB_GET_MERGE_REQUEST_TOOL_NAME)
            .unwrap_err()
            .message,
        "tool not available"
    );

    let developer = server_with_config(RuntimeConfig {
        gitlab: Some(gitlab_config()),
        enabled_toolsets: tool_registry::toolsets_for_profile("developer")
            .unwrap()
            .iter()
            .map(|toolset| (*toolset).to_string())
            .collect(),
        ..runtime_config()
    });
    let developer_names = current_tool_names(&developer);

    assert!(
        developer_names.contains(&gitlab_tools::GITLAB_CREATE_MERGE_REQUEST_TOOL_NAME.to_string())
    );
    assert!(
        developer_names.contains(&gitlab_tools::GITLAB_ACCEPT_MERGE_REQUEST_TOOL_NAME.to_string())
    );
    developer
        .guard_registered_tool_call(gitlab_tools::GITLAB_CREATE_MERGE_REQUEST_TOOL_NAME)
        .unwrap();
    developer
        .guard_registered_tool_call(gitlab_tools::GITLAB_ACCEPT_MERGE_REQUEST_TOOL_NAME)
        .unwrap();
}

#[test]
fn tool_discovery_lists_jira_default_tools_when_configured() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        ..runtime_config()
    });
    let names = current_tool_names(&server);

    for name in expected_jira_core_default_tools() {
        assert!(names.contains(&name), "{name} should be visible by default");
    }
    assert!(server.get_tool(tools::JIRA_GET_ISSUE_TOOL_NAME).is_some());
}

#[test]
fn tool_discovery_applies_toolsets_and_disabled_tools_to_real_jira_tools() {
    let fields_only = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_fields_read".to_string()]),
        ..runtime_config()
    });
    let transition_disabled = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        disabled_tools: BTreeSet::from([tools::JIRA_TRANSITION_ISSUE_TOOL_NAME.to_string()]),
        ..runtime_config()
    });

    assert_eq!(
        current_tool_names(&fields_only),
        vec![
            tools::JIRA_GET_FIELD_OPTIONS_TOOL_NAME.to_string(),
            tools::JIRA_SEARCH_FIELDS_TOOL_NAME.to_string(),
        ]
    );
    assert!(
        !current_tool_names(&transition_disabled)
            .contains(&tools::JIRA_TRANSITION_ISSUE_TOOL_NAME.to_string())
    );
    assert!(
        transition_disabled
            .guard_registered_tool_call(tools::JIRA_TRANSITION_ISSUE_TOOL_NAME)
            .is_err()
    );
}

#[test]
fn jira_extension_tool_discovery_uses_registered_metadata_at_mcp_boundary() {
    let agile_only = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_agile_boards_read".to_string()]),
        ..runtime_config()
    });
    let default_profile = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        ..runtime_config()
    });

    assert_eq!(
        tool_names(agile_only.filtered_tools_from(jira_extension_candidate_tools())),
        vec![
            tools::JIRA_LIST_AGILE_BOARDS_TOOL_NAME.to_string(),
            tools::JIRA_LIST_BOARD_ISSUES_TOOL_NAME.to_string(),
        ]
    );
    assert!(
        tool_names(default_profile.filtered_tools_from(jira_extension_candidate_tools()))
            .contains(&tools::JIRA_CREATE_ISSUE_TOOL_NAME.to_string())
    );
    assert!(
        !tool_names(default_profile.filtered_tools_from(jira_extension_candidate_tools()))
            .contains(&tools::JIRA_GET_ISSUE_CHANGELOGS_TOOL_NAME.to_string())
    );
}

#[test]
fn jira_product_dependent_tools_have_routes_and_registered_metadata() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_toolsets: tool_registry::all_toolsets(),
        atlassian_oauth_cloud_id: Some("cloud-123".to_string()),
        ..runtime_config()
    });
    let names = current_tool_names(&server);
    let jira_product_tools = jira_product_extension_tool_names();

    assert_eq!(jira_product_tools.len(), 17);
    for name in jira_product_tools {
        assert!(
            tool_registry::metadata_for(name).is_some(),
            "{name} should have registered metadata"
        );
        assert!(
            server.get_tool(name).is_some(),
            "{name} should have a route"
        );
        assert!(
            names.contains(&name.to_string()),
            "{name} should be visible"
        );
    }
}

#[test]
fn jira_product_dependent_toolsets_filter_to_expected_tools() {
    let cases = [
        (
            "jira_agile_boards_read",
            vec![
                tools::JIRA_LIST_AGILE_BOARDS_TOOL_NAME,
                tools::JIRA_LIST_BOARD_ISSUES_TOOL_NAME,
            ],
        ),
        (
            "jira_sprints_read",
            vec![
                tools::JIRA_LIST_BOARD_SPRINTS_TOOL_NAME,
                tools::JIRA_LIST_SPRINT_ISSUES_TOOL_NAME,
            ],
        ),
        (
            "jira_sprints_write",
            vec![
                tools::JIRA_CREATE_SPRINT_TOOL_NAME,
                tools::JIRA_UPDATE_SPRINT_TOOL_NAME,
            ],
        ),
        (
            "jira_sprint_membership_write",
            vec![tools::JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME],
        ),
        (
            "jira_service_desks_read",
            vec![
                tools::JIRA_GET_SERVICE_DESK_FOR_PROJECT_TOOL_NAME,
                tools::JIRA_LIST_SERVICE_DESK_QUEUES_TOOL_NAME,
                tools::JIRA_LIST_SERVICE_DESK_QUEUE_ISSUES_TOOL_NAME,
            ],
        ),
        (
            "jira_issue_forms_read",
            vec![
                tools::JIRA_LIST_ISSUE_FORMS_TOOL_NAME,
                tools::JIRA_GET_ISSUE_FORM_TOOL_NAME,
            ],
        ),
        (
            "jira_issue_forms_write",
            vec![tools::JIRA_UPDATE_ISSUE_FORM_ANSWERS_TOOL_NAME],
        ),
        (
            "jira_issue_metrics_read",
            vec![tools::JIRA_GET_ISSUE_TIMELINE_TOOL_NAME],
        ),
        (
            "jira_issue_sla_read",
            vec![tools::JIRA_GET_ISSUE_SLA_METRICS_TOOL_NAME],
        ),
        (
            "jira_issue_development_read",
            vec![
                tools::JIRA_GET_ISSUE_DEVELOPMENT_TOOL_NAME,
                tools::JIRA_GET_ISSUES_DEVELOPMENT_TOOL_NAME,
            ],
        ),
    ];
    let jira_product_tools = jira_product_extension_tool_names();

    for (toolset, expected) in cases {
        let server = server_with_config(RuntimeConfig {
            jira: Some(jira_config()),
            enabled_toolsets: BTreeSet::from([toolset.to_string()]),
            atlassian_oauth_cloud_id: Some("cloud-123".to_string()),
            ..runtime_config()
        });
        let names = current_tool_names(&server);
        for expected_name in expected {
            assert!(
                names.contains(&expected_name.to_string()),
                "{toolset} should expose {expected_name}"
            );
        }
        for name in jira_product_tools.iter().copied() {
            if tool_registry::metadata_for(name)
                .and_then(|metadata| metadata.toolset)
                .is_some_and(|metadata_toolset| metadata_toolset != toolset)
            {
                assert!(
                    !names.contains(&name.to_string()),
                    "{toolset} should not expose {name}"
                );
            }
        }
    }
}

#[test]
fn all_business_tools_have_metadata_routes_docs_and_control_plane_policy() {
    let jira_names = all_jira_tool_names();
    let confluence_names = all_confluence_tool_names();
    let gitlab_names = all_gitlab_tool_names();
    let mut all_names = jira_names.clone();
    all_names.extend(confluence_names.clone());
    all_names.extend(gitlab_names.clone());
    let unique_names = all_names.iter().collect::<BTreeSet<_>>();
    let support_matrix_rows = support_matrix_tool_rows();
    let support_matrix_names = support_matrix_tool_names();
    let read_write = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        confluence: Some(confluence_config()),
        gitlab: Some(gitlab_config()),
        enabled_toolsets: tool_registry::all_toolsets(),
        atlassian_oauth_cloud_id: Some("cloud-123".to_string()),
        ..runtime_config()
    });
    let read_write_names = current_tool_names(&read_write);

    assert_eq!(jira_names.len(), 49);
    assert_eq!(confluence_names.len(), 24);
    assert_eq!(gitlab_names.len(), 15);
    assert_eq!(all_names.len(), 88);
    assert_eq!(unique_names.len(), all_names.len());
    assert_eq!(support_matrix_rows.len(), 88);
    assert_eq!(support_matrix_names.len(), support_matrix_rows.len());
    assert_eq!(
        support_matrix_names
            .iter()
            .filter(|name| name.starts_with("jira_"))
            .count(),
        49
    );
    assert_eq!(
        support_matrix_names
            .iter()
            .filter(|name| name.starts_with("confluence_"))
            .count(),
        24
    );
    assert_eq!(
        support_matrix_names
            .iter()
            .filter(|name| name.starts_with("gitlab_"))
            .count(),
        15
    );

    for name in all_names {
        let metadata = tool_registry::metadata_for(&name)
            .unwrap_or_else(|| panic!("{name} should have metadata"));
        assert!(metadata.toolset.is_some(), "{name} should have a toolset");
        assert!(
            read_write.get_tool(&name).is_some(),
            "{name} should have a route"
        );
        assert!(
            read_write_names.contains(&name),
            "{name} should be discoverable when service and toolset are enabled"
        );
        assert!(
            support_matrix_names.contains(&name),
            "{name} should be documented in docs/support-matrix.md"
        );
        let support_matrix_row = support_matrix_rows
            .iter()
            .find(|row| row.name == name)
            .unwrap_or_else(|| panic!("{name} should have a support matrix row"));
        assert_eq!(
            support_matrix_row.access,
            match metadata.access {
                ToolAccess::Read => "read",
                ToolAccess::Write => "write",
            },
            "{name} support matrix access should match registry"
        );
        assert_eq!(
            Some(support_matrix_row.toolset.as_str()),
            metadata.toolset,
            "{name} support matrix toolset should match registry"
        );

        read_write
            .guard_registered_tool_call(&name)
            .unwrap_or_else(|_| panic!("{name} should be callable when enabled"));
    }
}

#[test]
fn tool_discovery_uses_registry_metadata_as_single_source() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        confluence: Some(confluence_config()),
        enabled_toolsets: tool_registry::all_toolsets(),
        atlassian_oauth_cloud_id: Some("cloud-123".to_string()),
        ..runtime_config()
    });

    for tool in server.current_tools_result().tools {
        let metadata = tool_registry::metadata_for(tool.name.as_ref())
            .unwrap_or_else(|| panic!("{} should have metadata", tool.name));
        let annotations = tool
            .annotations
            .as_ref()
            .unwrap_or_else(|| panic!("{} should have annotations", tool.name));

        assert_eq!(tool.title.as_deref(), Some(metadata.title));
        assert_eq!(tool.description.as_deref(), Some(metadata.description));
        assert_eq!(
            tool.output_schema.is_some(),
            metadata.output_schema.is_some()
        );
        assert_eq!(annotations.title.as_deref(), Some(metadata.title));
        assert_eq!(
            annotations.read_only_hint,
            Some(metadata.annotations.read_only)
        );
        assert_eq!(
            annotations.destructive_hint,
            Some(metadata.annotations.destructive)
        );
        assert_eq!(
            annotations.idempotent_hint,
            Some(metadata.annotations.idempotent)
        );
        assert_eq!(
            annotations.open_world_hint,
            Some(metadata.annotations.open_world)
        );
    }
}

#[test]
fn confluence_scaffold_routes_are_discoverable_with_registered_metadata() {
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config()
    });
    let names = current_tool_names(&server);

    assert!(names.contains(&confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME.to_string()));
    assert!(names.contains(&confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME.to_string()));
    assert!(
        server
            .get_tool(confluence_tools::CONFLUENCE_GET_SPACE_PAGE_TREE_TOOL_NAME)
            .is_some()
    );
    for name in confluence_tools::CONFLUENCE_TOOL_NAMES {
        assert!(
            tool_registry::metadata_for(name).is_some(),
            "{name} should have registered metadata"
        );
    }
}

#[test]
fn confluence_content_toolsets_are_exact_at_mcp_boundary() {
    let read_write = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_toolsets: BTreeSet::from([
            "confluence_content_read".to_string(),
            "confluence_content_write".to_string(),
            "confluence_content_update".to_string(),
            "confluence_content_delete".to_string(),
            "confluence_page_comments_read".to_string(),
            "confluence_page_comments_write".to_string(),
        ]),
        ..runtime_config()
    });
    let unknown_only = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_toolsets: BTreeSet::from(["confluence_unknown".to_string()]),
        ..runtime_config()
    });
    let read_write_names = current_tool_names(&read_write);

    for expected in [
        confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME,
        confluence_tools::CONFLUENCE_GET_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_LIST_PAGE_CHILDREN_TOOL_NAME,
        confluence_tools::CONFLUENCE_GET_SPACE_PAGE_TREE_TOOL_NAME,
        confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_UPDATE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_DELETE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_MOVE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_LIST_PAGE_COMMENTS_TOOL_NAME,
        confluence_tools::CONFLUENCE_ADD_COMMENT_TOOL_NAME,
        confluence_tools::CONFLUENCE_REPLY_TO_COMMENT_TOOL_NAME,
    ] {
        assert!(
            read_write_names.contains(&expected.to_string()),
            "{expected} should be visible in Confluence content read/write"
        );
    }
    assert!(
        !read_write_names
            .contains(&confluence_tools::CONFLUENCE_LIST_CONTENT_LABELS_TOOL_NAME.to_string())
    );
    assert!(current_tool_names(&unknown_only).is_empty());
    assert!(
        unknown_only
            .guard_registered_tool_call(confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME)
            .is_err()
    );
}

#[test]
fn confluence_attachments_toolsets_are_split_at_mcp_boundary() {
    let read_write = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_toolsets: BTreeSet::from([
            "confluence_attachments_read".to_string(),
            "confluence_attachments_write".to_string(),
            "confluence_attachments_delete".to_string(),
        ]),
        ..runtime_config()
    });
    let read_write_tools = read_write.current_tools_result().tools;
    let read_write_names = tool_names(read_write_tools.clone());

    for expected in [
        confluence_tools::CONFLUENCE_UPLOAD_CONTENT_ATTACHMENT_TOOL_NAME,
        confluence_tools::CONFLUENCE_UPLOAD_CONTENT_ATTACHMENTS_TOOL_NAME,
        confluence_tools::CONFLUENCE_LIST_CONTENT_ATTACHMENTS_TOOL_NAME,
        confluence_tools::CONFLUENCE_DOWNLOAD_ATTACHMENT_TOOL_NAME,
        confluence_tools::CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_TOOL_NAME,
        confluence_tools::CONFLUENCE_DELETE_ATTACHMENT_TOOL_NAME,
        confluence_tools::CONFLUENCE_GET_CONTENT_IMAGE_ATTACHMENTS_TOOL_NAME,
    ] {
        assert!(
            read_write_names.contains(&expected.to_string()),
            "{expected} should be visible for confluence_attachments"
        );
    }
    assert!(!read_write_names.contains(&confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME.to_string()));

    assert_client_compatible_tool_schemas(&read_write_tools);
}

#[test]
fn confluence_enabled_tools_filter_and_direct_call_guard_use_registered_metadata() {
    let unavailable = AtlassianMcpServer::default();
    let search_only = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_tools: Some(BTreeSet::from([
            confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME.to_string(),
        ])),
        enabled_toolsets: BTreeSet::new(),
        ..runtime_config()
    });
    let create_disabled = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_toolsets: tool_registry::all_toolsets(),
        disabled_tools: BTreeSet::from([
            confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME.to_string()
        ]),
        ..runtime_config()
    });

    assert_eq!(
        current_tool_names(&search_only),
        vec![confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME.to_string()]
    );
    assert!(
        unavailable
            .guard_registered_tool_call(confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME)
            .is_err()
    );
    assert!(
        search_only
            .guard_registered_tool_call(confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME)
            .is_ok()
    );
    assert_eq!(
        create_disabled
            .guard_registered_tool_call(confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME)
            .unwrap_err()
            .message,
        "tool not available"
    );
}

#[test]
fn tool_discovery_applies_enabled_tools_filter_to_business_tools() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_tools: Some(BTreeSet::from(
            [tools::JIRA_GET_ISSUE_TOOL_NAME.to_string()],
        )),
        enabled_toolsets: BTreeSet::new(),
        ..runtime_config()
    });

    assert_eq!(
        current_tool_names(&server),
        vec![tools::JIRA_GET_ISSUE_TOOL_NAME.to_string()]
    );
    assert!(
        server
            .guard_registered_tool_call(tools::JIRA_GET_ISSUE_TOOL_NAME)
            .is_ok()
    );
}

#[test]
fn tool_discovery_applies_toolsets_to_business_tools() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::new(),
        ..runtime_config()
    });

    assert!(current_tool_names(&server).is_empty());
}

#[test]
fn tool_discovery_fails_closed_for_unmapped_tools() {
    let server = AtlassianMcpServer::default();
    let tools = server.filtered_tools_from([tool("unmapped_tool")]);
    let names: Vec<_> = tools
        .into_iter()
        .map(|tool| tool.name.to_string())
        .collect();

    assert!(names.is_empty());
}

#[test]
fn tool_discovery_applies_future_service_and_toolset_policy_at_server_boundary() {
    let unavailable = AtlassianMcpServer::default();
    let available = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        confluence: Some(confluence_config()),
        ..runtime_config()
    });
    let jira_fields_only = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_fields_read".to_string()]),
        ..runtime_config()
    });

    assert_eq!(
        tool_names(unavailable.filtered_tools_from_with_metadata(
            [
                tool("synthetic_jira_read"),
                tool("synthetic_confluence_read"),
            ],
            metadata_for_test_tool,
        )),
        Vec::<String>::new()
    );
    assert_eq!(
        tool_names(available.filtered_tools_from_with_metadata(
            [
                tool("synthetic_jira_read"),
                tool("synthetic_confluence_read"),
            ],
            metadata_for_test_tool,
        )),
        vec![
            "synthetic_confluence_read".to_string(),
            "synthetic_jira_read".to_string(),
        ]
    );
    assert!(
        jira_fields_only
            .filtered_tools_from_with_metadata(
                [tool("synthetic_jira_read")],
                metadata_for_test_tool,
            )
            .is_empty()
    );
}

#[test]
fn direct_call_guard_applies_disabled_tools_at_server_boundary() {
    let disabled_server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        disabled_tools: BTreeSet::from(["synthetic_jira_write".to_string()]),
        ..runtime_config()
    });
    let read_write_server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config()
    });

    let error = disabled_server
        .guard_tool_call_with_metadata("synthetic_jira_write", true, metadata_for_test_tool)
        .unwrap_err();

    assert_eq!(error.message, "tool not available");
    assert!(
        read_write_server
            .guard_tool_call_with_metadata("synthetic_jira_write", true, metadata_for_test_tool,)
            .is_ok()
    );
    assert!(
        read_write_server
            .guard_tool_call_with_metadata("synthetic_jira_write", false, metadata_for_test_tool,)
            .is_err()
    );
}

#[test]
fn jira_extension_direct_call_guard_uses_registered_metadata_at_mcp_boundary() {
    let disabled_server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        disabled_tools: BTreeSet::from(
            jira_extension_write_tool_names()
                .into_iter()
                .map(str::to_string)
                .collect::<BTreeSet<_>>(),
        ),
        ..runtime_config()
    });
    let read_write_server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        ..runtime_config()
    });

    for name in jira_extension_write_tool_names() {
        let error = disabled_server
            .guard_tool_call_with_metadata(name, true, tool_registry::metadata_for)
            .unwrap_err();
        assert_eq!(error.message, "tool not available");
    }
    assert!(
        read_write_server
            .guard_tool_call_with_metadata(
                tools::JIRA_CREATE_ISSUE_TOOL_NAME,
                true,
                tool_registry::metadata_for,
            )
            .is_ok()
    );
    assert!(
        read_write_server
            .guard_tool_call_with_metadata(
                tools::JIRA_CREATE_ISSUE_TOOL_NAME,
                false,
                tool_registry::metadata_for,
            )
            .is_err()
    );
}

#[test]
fn jira_general_extension_cross_check_lists_all_names_and_routes() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config()
    });
    let names = jira_general_extension_tool_names();

    assert_eq!(names.len(), 18);
    for name in names {
        let metadata = tool_registry::metadata_for(name)
            .unwrap_or_else(|| panic!("{name} should have metadata"));
        assert_eq!(metadata.service, ToolService::Jira);
        assert!(
            server.get_tool(name).is_some(),
            "{name} should have a route"
        );
    }
}

#[test]
fn jira_general_toolset_and_enabled_tools_filters_are_exact_at_mcp_boundary() {
    let projects_only = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from([
            "jira_projects_read".to_string(),
            "jira_projects_metadata_read".to_string(),
            "jira_project_versions_write".to_string(),
        ]),
        ..runtime_config()
    });
    let worklog_only = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_tools: Some(BTreeSet::from([
            tools::JIRA_LIST_ISSUE_WORKLOGS_TOOL_NAME.to_string()
        ])),
        enabled_toolsets: BTreeSet::new(),
        ..runtime_config()
    });

    assert_eq!(
        current_tool_names(&projects_only),
        vec![
            tools::JIRA_CREATE_PROJECT_VERSION_TOOL_NAME.to_string(),
            tools::JIRA_CREATE_PROJECT_VERSIONS_TOOL_NAME.to_string(),
            tools::JIRA_LIST_PROJECT_COMPONENTS_TOOL_NAME.to_string(),
            tools::JIRA_LIST_PROJECT_VERSIONS_TOOL_NAME.to_string(),
            tools::JIRA_LIST_PROJECTS_TOOL_NAME.to_string(),
        ]
    );
    assert_eq!(
        current_tool_names(&worklog_only),
        vec![tools::JIRA_LIST_ISSUE_WORKLOGS_TOOL_NAME.to_string()]
    );
    assert!(
        worklog_only
            .guard_registered_tool_call(tools::JIRA_LIST_ISSUE_WORKLOGS_TOOL_NAME)
            .is_ok()
    );
    assert!(
        worklog_only
            .guard_registered_tool_call(tools::JIRA_LIST_ISSUE_LINK_TYPES_TOOL_NAME)
            .is_err()
    );
}
