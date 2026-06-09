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
        read_only: true,
        ..RuntimeConfig::default()
    };
    let server = AtlassianMcpServer::new(Arc::new(AppContext::from_config(&config)));
    let info = server.get_info();
    let instructions = info.instructions.unwrap_or_default();

    assert!(instructions.contains("read-only mode"));
    assert!(instructions.contains("73 Jira and Confluence business tools"));
    assert!(instructions.contains("docs/support-matrix.md"));
}

#[test]
fn tool_discovery_has_no_tools_without_service_config() {
    let server = AtlassianMcpServer::default();

    assert!(current_tool_names(&server).is_empty());
    assert!(server.get_tool(tools::JIRA_GET_ISSUE_TOOL_NAME).is_none());
}

#[test]
fn tool_discovery_lists_jira_default_tools_when_configured() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        ..runtime_config()
    });
    let names = current_tool_names(&server);

    for name in expected_stage_two_default_tools() {
        assert!(names.contains(&name), "{name} should be visible by default");
    }
    assert!(server.get_tool(tools::JIRA_GET_ISSUE_TOOL_NAME).is_some());
}

#[test]
fn tool_discovery_applies_toolsets_and_read_only_to_real_jira_tools() {
    let fields_only = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_fields".to_string()]),
        ..runtime_config()
    });
    let read_only = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
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
        !current_tool_names(&read_only).contains(&tools::JIRA_ADD_COMMENT_TOOL_NAME.to_string())
    );
    assert!(
        read_only
            .guard_registered_tool_call(tools::JIRA_TRANSITION_ISSUE_TOOL_NAME)
            .is_err()
    );
}

#[test]
fn stage_three_candidate_tool_discovery_uses_registered_metadata_at_mcp_boundary() {
    let agile_only = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_agile".to_string()]),
        ..runtime_config()
    });
    let read_only = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        ..runtime_config()
    });

    assert_eq!(
        tool_names(agile_only.filtered_tools_from(stage_three_candidate_tools())),
        vec![
            tools::JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME.to_string(),
            tools::JIRA_CREATE_SPRINT_TOOL_NAME.to_string(),
            tools::JIRA_GET_AGILE_BOARDS_TOOL_NAME.to_string(),
            tools::JIRA_GET_BOARD_ISSUES_TOOL_NAME.to_string(),
            tools::JIRA_GET_SPRINT_ISSUES_TOOL_NAME.to_string(),
            tools::JIRA_GET_SPRINTS_FROM_BOARD_TOOL_NAME.to_string(),
            tools::JIRA_UPDATE_SPRINT_TOOL_NAME.to_string(),
        ]
    );
    assert!(
        !tool_names(read_only.filtered_tools_from(stage_three_candidate_tools()))
            .contains(&tools::JIRA_CREATE_ISSUE_TOOL_NAME.to_string())
    );
    assert!(
        tool_names(read_only.filtered_tools_from(stage_three_candidate_tools()))
            .contains(&tools::JIRA_BATCH_GET_CHANGELOGS_TOOL_NAME.to_string())
    );
}

#[test]
fn c4_product_dependent_tools_have_routes_and_registered_metadata() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        atlassian_oauth_cloud_id: Some("cloud-123".to_string()),
        ..runtime_config()
    });
    let names = current_tool_names(&server);
    let c4_tools = stage_three_c4_tool_names();

    assert_eq!(c4_tools.len(), 17);
    for name in c4_tools {
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
fn c4_product_dependent_toolsets_filter_to_expected_tools() {
    let cases = [
        (
            "jira_agile",
            vec![
                tools::JIRA_GET_AGILE_BOARDS_TOOL_NAME,
                tools::JIRA_GET_BOARD_ISSUES_TOOL_NAME,
                tools::JIRA_GET_SPRINTS_FROM_BOARD_TOOL_NAME,
                tools::JIRA_GET_SPRINT_ISSUES_TOOL_NAME,
                tools::JIRA_CREATE_SPRINT_TOOL_NAME,
                tools::JIRA_UPDATE_SPRINT_TOOL_NAME,
                tools::JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME,
            ],
        ),
        (
            "jira_service_desk",
            vec![
                tools::JIRA_GET_SERVICE_DESK_FOR_PROJECT_TOOL_NAME,
                tools::JIRA_GET_SERVICE_DESK_QUEUES_TOOL_NAME,
                tools::JIRA_GET_QUEUE_ISSUES_TOOL_NAME,
            ],
        ),
        (
            "jira_forms",
            vec![
                tools::JIRA_GET_ISSUE_PROFORMA_FORMS_TOOL_NAME,
                tools::JIRA_GET_PROFORMA_FORM_DETAILS_TOOL_NAME,
                tools::JIRA_UPDATE_PROFORMA_FORM_ANSWERS_TOOL_NAME,
            ],
        ),
        (
            "jira_metrics",
            vec![
                tools::JIRA_GET_ISSUE_DATES_TOOL_NAME,
                tools::JIRA_GET_ISSUE_SLA_TOOL_NAME,
            ],
        ),
        (
            "jira_development",
            vec![
                tools::JIRA_GET_ISSUE_DEVELOPMENT_INFO_TOOL_NAME,
                tools::JIRA_GET_ISSUES_DEVELOPMENT_INFO_TOOL_NAME,
            ],
        ),
    ];
    let c4_tools = stage_three_c4_tool_names();

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
        for name in c4_tools.iter().copied() {
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
fn all_business_tools_have_metadata_routes_docs_and_read_only_policy() {
    let jira_names = all_jira_tool_names();
    let confluence_names = all_confluence_tool_names();
    let mut all_names = jira_names.clone();
    all_names.extend(confluence_names.clone());
    let unique_names = all_names.iter().collect::<BTreeSet<_>>();
    let support_matrix_names = support_matrix_tool_names();
    let read_write = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        confluence: Some(confluence_config()),
        enabled_toolsets: tool_registry::all_toolsets(),
        atlassian_oauth_cloud_id: Some("cloud-123".to_string()),
        ..runtime_config()
    });
    let read_only = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        confluence: Some(confluence_config()),
        enabled_toolsets: tool_registry::all_toolsets(),
        atlassian_oauth_cloud_id: Some("cloud-123".to_string()),
        ..runtime_config()
    });
    let read_write_names = current_tool_names(&read_write);
    let read_only_names = current_tool_names(&read_only);

    assert_eq!(jira_names.len(), 49);
    assert_eq!(confluence_names.len(), 24);
    assert_eq!(all_names.len(), 73);
    assert_eq!(unique_names.len(), all_names.len());
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

        match metadata.access {
            ToolAccess::Read => {
                assert!(
                    read_only_names.contains(&name),
                    "{name} read tool should remain visible in read-only mode"
                );
                read_only
                    .guard_registered_tool_call(&name)
                    .unwrap_or_else(|_| panic!("{name} read tool should be callable"));
            }
            ToolAccess::Write => {
                assert!(
                    !read_only_names.contains(&name),
                    "{name} write tool should be hidden in read-only mode"
                );
                let error = read_only.guard_registered_tool_call(&name).unwrap_err();
                assert!(
                    error.message.contains("disabled in read-only mode"),
                    "{name} should be blocked by read-only guard"
                );
            }
        }
    }
}

#[test]
fn confluence_scaffold_routes_are_discoverable_with_registered_metadata() {
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config()),
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
    for name in confluence_tools::STAGE4_CONFLUENCE_TOOL_NAMES {
        assert!(
            tool_registry::metadata_for(name).is_some(),
            "{name} should have registered metadata"
        );
    }
}

#[test]
fn confluence_c2_toolsets_are_exact_at_mcp_boundary() {
    let read_write = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_toolsets: BTreeSet::from([
            "confluence_pages".to_string(),
            "confluence_comments".to_string(),
        ]),
        ..runtime_config()
    });
    let read_only = server_with_config(RuntimeConfig {
        read_only: true,
        confluence: Some(confluence_config()),
        enabled_toolsets: BTreeSet::from([
            "confluence_pages".to_string(),
            "confluence_comments".to_string(),
        ]),
        ..runtime_config()
    });
    let unknown_only = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_toolsets: BTreeSet::from(["confluence_unknown".to_string()]),
        ..runtime_config()
    });
    let read_write_names = current_tool_names(&read_write);
    let read_only_names = current_tool_names(&read_only);

    for expected in [
        confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME,
        confluence_tools::CONFLUENCE_GET_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_GET_PAGE_CHILDREN_TOOL_NAME,
        confluence_tools::CONFLUENCE_GET_SPACE_PAGE_TREE_TOOL_NAME,
        confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_UPDATE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_DELETE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_MOVE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_GET_COMMENTS_TOOL_NAME,
        confluence_tools::CONFLUENCE_ADD_COMMENT_TOOL_NAME,
        confluence_tools::CONFLUENCE_REPLY_TO_COMMENT_TOOL_NAME,
    ] {
        assert!(
            read_write_names.contains(&expected.to_string()),
            "{expected} should be visible in C2 read/write"
        );
    }
    assert!(
        !read_write_names.contains(&confluence_tools::CONFLUENCE_GET_LABELS_TOOL_NAME.to_string())
    );
    for expected in [
        confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME,
        confluence_tools::CONFLUENCE_GET_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_GET_PAGE_CHILDREN_TOOL_NAME,
        confluence_tools::CONFLUENCE_GET_SPACE_PAGE_TREE_TOOL_NAME,
        confluence_tools::CONFLUENCE_GET_COMMENTS_TOOL_NAME,
    ] {
        assert!(
            read_only_names.contains(&expected.to_string()),
            "{expected} should remain visible in C2 read-only"
        );
    }
    for blocked in [
        confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_UPDATE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_DELETE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_MOVE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_ADD_COMMENT_TOOL_NAME,
        confluence_tools::CONFLUENCE_REPLY_TO_COMMENT_TOOL_NAME,
    ] {
        assert!(
            !read_only_names.contains(&blocked.to_string()),
            "{blocked} should be hidden in C2 read-only"
        );
        assert_eq!(
            read_only
                .guard_registered_tool_call(blocked)
                .unwrap_err()
                .message,
            "tool is disabled in read-only mode"
        );
    }
    assert!(current_tool_names(&unknown_only).is_empty());
    assert!(
        unknown_only
            .guard_registered_tool_call(confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME)
            .is_err()
    );
}

#[test]
fn confluence_attachments_toolset_obeys_read_only_at_mcp_boundary() {
    let read_write = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_toolsets: BTreeSet::from(["confluence_attachments".to_string()]),
        ..runtime_config()
    });
    let read_only = server_with_config(RuntimeConfig {
        read_only: true,
        confluence: Some(confluence_config()),
        enabled_toolsets: BTreeSet::from(["confluence_attachments".to_string()]),
        ..runtime_config()
    });
    let read_write_tools = read_write.current_tools_result().tools;
    let read_write_names = tool_names(read_write_tools.clone());
    let read_only_names = current_tool_names(&read_only);

    for expected in [
        confluence_tools::CONFLUENCE_UPLOAD_ATTACHMENT_TOOL_NAME,
        confluence_tools::CONFLUENCE_UPLOAD_ATTACHMENTS_TOOL_NAME,
        confluence_tools::CONFLUENCE_GET_ATTACHMENTS_TOOL_NAME,
        confluence_tools::CONFLUENCE_DOWNLOAD_ATTACHMENT_TOOL_NAME,
        confluence_tools::CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_TOOL_NAME,
        confluence_tools::CONFLUENCE_DELETE_ATTACHMENT_TOOL_NAME,
        confluence_tools::CONFLUENCE_GET_PAGE_IMAGES_TOOL_NAME,
    ] {
        assert!(
            read_write_names.contains(&expected.to_string()),
            "{expected} should be visible for confluence_attachments"
        );
    }
    assert!(!read_write_names.contains(&confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME.to_string()));

    for expected in [
        confluence_tools::CONFLUENCE_GET_ATTACHMENTS_TOOL_NAME,
        confluence_tools::CONFLUENCE_DOWNLOAD_ATTACHMENT_TOOL_NAME,
        confluence_tools::CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_TOOL_NAME,
        confluence_tools::CONFLUENCE_GET_PAGE_IMAGES_TOOL_NAME,
    ] {
        assert!(
            read_only_names.contains(&expected.to_string()),
            "{expected} should remain visible in read-only"
        );
    }
    for blocked in [
        confluence_tools::CONFLUENCE_UPLOAD_ATTACHMENT_TOOL_NAME,
        confluence_tools::CONFLUENCE_UPLOAD_ATTACHMENTS_TOOL_NAME,
        confluence_tools::CONFLUENCE_DELETE_ATTACHMENT_TOOL_NAME,
    ] {
        assert!(
            !read_only_names.contains(&blocked.to_string()),
            "{blocked} should be hidden in read-only"
        );
        assert_eq!(
            read_only
                .guard_registered_tool_call(blocked)
                .unwrap_err()
                .message,
            "tool is disabled in read-only mode"
        );
    }
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
        ..runtime_config()
    });
    let read_only = server_with_config(RuntimeConfig {
        read_only: true,
        confluence: Some(confluence_config()),
        enabled_toolsets: tool_registry::all_toolsets(),
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
        read_only
            .guard_registered_tool_call(confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME)
            .unwrap_err()
            .message,
        "tool is disabled in read-only mode"
    );
}

#[test]
fn tool_discovery_applies_enabled_tools_filter_to_business_tools() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_tools: Some(BTreeSet::from(
            [tools::JIRA_GET_ISSUE_TOOL_NAME.to_string()],
        )),
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
        enabled_toolsets: BTreeSet::from(["jira_fields".to_string()]),
        ..runtime_config()
    });

    assert_eq!(
        tool_names(unavailable.filtered_tools_from_with_metadata(
            [
                tool("stage1_synthetic_jira_read"),
                tool("stage1_synthetic_confluence_read"),
            ],
            metadata_for_test_tool,
        )),
        Vec::<String>::new()
    );
    assert_eq!(
        tool_names(available.filtered_tools_from_with_metadata(
            [
                tool("stage1_synthetic_jira_read"),
                tool("stage1_synthetic_confluence_read"),
            ],
            metadata_for_test_tool,
        )),
        vec![
            "stage1_synthetic_confluence_read".to_string(),
            "stage1_synthetic_jira_read".to_string(),
        ]
    );
    assert!(
        jira_fields_only
            .filtered_tools_from_with_metadata(
                [tool("stage1_synthetic_jira_read")],
                metadata_for_test_tool,
            )
            .is_empty()
    );
}

#[test]
fn direct_call_guard_applies_future_read_only_policy_at_server_boundary() {
    let read_only_server = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        ..runtime_config()
    });
    let read_write_server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        ..runtime_config()
    });

    let error = read_only_server
        .guard_tool_call_with_metadata("stage1_synthetic_jira_write", true, metadata_for_test_tool)
        .unwrap_err();

    assert_eq!(error.message, "tool is disabled in read-only mode");
    assert!(
        read_write_server
            .guard_tool_call_with_metadata(
                "stage1_synthetic_jira_write",
                true,
                metadata_for_test_tool,
            )
            .is_ok()
    );
    assert!(
        read_write_server
            .guard_tool_call_with_metadata(
                "stage1_synthetic_jira_write",
                false,
                metadata_for_test_tool,
            )
            .is_err()
    );
}

#[test]
fn stage_three_direct_call_guard_uses_registered_metadata_at_mcp_boundary() {
    let read_only_server = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        ..runtime_config()
    });
    let read_write_server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        ..runtime_config()
    });

    for name in stage_three_write_tool_names() {
        let error = read_only_server
            .guard_tool_call_with_metadata(name, true, tool_registry::metadata_for)
            .unwrap_err();
        assert_eq!(error.message, "tool is disabled in read-only mode");
    }
    assert!(
        read_write_server
            .guard_tool_call_with_metadata(
                tools::JIRA_BATCH_GET_CHANGELOGS_TOOL_NAME,
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
fn c3_common_tool_cross_check_lists_all_names_and_routes() {
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        ..runtime_config()
    });
    let names = stage_three_c3_tool_names();

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
fn c3_toolset_and_enabled_tools_filters_are_exact_at_mcp_boundary() {
    let projects_only = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_projects".to_string()]),
        ..runtime_config()
    });
    let worklog_only = server_with_config(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_tools: Some(BTreeSet::from([
            tools::JIRA_GET_WORKLOG_TOOL_NAME.to_string()
        ])),
        ..runtime_config()
    });

    assert_eq!(
        current_tool_names(&projects_only),
        vec![
            tools::JIRA_BATCH_CREATE_VERSIONS_TOOL_NAME.to_string(),
            tools::JIRA_CREATE_VERSION_TOOL_NAME.to_string(),
            tools::JIRA_GET_ALL_PROJECTS_TOOL_NAME.to_string(),
            tools::JIRA_GET_PROJECT_COMPONENTS_TOOL_NAME.to_string(),
            tools::JIRA_GET_PROJECT_VERSIONS_TOOL_NAME.to_string(),
        ]
    );
    assert_eq!(
        current_tool_names(&worklog_only),
        vec![tools::JIRA_GET_WORKLOG_TOOL_NAME.to_string()]
    );
    assert!(
        worklog_only
            .guard_registered_tool_call(tools::JIRA_GET_WORKLOG_TOOL_NAME)
            .is_ok()
    );
    assert!(
        worklog_only
            .guard_registered_tool_call(tools::JIRA_GET_LINK_TYPES_TOOL_NAME)
            .is_err()
    );
}
