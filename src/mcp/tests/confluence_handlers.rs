use super::support::*;
use super::*;

#[tokio::test]
async fn confluence_search_content_handler_returns_structured_content_from_mock_rest() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .search_content(Parameters(confluence_tools::ConfluenceSearchArgs {
            query: "project docs".to_string(),
            limit: Some(10),
            spaces_filter: Some("ENG".to_string()),
        }))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["results"][0]["title"], json!("Roadmap"));
    assert_eq!(structured["results"][0]["space"]["key"], json!("ENG"));
    assert_eq!(structured["start"], json!(0));
    assert_eq!(structured["limit"], json!(10));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, Method::GET);
    assert_eq!(
        query_value(&requests[0].path, "limit").as_deref(),
        Some("10")
    );
    assert_eq!(
        query_value(&requests[0].path, "cql").as_deref(),
        Some(r#"(siteSearch ~ "project docs") AND (space = ENG)"#)
    );
}

#[tokio::test]
async fn confluence_search_content_handler_rejects_invalid_limit_before_http_request() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let error = server
        .search_content(Parameters(confluence_tools::ConfluenceSearchArgs {
            query: "project docs".to_string(),
            limit: Some(51),
            spaces_filter: None,
        }))
        .await
        .unwrap_err();

    assert!(
        error
            .message
            .contains("limit must be less than or equal to 50")
    );
    assert_eq!(requests.lock().await.len(), 0);
}

#[tokio::test]
async fn confluence_get_page_handler_can_lookup_by_title_and_return_raw_content_only() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .get_page(Parameters(confluence_tools::ConfluenceGetPageArgs {
            page_id: None,
            title: Some("Roadmap".to_string()),
            space_key: Some("ENG".to_string()),
            include_metadata: Some(false),
            convert_to_markdown: Some(false),
        }))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["content"]["value"], json!("<p>Raw storage</p>"));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 1);
    assert!(requests[0].path.starts_with("/rest/api/content?"));
    assert_eq!(
        query_value(&requests[0].path, "title").as_deref(),
        Some("Roadmap")
    );
    assert_eq!(
        query_value(&requests[0].path, "spaceKey").as_deref(),
        Some("ENG")
    );
}

#[tokio::test]
async fn confluence_get_page_handler_requires_page_id_or_title_and_space_key() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let error = server
        .get_page(Parameters(confluence_tools::ConfluenceGetPageArgs {
            page_id: None,
            title: Some("Roadmap".to_string()),
            space_key: None,
            include_metadata: None,
            convert_to_markdown: None,
        }))
        .await
        .unwrap_err();

    assert!(
        error
            .message
            .contains("Either page_id OR both title and space_key must be provided")
    );
    assert_eq!(requests.lock().await.len(), 0);
}

#[tokio::test]
async fn confluence_get_page_handler_returns_structured_error_for_missing_page() {
    let (base_url, _requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let by_id = server
        .get_page(Parameters(confluence_tools::ConfluenceGetPageArgs {
            page_id: Some("missing".to_string()),
            title: None,
            space_key: None,
            include_metadata: None,
            convert_to_markdown: None,
        }))
        .await
        .unwrap();
    let by_title = server
        .get_page(Parameters(confluence_tools::ConfluenceGetPageArgs {
            page_id: None,
            title: Some("Missing".to_string()),
            space_key: Some("ENG".to_string()),
            include_metadata: None,
            convert_to_markdown: None,
        }))
        .await
        .unwrap();

    assert!(
        by_id.structured_content.as_ref().unwrap()["error"]
            .as_str()
            .unwrap()
            .contains("page not found")
    );
    assert!(
        by_title.structured_content.as_ref().unwrap()["error"]
            .as_str()
            .unwrap()
            .contains("Page with title 'Missing' not found")
    );
    assert_eq!(by_id.is_error, Some(true));
    assert_eq!(by_title.is_error, Some(true));
}

#[tokio::test]
async fn confluence_list_page_children_handler_returns_pages_and_folders() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .list_page_children(Parameters(
            confluence_tools::ConfluenceListPageChildrenArgs {
                parent_id: "123".to_string(),
                expand: Some("version".to_string()),
                limit: Some(2),
                include_content: Some(true),
                convert_to_markdown: Some(true),
                start: Some(0),
                include_folders: Some(true),
            },
        ))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["parent_id"], json!("123"));
    assert_eq!(structured["count"], json!(2));
    assert_eq!(structured["page_results"], json!(1));
    assert_eq!(structured["folder_results"], json!(1));
    assert_eq!(structured["queries"]["page"]["requested_limit"], json!(2));
    assert_eq!(structured["queries"]["folder"]["requested_limit"], json!(1));
    assert_eq!(structured["results"][0]["title"], json!("Child page"));
    assert_eq!(structured["results"][0]["content"], json!("Child body"));
    assert_eq!(structured["results"][1]["type"], json!("folder"));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert!(
        requests[0]
            .path
            .starts_with("/rest/api/content/123/child/page?")
    );
    assert!(
        requests[1]
            .path
            .starts_with("/rest/api/content/123/child/folder?")
    );
    assert!(
        query_value(&requests[0].path, "expand")
            .unwrap()
            .contains("body.storage")
    );
    assert_eq!(
        query_value(&requests[1].path, "limit").as_deref(),
        Some("1")
    );
}

#[tokio::test]
async fn confluence_list_page_children_handler_rejects_invalid_limit_before_http_request() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let error = server
        .list_page_children(Parameters(
            confluence_tools::ConfluenceListPageChildrenArgs {
                parent_id: "123".to_string(),
                expand: None,
                limit: Some(51),
                include_content: None,
                convert_to_markdown: None,
                start: None,
                include_folders: None,
            },
        ))
        .await
        .unwrap_err();

    assert!(
        error
            .message
            .contains("limit must be less than or equal to 50")
    );
    assert_eq!(requests.lock().await.len(), 0);
}

#[tokio::test]
async fn confluence_get_space_page_tree_handler_returns_sorted_flat_tree() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .get_space_page_tree(Parameters(
            confluence_tools::ConfluenceGetSpacePageTreeArgs {
                space_key: "ENG".to_string(),
                limit: Some(2),
            },
        ))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["space_key"], json!("ENG"));
    assert_eq!(structured["total_pages"], json!(2));
    assert_eq!(structured["has_more"], json!(false));
    assert_eq!(structured["pages"][0]["id"], json!("100"));
    assert_eq!(structured["pages"][0]["parent_id"], Value::Null);
    assert_eq!(structured["pages"][0]["depth"], json!(0));
    assert_eq!(structured["pages"][1]["parent_id"], json!("100"));
    assert_eq!(structured["pages"][1]["depth"], json!(1));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(
        query_value(&requests[0].path, "expand").as_deref(),
        Some("ancestors")
    );
}

#[tokio::test]
async fn confluence_get_space_page_tree_handler_reports_truncation_hint() {
    let (base_url, _requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .get_space_page_tree(Parameters(
            confluence_tools::ConfluenceGetSpacePageTreeArgs {
                space_key: "ENG".to_string(),
                limit: Some(1),
            },
        ))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["total_pages"], json!(1));
    assert_eq!(structured["has_more"], json!(true));
    assert_eq!(structured["next_start"], json!(1));
}

#[tokio::test]
async fn confluence_get_space_page_tree_handler_rejects_invalid_limit_before_http_request() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let error = server
        .get_space_page_tree(Parameters(
            confluence_tools::ConfluenceGetSpacePageTreeArgs {
                space_key: "ENG".to_string(),
                limit: Some(1001),
            },
        ))
        .await
        .unwrap_err();

    assert!(
        error
            .message
            .contains("limit must be less than or equal to 1000")
    );
    assert_eq!(requests.lock().await.len(), 0);
}

#[tokio::test]
async fn confluence_create_page_handler_posts_storage_payload() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .create_page(Parameters(confluence_tools::ConfluenceCreatePageArgs {
            space_key: "ENG".to_string(),
            title: "New page".to_string(),
            content: "# Heading".to_string(),
            parent_id: Some("123".to_string()),
            content_format: Some("markdown".to_string()),
            include_content: Some(false),
            emoji: Some("note".to_string()),
        }))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["message"], json!("Page created successfully"));
    assert_eq!(structured["page"]["id"], json!("900"));
    assert!(structured["page"].get("content").is_none());
    assert_eq!(structured["emoji_status"]["requested"], json!(true));
    assert_eq!(structured["emoji_status"]["applied"], json!(true));
    assert_eq!(structured["emoji_status"]["emoji"], json!("note"));
    assert!(structured["emoji_status"]["error"].is_null());
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].method, Method::POST);
    assert_eq!(requests[0].path, "/rest/api/content");
    assert_eq!(requests[0].body["space"]["key"], json!("ENG"));
    assert_eq!(requests[0].body["ancestors"][0]["id"], json!("123"));
    assert_eq!(
        requests[0].body["body"]["storage"]["value"],
        json!("<h1>Heading</h1>")
    );
    assert_eq!(requests[1].method, Method::PUT);
    assert_eq!(
        requests[1].path,
        "/rest/api/content/900/property/emoji-title-published"
    );
    assert_eq!(requests[1].body["value"], json!("note"));
}

#[tokio::test]
async fn confluence_update_page_handler_increments_version_and_preserves_write_options() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .update_page(Parameters(confluence_tools::ConfluenceUpdatePageArgs {
            page_id: "123".to_string(),
            title: "Updated".to_string(),
            content: "<p>Storage</p>".to_string(),
            is_minor_edit: Some(true),
            version_comment: Some("minor update".to_string()),
            parent_id: Some("100".to_string()),
            content_format: Some("storage".to_string()),
            include_content: Some(true),
            emoji: None,
        }))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["message"], json!("Page updated successfully"));
    assert_eq!(structured["page"]["title"], json!("Updated"));
    assert_eq!(structured["page"]["content"], json!("<p>Storage</p>"));
    assert_eq!(structured["emoji_status"]["requested"], json!(false));
    assert_eq!(structured["emoji_status"]["applied"], json!(false));
    assert!(structured["emoji_status"]["emoji"].is_null());
    assert!(structured["emoji_status"]["error"].is_null());
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].method, Method::GET);
    assert_eq!(requests[1].method, Method::PUT);
    assert_eq!(requests[1].body["version"]["number"], json!(8));
    assert_eq!(requests[1].body["version"]["minorEdit"], json!(true));
    assert_eq!(
        requests[1].body["version"]["message"],
        json!("minor update")
    );
    assert_eq!(requests[1].body["ancestors"][0]["id"], json!("100"));
}

#[tokio::test]
async fn confluence_update_page_handler_reports_emoji_failure_without_failing_page_update() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .update_page(Parameters(confluence_tools::ConfluenceUpdatePageArgs {
            page_id: "123".to_string(),
            title: "Updated".to_string(),
            content: "<p>Storage</p>".to_string(),
            is_minor_edit: Some(false),
            version_comment: None,
            parent_id: None,
            content_format: Some("storage".to_string()),
            include_content: Some(false),
            emoji: Some("fail".to_string()),
        }))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["message"], json!("Page updated successfully"));
    assert_eq!(structured["page"]["id"], json!("123"));
    assert_eq!(structured["emoji_status"]["requested"], json!(true));
    assert_eq!(structured["emoji_status"]["applied"], json!(false));
    assert_eq!(structured["emoji_status"]["emoji"], json!("fail"));
    let error = structured["emoji_status"]["error"].as_str().unwrap();
    assert!(error.contains("emoji failed"));
    assert!(!error.contains("token=secret"));
    assert!(error.contains("token=<redacted>"));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 3);
    assert_eq!(
        requests[2].path,
        "/rest/api/content/123/property/emoji-title-published"
    );
}

#[tokio::test]
async fn confluence_write_handlers_reject_invalid_content_format_before_http_request() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let error = server
        .create_page(Parameters(confluence_tools::ConfluenceCreatePageArgs {
            space_key: "ENG".to_string(),
            title: "New page".to_string(),
            content: "body".to_string(),
            parent_id: None,
            content_format: Some("html".to_string()),
            include_content: None,
            emoji: None,
        }))
        .await
        .unwrap_err();

    assert!(
        error
            .message
            .contains("content_format must be markdown, wiki, or storage")
    );
    assert_eq!(requests.lock().await.len(), 0);
}

#[tokio::test]
async fn confluence_delete_page_handler_returns_success_and_structured_failure() {
    let (base_url, _requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let success = server
        .delete_page(Parameters(confluence_tools::ConfluenceDeletePageArgs {
            page_id: "123".to_string(),
        }))
        .await
        .unwrap();
    let failure = server
        .delete_page(Parameters(confluence_tools::ConfluenceDeletePageArgs {
            page_id: "delete-error".to_string(),
        }))
        .await
        .unwrap();

    assert_eq!(
        success.structured_content.as_ref().unwrap()["success"],
        json!(true)
    );
    assert_eq!(
        failure.structured_content.as_ref().unwrap()["success"],
        json!(false)
    );
    assert_eq!(success.is_error, Some(false));
    assert_eq!(failure.is_error, Some(true));
    assert!(
        failure.structured_content.as_ref().unwrap()["error"]
            .as_str()
            .unwrap()
            .contains("delete failed")
    );
}

#[tokio::test]
async fn confluence_move_page_handler_updates_parent_or_calls_position_endpoint() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let appended = server
        .move_page(Parameters(confluence_tools::ConfluenceMovePageArgs {
            page_id: "123".to_string(),
            target_parent_id: Some("100".to_string()),
            target_space_key: None,
            position: Some("append".to_string()),
        }))
        .await
        .unwrap();
    let positioned = server
        .move_page(Parameters(confluence_tools::ConfluenceMovePageArgs {
            page_id: "123".to_string(),
            target_parent_id: Some("999".to_string()),
            target_space_key: None,
            position: Some("above".to_string()),
        }))
        .await
        .unwrap();

    assert_eq!(
        appended.structured_content.as_ref().unwrap()["message"],
        json!("Page moved successfully")
    );
    assert_eq!(
        positioned.structured_content.as_ref().unwrap()["page"]["id"],
        json!("123")
    );
    let requests = requests.lock().await;
    assert_eq!(requests[1].method, Method::PUT);
    assert_eq!(requests[1].body["ancestors"][0]["id"], json!("100"));
    assert!(
        requests
            .iter()
            .any(|request| request.path == "/rest/api/content/123/move/above/999")
    );
}

#[tokio::test]
async fn confluence_move_page_handler_rejects_invalid_position_before_http_request() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let error = server
        .move_page(Parameters(confluence_tools::ConfluenceMovePageArgs {
            page_id: "123".to_string(),
            target_parent_id: Some("100".to_string()),
            target_space_key: None,
            position: Some("sideways".to_string()),
        }))
        .await
        .unwrap_err();

    assert!(
        error
            .message
            .contains("position must be append, above, or below")
    );
    assert_eq!(requests.lock().await.len(), 0);
}

#[tokio::test]
async fn confluence_list_page_comments_handler_returns_comment_list_and_empty_list() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .list_page_comments(Parameters(
            confluence_tools::ConfluenceListPageCommentsArgs {
                page_id: "123".to_string(),
            },
        ))
        .await
        .unwrap();
    let empty = server
        .list_page_comments(Parameters(
            confluence_tools::ConfluenceListPageCommentsArgs {
                page_id: "empty".to_string(),
            },
        ))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["page_id"], json!("123"));
    assert_eq!(structured["count"], json!(2));
    assert_eq!(structured["comments"][0]["body"], json!("First comment"));
    assert_eq!(
        structured["comments"][0]["author"]["display_name"],
        json!("Ada")
    );
    assert_eq!(structured["comments"][1]["parent_comment_id"], json!("c-1"));
    assert_eq!(
        empty.structured_content.as_ref().unwrap()["count"],
        json!(0)
    );
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert!(
        requests[0]
            .path
            .starts_with("/rest/api/content/123/child/comment?")
    );
    assert!(
        query_value(&requests[0].path, "expand")
            .unwrap()
            .contains("body.storage")
    );
    assert_eq!(
        query_value(&requests[0].path, "depth").as_deref(),
        Some("all")
    );
}

#[tokio::test]
async fn confluence_add_and_reply_comment_handlers_post_storage_payloads() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let added = server
        .add_page_comment(Parameters(confluence_tools::ConfluenceAddCommentArgs {
            page_id: "123".to_string(),
            body: "# Comment".to_string(),
        }))
        .await
        .unwrap();
    let replied = server
        .reply_to_comment(Parameters(confluence_tools::ConfluenceReplyToCommentArgs {
            comment_id: "c-1".to_string(),
            body: "Reply body".to_string(),
        }))
        .await
        .unwrap();

    let added_structured = added.structured_content.as_ref().unwrap();
    let replied_structured = replied.structured_content.as_ref().unwrap();
    assert_eq!(added_structured["success"], json!(true));
    assert_eq!(added_structured["comment"]["id"], json!("c-1"));
    assert_eq!(added_structured["comment"]["body"], json!("Comment"));
    assert_eq!(replied_structured["success"], json!(true));
    assert_eq!(
        replied_structured["comment"]["parent_comment_id"],
        json!("c-1")
    );
    assert_eq!(replied_structured["comment"]["body"], json!("Reply body"));

    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].method, Method::POST);
    assert_eq!(requests[0].path, "/rest/api/content");
    assert_eq!(requests[0].body["type"], json!("comment"));
    assert_eq!(requests[0].body["container"]["id"], json!("123"));
    assert_eq!(requests[0].body["container"]["type"], json!("page"));
    assert_eq!(
        requests[0].body["body"]["storage"]["value"],
        json!("<h1>Comment</h1>")
    );
    assert_eq!(requests[1].body["container"]["id"], json!("c-1"));
    assert_eq!(requests[1].body["container"]["type"], json!("comment"));
    assert_eq!(
        requests[1].body["body"]["storage"]["value"],
        json!("<p>Reply body</p>")
    );
}

#[tokio::test]
async fn confluence_comment_write_handlers_return_structured_failure() {
    let (base_url, _requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let add_failure = server
        .add_page_comment(Parameters(confluence_tools::ConfluenceAddCommentArgs {
            page_id: "comment-error".to_string(),
            body: "Comment".to_string(),
        }))
        .await
        .unwrap();
    let reply_failure = server
        .reply_to_comment(Parameters(confluence_tools::ConfluenceReplyToCommentArgs {
            comment_id: "reply-error".to_string(),
            body: "Reply".to_string(),
        }))
        .await
        .unwrap();

    for result in [add_failure, reply_failure] {
        let structured = result.structured_content.as_ref().unwrap();
        assert_eq!(structured["success"], json!(false));
        assert_eq!(result.is_error, Some(true));
        assert!(
            structured["error"]
                .as_str()
                .unwrap()
                .contains("comment failed")
        );
    }
}

#[tokio::test]
async fn confluence_list_content_labels_handler_returns_label_list_and_empty_list() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .list_content_labels(Parameters(
            confluence_tools::ConfluenceListContentLabelsArgs {
                page_id: "123".to_string(),
            },
        ))
        .await
        .unwrap();
    let empty = server
        .list_content_labels(Parameters(
            confluence_tools::ConfluenceListContentLabelsArgs {
                page_id: "empty-labels".to_string(),
            },
        ))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["content_id"], json!("123"));
    assert_eq!(structured["count"], json!(2));
    assert_eq!(structured["labels"][0]["name"], json!("draft"));
    assert_eq!(structured["labels"][1]["prefix"], json!("my"));
    assert_eq!(
        empty.structured_content.as_ref().unwrap()["count"],
        json!(0)
    );
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].method, Method::GET);
    assert_eq!(requests[0].path, "/rest/api/content/123/label");
    assert_eq!(requests[1].path, "/rest/api/content/empty-labels/label");
}

#[tokio::test]
async fn confluence_add_content_label_handler_posts_label_and_refreshes_list() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .add_content_label(Parameters(confluence_tools::ConfluenceAddLabelArgs {
            page_id: "123".to_string(),
            name: "draft".to_string(),
        }))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["message"], json!("Label added successfully"));
    assert_eq!(structured["content_id"], json!("123"));
    assert_eq!(structured["count"], json!(2));
    assert_eq!(structured["labels"][0]["name"], json!("draft"));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].method, Method::POST);
    assert_eq!(requests[0].path, "/rest/api/content/123/label");
    assert_eq!(requests[0].body[0]["prefix"], json!("global"));
    assert_eq!(requests[0].body[0]["name"], json!("draft"));
    assert_eq!(requests[1].method, Method::GET);
    assert_eq!(requests[1].path, "/rest/api/content/123/label");
}

#[tokio::test]
async fn confluence_add_content_label_handler_returns_error_on_api_failure() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let error = server
        .add_content_label(Parameters(confluence_tools::ConfluenceAddLabelArgs {
            page_id: "label-error".to_string(),
            name: "draft".to_string(),
        }))
        .await
        .unwrap_err();

    assert!(error.message.contains("label failed"));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].path, "/rest/api/content/label-error/label");
}

#[tokio::test]
async fn confluence_search_users_handler_wraps_simple_query_for_cloud() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_cloud_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .search_users(Parameters(confluence_tools::ConfluenceSearchUserArgs {
            query: "Ada".to_string(),
            limit: Some(5),
            group_name: None,
        }))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["cql_query"], json!("user.fullname ~ \"Ada\""));
    assert_eq!(structured["count"], json!(1));
    assert_eq!(structured["results"][0]["title"], json!("Ada Lovelace"));
    assert_eq!(structured["results"][0]["user"]["active"], json!(true));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 1);
    assert!(requests[0].path.starts_with("/rest/api/search/user?"));
    assert_eq!(
        query_value(&requests[0].path, "cql").as_deref(),
        Some("user.fullname ~ \"Ada\"")
    );
    assert_eq!(
        query_value(&requests[0].path, "limit").as_deref(),
        Some("5")
    );
}

#[tokio::test]
async fn confluence_search_users_handler_uses_group_member_fallback_on_server() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .search_users(Parameters(confluence_tools::ConfluenceSearchUserArgs {
            query: "Ada".to_string(),
            limit: Some(10),
            group_name: None,
        }))
        .await
        .unwrap();
    let empty = server
        .search_users(Parameters(confluence_tools::ConfluenceSearchUserArgs {
            query: "Nobody".to_string(),
            limit: Some(10),
            group_name: None,
        }))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["group_name"], json!("confluence-users"));
    assert_eq!(structured["count"], json!(1));
    assert_eq!(structured["results"][0]["title"], json!("Ada Lovelace"));
    assert_eq!(
        empty.structured_content.as_ref().unwrap()["count"],
        json!(0)
    );
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert!(
        requests[0]
            .path
            .starts_with("/rest/api/group/confluence-users/member?")
    );
    assert_eq!(
        query_value(&requests[0].path, "limit").as_deref(),
        Some("200")
    );
}

#[tokio::test]
async fn confluence_search_users_handler_returns_structured_auth_error() {
    let (base_url, _requests) = mock_confluence_server().await;
    let mut config = confluence_config_with_base_url(base_url);
    config.auth = UpstreamAuth::Pat {
        personal_token: "wrong-token".to_string(),
    };
    let server = server_with_config(RuntimeConfig {
        confluence: Some(config),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .search_users(Parameters(confluence_tools::ConfluenceSearchUserArgs {
            query: "Ada".to_string(),
            limit: Some(10),
            group_name: None,
        }))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["success"], json!(false));
    assert_eq!(structured["status"], json!(401));
    assert_eq!(result.is_error, Some(true));
    assert!(
        structured["error"]
            .as_str()
            .unwrap()
            .contains("Authentication failed")
    );
}

#[tokio::test]
async fn confluence_search_users_handler_rejects_invalid_limit_before_http_request() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let error = server
        .search_users(Parameters(confluence_tools::ConfluenceSearchUserArgs {
            query: "Ada".to_string(),
            limit: Some(51),
            group_name: None,
        }))
        .await
        .unwrap_err();

    assert!(
        error
            .message
            .contains("limit must be less than or equal to 50")
    );
    assert_eq!(requests.lock().await.len(), 0);
}

#[tokio::test]
async fn confluence_get_page_version_handler_returns_specific_version() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .get_page_version(Parameters(confluence_tools::ConfluenceGetPageVersionArgs {
            page_id: "123".to_string(),
            version: 1,
            convert_to_markdown: Some(false),
        }))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["id"], json!("123"));
    assert_eq!(structured["status"], json!("historical"));
    assert_eq!(structured["version"]["number"], json!(1));
    assert_eq!(
        structured["content"],
        json!("<h1>Roadmap</h1><p>Hello team</p>")
    );
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 1);
    assert!(requests[0].path.starts_with("/rest/api/content/123?"));
    assert_eq!(
        query_value(&requests[0].path, "status").as_deref(),
        Some("historical")
    );
    assert_eq!(
        query_value(&requests[0].path, "version").as_deref(),
        Some("1")
    );
}

#[tokio::test]
async fn confluence_get_page_version_handler_rejects_zero_version_before_http_request() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let error = server
        .get_page_version(Parameters(confluence_tools::ConfluenceGetPageVersionArgs {
            page_id: "123".to_string(),
            version: 0,
            convert_to_markdown: None,
        }))
        .await
        .unwrap_err();

    assert!(error.message.contains("version must be positive"));
    assert!(requests.lock().await.is_empty());
}

#[tokio::test]
async fn confluence_get_page_version_handler_surfaces_missing_version_error() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let error = server
        .get_page_version(Parameters(confluence_tools::ConfluenceGetPageVersionArgs {
            page_id: "123".to_string(),
            version: 99,
            convert_to_markdown: None,
        }))
        .await
        .unwrap_err();

    assert!(error.message.contains("historical version not found"));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(
        query_value(&requests[0].path, "version").as_deref(),
        Some("99")
    );
}

#[tokio::test]
async fn confluence_get_page_diff_handler_returns_deterministic_diff() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .get_page_diff(Parameters(confluence_tools::ConfluenceGetPageDiffArgs {
            page_id: "123".to_string(),
            from_version: 1,
            to_version: 2,
            context_lines: Some(0),
        }))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["page_id"], json!("123"));
    assert_eq!(structured["title"], json!("Roadmap"));
    assert_eq!(structured["has_changes"], json!(true));
    assert_eq!(structured["context_lines"], json!(0));
    assert_eq!(
        structured["diff"],
        json!("--- v1\n+++ v2\n@@ -1 +1 @@\n-Roadmap Hello team\n+Roadmap Hello team and partners")
    );
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert_eq!(
        query_value(&requests[0].path, "version").as_deref(),
        Some("1")
    );
    assert_eq!(
        query_value(&requests[1].path, "version").as_deref(),
        Some("2")
    );
}

#[tokio::test]
async fn confluence_get_page_diff_handler_returns_empty_diff_for_same_version() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .get_page_diff(Parameters(confluence_tools::ConfluenceGetPageDiffArgs {
            page_id: "123".to_string(),
            from_version: 2,
            to_version: 2,
            context_lines: None,
        }))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["from_version"], json!(2));
    assert_eq!(structured["to_version"], json!(2));
    assert_eq!(structured["diff"], json!(""));
    assert_eq!(structured["has_changes"], json!(false));
    assert_eq!(requests.lock().await.len(), 1);
}

#[tokio::test]
async fn confluence_get_page_diff_handler_rejects_invalid_order_before_http_request() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let error = server
        .get_page_diff(Parameters(confluence_tools::ConfluenceGetPageDiffArgs {
            page_id: "123".to_string(),
            from_version: 3,
            to_version: 2,
            context_lines: None,
        }))
        .await
        .unwrap_err();

    assert!(
        error
            .message
            .contains("from_version must be less than or equal to to_version")
    );
    assert!(requests.lock().await.is_empty());
}

#[tokio::test]
async fn confluence_get_page_view_analytics_handler_returns_cloud_analytics_with_title() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_cloud_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .get_page_view_analytics(Parameters(
            confluence_tools::ConfluenceGetPageViewAnalyticsArgs {
                page_id: "123".to_string(),
                include_title: Some(true),
                from_date: Some("2026-01-01".to_string()),
                to_date: Some("2026-01-31".to_string()),
            },
        ))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["page_id"], json!("123"));
    assert_eq!(structured["page_title"], json!("Roadmap"));
    assert_eq!(structured["total_views"], json!(42));
    assert_eq!(structured["unique_viewers"], json!(7));
    assert_eq!(structured["last_viewed"], json!("2026-06-04T12:00:00Z"));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert!(requests[0].path.starts_with("/rest/api/content/123?"));
    assert!(
        requests[1]
            .path
            .starts_with("/rest/api/analytics/content/123/views?")
    );
    assert_eq!(
        query_value(&requests[1].path, "from").as_deref(),
        Some("2026-01-01")
    );
    assert_eq!(
        query_value(&requests[1].path, "to").as_deref(),
        Some("2026-01-31")
    );
}

#[tokio::test]
async fn confluence_get_page_view_analytics_handler_skips_title_lookup_when_disabled() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_cloud_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .get_page_view_analytics(Parameters(
            confluence_tools::ConfluenceGetPageViewAnalyticsArgs {
                page_id: "123".to_string(),
                include_title: Some(false),
                from_date: None,
                to_date: None,
            },
        ))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert!(structured["page_title"].is_null());
    assert_eq!(structured["total_views"], json!(42));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].path, "/rest/api/analytics/content/123/views");
}

#[tokio::test]
async fn confluence_get_page_view_analytics_handler_returns_unavailable_on_server_without_http() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .get_page_view_analytics(Parameters(
            confluence_tools::ConfluenceGetPageViewAnalyticsArgs {
                page_id: "123".to_string(),
                include_title: Some(true),
                from_date: None,
                to_date: None,
            },
        ))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["success"], json!(false));
    assert_eq!(structured["available"], json!(false));
    assert_eq!(result.is_error, Some(false));
    assert_registered_output_schema_declares_properties(
        confluence_tools::CONFLUENCE_GET_PAGE_VIEW_ANALYTICS_TOOL_NAME,
        &["success", "available", "page_id", "total_views", "error"],
    );
    assert!(
        structured["error"]
            .as_str()
            .unwrap()
            .contains("only available for Confluence Cloud")
    );
    assert!(requests.lock().await.is_empty());
}

#[tokio::test]
async fn confluence_get_page_view_analytics_handler_returns_structured_auth_error() {
    let (base_url, _requests) = mock_confluence_server().await;
    let mut config = confluence_cloud_config_with_base_url(base_url);
    config.auth = UpstreamAuth::Pat {
        personal_token: "wrong-token".to_string(),
    };
    let server = server_with_config(RuntimeConfig {
        confluence: Some(config),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .get_page_view_analytics(Parameters(
            confluence_tools::ConfluenceGetPageViewAnalyticsArgs {
                page_id: "123".to_string(),
                include_title: Some(false),
                from_date: None,
                to_date: None,
            },
        ))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["success"], json!(false));
    assert_eq!(structured["status"], json!(401));
    assert_eq!(result.is_error, Some(true));
    assert!(
        structured["error"]
            .as_str()
            .unwrap()
            .contains("Authentication failed")
    );
}

#[tokio::test]
async fn confluence_get_page_view_analytics_handler_rejects_empty_page_id_before_http() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_cloud_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let error = server
        .get_page_view_analytics(Parameters(
            confluence_tools::ConfluenceGetPageViewAnalyticsArgs {
                page_id: " ".to_string(),
                include_title: None,
                from_date: None,
                to_date: None,
            },
        ))
        .await
        .unwrap_err();

    assert!(error.message.contains("page_id must not be empty"));
    assert!(requests.lock().await.is_empty());
}

#[tokio::test]
async fn confluence_list_content_attachments_handler_handles_empty_and_missing_fields() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let empty = server
        .list_content_attachments(Parameters(
            confluence_tools::ConfluenceListContentAttachmentsArgs {
                content_id: "empty-attachments".to_string(),
                start: None,
                limit: None,
                filename: None,
                media_type: None,
            },
        ))
        .await
        .unwrap();
    let missing_fields = server
        .list_content_attachments(Parameters(
            confluence_tools::ConfluenceListContentAttachmentsArgs {
                content_id: "missing-attachment-fields".to_string(),
                start: None,
                limit: None,
                filename: None,
                media_type: None,
            },
        ))
        .await
        .unwrap();

    assert_eq!(
        empty.structured_content.as_ref().unwrap()["count"],
        json!(0)
    );
    let missing = missing_fields.structured_content.as_ref().unwrap();
    assert_eq!(missing["count"], json!(1));
    assert_eq!(missing["attachments"][0]["id"], json!("att-min"));
    assert!(missing["attachments"][0]["title"].is_null());
    assert!(missing["attachments"][0]["media_type"].is_null());
    assert_eq!(requests.lock().await.len(), 2);
}

#[tokio::test]
async fn confluence_list_content_attachments_handler_filters_filename_and_media_type() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let by_filename = server
        .list_content_attachments(Parameters(
            confluence_tools::ConfluenceListContentAttachmentsArgs {
                content_id: "123".to_string(),
                start: None,
                limit: None,
                filename: Some("file.png".to_string()),
                media_type: None,
            },
        ))
        .await
        .unwrap();
    let by_media_type = server
        .list_content_attachments(Parameters(
            confluence_tools::ConfluenceListContentAttachmentsArgs {
                content_id: "123".to_string(),
                start: None,
                limit: None,
                filename: None,
                media_type: Some("text/plain".to_string()),
            },
        ))
        .await
        .unwrap();

    let filename = by_filename.structured_content.as_ref().unwrap();
    assert_eq!(filename["count"], json!(1));
    assert_eq!(filename["attachments"][0]["title"], json!("file.png"));
    let media_type = by_media_type.structured_content.as_ref().unwrap();
    assert_eq!(media_type["count"], json!(1));
    assert_eq!(media_type["attachments"][0]["title"], json!("notes.txt"));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert!(query_value(&requests[0].path, "filename").is_none());
    assert!(query_value(&requests[1].path, "media-type").is_none());
}

#[tokio::test]
async fn confluence_list_content_attachments_handler_rejects_invalid_limit_before_http() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let error = server
        .list_content_attachments(Parameters(
            confluence_tools::ConfluenceListContentAttachmentsArgs {
                content_id: "123".to_string(),
                start: None,
                limit: Some(101),
                filename: None,
                media_type: None,
            },
        ))
        .await
        .unwrap_err();

    assert!(
        error
            .message
            .contains("limit must be less than or equal to 100")
    );
    assert!(requests.lock().await.is_empty());
}

#[tokio::test]
async fn confluence_download_attachment_handler_returns_bounded_base64_content() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .download_attachment(Parameters(
            confluence_tools::ConfluenceDownloadAttachmentArgs {
                attachment_id: "att-1".to_string(),
                max_bytes: None,
            },
        ))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["success"], json!(true));
    assert_eq!(structured["attachment"]["id"], json!("att-1"));
    assert_registered_output_schema_declares_properties(
        confluence_tools::CONFLUENCE_DOWNLOAD_ATTACHMENT_TOOL_NAME,
        &["success", "attachment", "error"],
    );
    assert_eq!(
        structured["attachment"]["content"],
        json!({
            "encoding": "base64",
            "content_type": "image/png",
            "size": 11,
            "data": "aW1hZ2UtYnl0ZXM="
        })
    );
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert!(requests[0].path.starts_with("/rest/api/content/att-1?"));
    assert_eq!(
        requests[1].path,
        "/download/attachments/att-1/file.png?token=secret"
    );
}

#[tokio::test]
async fn confluence_download_attachment_handler_rejects_stream_limit_and_cross_origin_url() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let stream_too_large = server
        .download_attachment(Parameters(
            confluence_tools::ConfluenceDownloadAttachmentArgs {
                attachment_id: "att-stream-large".to_string(),
                max_bytes: None,
            },
        ))
        .await
        .unwrap();
    let cross_origin = server
        .download_attachment(Parameters(
            confluence_tools::ConfluenceDownloadAttachmentArgs {
                attachment_id: "att-cross".to_string(),
                max_bytes: None,
            },
        ))
        .await
        .unwrap();

    assert_eq!(stream_too_large.is_error, Some(true));
    assert_eq!(cross_origin.is_error, Some(true));
    let stream_too_large = stream_too_large.structured_content.as_ref().unwrap();
    assert_eq!(stream_too_large["success"], json!(false));
    assert!(
        stream_too_large["error"]
            .as_str()
            .unwrap()
            .contains("exceeds configured limit")
    );
    let cross_origin = cross_origin.structured_content.as_ref().unwrap();
    assert_eq!(cross_origin["success"], json!(false));
    assert!(
        cross_origin["error"]
            .as_str()
            .unwrap()
            .contains("configured upstream base origin")
    );
    assert!(
        !cross_origin["error"]
            .as_str()
            .unwrap()
            .contains("token=secret")
    );
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 3);
    assert!(
        requests
            .iter()
            .any(|request| request.path == "/download/attachments/att-stream-large/large.bin")
    );
    assert!(
        requests
            .iter()
            .all(|request| !request.path.contains("other.example"))
    );
}

#[tokio::test]
async fn confluence_download_attachment_handler_rejects_invalid_max_bytes_before_http() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let error = server
        .download_attachment(Parameters(
            confluence_tools::ConfluenceDownloadAttachmentArgs {
                attachment_id: "att-1".to_string(),
                max_bytes: Some(0),
            },
        ))
        .await
        .unwrap_err();

    assert!(error.message.contains("max_bytes must be positive"));
    assert!(requests.lock().await.is_empty());
}

#[tokio::test]
async fn confluence_download_content_attachments_handler_returns_partial_failure_summary() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .download_content_attachments(Parameters(
            confluence_tools::ConfluenceDownloadContentAttachmentsArgs {
                content_id: "download-batch".to_string(),
                filename: None,
                media_type: None,
                max_bytes: None,
                limit: None,
            },
        ))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["success"], json!(true));
    assert_eq!(structured["summary"]["total"], json!(3));
    assert_eq!(structured["summary"]["downloaded"], json!(1));
    assert_eq!(structured["summary"]["failed"], json!(2));
    assert_eq!(structured["summary"]["pages_fetched"], json!(1));
    assert_eq!(structured["summary"]["has_more"], json!(false));
    assert_eq!(structured["summary"]["limit_applied"], json!(false));
    assert_eq!(structured["attachments"][0]["id"], json!("att-1"));
    assert_registered_output_schema_declares_properties(
        confluence_tools::CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_TOOL_NAME,
        &["success", "summary", "attachments", "failed"],
    );
    assert_eq!(structured["failed"].as_array().unwrap().len(), 2);
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert!(
        requests[0]
            .path
            .starts_with("/rest/api/content/download-batch/child/attachment?")
    );
    assert_eq!(
        requests[1].path,
        "/download/attachments/att-1/file.png?token=secret"
    );
}

#[tokio::test]
async fn confluence_download_content_attachments_handler_applies_filters_limit_and_max_bytes() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .download_content_attachments(Parameters(
            confluence_tools::ConfluenceDownloadContentAttachmentsArgs {
                content_id: "download-batch".to_string(),
                filename: Some("file.png".to_string()),
                media_type: Some("image/png".to_string()),
                max_bytes: Some(20),
                limit: Some(1),
            },
        ))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["success"], json!(true));
    assert_eq!(structured["summary"]["downloaded"], json!(1));
    assert_eq!(structured["summary"]["failed"], json!(0));
    assert_eq!(
        structured["summary"]["filters"]["filename"],
        json!("file.png")
    );
    assert_eq!(
        structured["summary"]["filters"]["media_type"],
        json!("image/png")
    );
    assert_eq!(structured["summary"]["max_bytes"], json!(20));
    assert_eq!(structured["attachments"][0]["id"], json!("att-1"));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert_eq!(
        query_value(&requests[0].path, "limit").as_deref(),
        Some("1")
    );
    assert_eq!(
        requests[1].path,
        "/download/attachments/att-1/file.png?token=secret"
    );
}

#[tokio::test]
async fn confluence_download_content_attachments_handler_paginates_until_no_next_link() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .download_content_attachments(Parameters(
            confluence_tools::ConfluenceDownloadContentAttachmentsArgs {
                content_id: "download-paged".to_string(),
                filename: None,
                media_type: None,
                max_bytes: None,
                limit: None,
            },
        ))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["success"], json!(true));
    assert_eq!(structured["summary"]["total"], json!(2));
    assert_eq!(structured["summary"]["downloaded"], json!(2));
    assert_eq!(structured["summary"]["failed"], json!(0));
    assert_eq!(structured["summary"]["pages_fetched"], json!(2));
    assert_eq!(structured["summary"]["has_more"], json!(false));
    assert_eq!(structured["summary"]["limit_applied"], json!(false));
    assert_eq!(structured["attachments"][0]["id"], json!("att-page-1"));
    assert_eq!(structured["attachments"][1]["id"], json!("att-page-2"));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 4);
    assert!(
        requests[0]
            .path
            .starts_with("/rest/api/content/download-paged/child/attachment?")
    );
    assert!(
        requests[2]
            .path
            .starts_with("/rest/api/content/download-paged/child/attachment?")
    );
    assert_eq!(
        query_value(&requests[2].path, "start").as_deref(),
        Some("1")
    );
}

#[tokio::test]
async fn confluence_download_content_attachments_handler_reports_page_protection_limit() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .download_content_attachments(Parameters(
            confluence_tools::ConfluenceDownloadContentAttachmentsArgs {
                content_id: "download-capped".to_string(),
                filename: None,
                media_type: None,
                max_bytes: None,
                limit: None,
            },
        ))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["success"], json!(true));
    assert_eq!(
        structured["summary"]["pages_fetched"],
        json!(CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_MAX_PAGES)
    );
    assert_eq!(structured["summary"]["downloaded"], json!(0));
    assert_eq!(
        structured["summary"]["failed"],
        json!(CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_MAX_PAGES)
    );
    assert_eq!(structured["summary"]["has_more"], json!(true));
    assert_eq!(structured["summary"]["next_start"], json!(10));
    assert_eq!(structured["summary"]["limit_applied"], json!(true));
    let requests = requests.lock().await;
    assert_eq!(
        requests.len() as u64,
        CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_MAX_PAGES
    );
}

#[tokio::test]
async fn confluence_get_content_image_attachments_handler_filters_non_images_and_uses_extension_fallback()
 {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .get_content_image_attachments(Parameters(
            confluence_tools::ConfluenceGetContentImageAttachmentsArgs {
                content_id: "images".to_string(),
                max_bytes: None,
            },
        ))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["success"], json!(true));
    assert_eq!(structured["images_only"], json!(true));
    assert_eq!(structured["count"], json!(2));
    assert_eq!(structured["skipped_non_images"], json!(1));
    assert_eq!(structured["images"][0]["id"], json!("att-1"));
    assert_eq!(
        structured["images"][0]["resolved_mime_type"],
        json!("image/png")
    );
    assert_eq!(structured["images"][1]["id"], json!("att-octet-image"));
    assert_eq!(
        structured["images"][1]["resolved_mime_type"],
        json!("image/jpeg")
    );
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 3);
    assert!(
        requests[0]
            .path
            .starts_with("/rest/api/content/images/child/attachment?")
    );
    assert!(
        requests
            .iter()
            .any(|request| request.path == "/download/attachments/att-1/file.png?token=secret")
    );
    assert!(
        requests
            .iter()
            .any(|request| request.path == "/download/attachments/att-octet-image/photo.jpg")
    );
    assert!(
        requests
            .iter()
            .all(|request| !request.path.contains("notes.txt"))
    );
}

#[tokio::test]
async fn confluence_upload_content_attachment_handler_sends_local_file_as_multipart() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let file_path = temp_confluence_upload_file("upload.txt", b"hello");
    let result = server
        .upload_content_attachment(Parameters(
            confluence_tools::ConfluenceUploadContentAttachmentArgs {
                content_id: "123".to_string(),
                file_path: file_path.clone(),
                comment: Some("Initial upload".to_string()),
                minor_edit: Some(true),
            },
        ))
        .await
        .unwrap();
    remove_temp_confluence_upload_file(&file_path);

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["success"], json!(true));
    assert_eq!(structured["filename"], json!("upload.txt"));
    assert_eq!(structured["minor_edit"], json!(true));
    assert_eq!(structured["attachment"]["title"], json!("upload.txt"));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, Method::PUT);
    assert_eq!(requests[0].path, "/rest/api/content/123/child/attachment");
    let body = requests[0].body.as_str().unwrap();
    assert!(body.contains("name=\"file\"; filename=\"upload.txt\""));
    assert!(body.contains("hello"));
    assert!(body.contains("name=\"comment\""));
    assert!(body.contains("Initial upload"));
    assert!(body.contains("name=\"minorEdit\""));
    assert!(body.contains("true"));
    assert!(!body.contains("workhub-rs-boundary"));
    assert!(!body.contains(&file_path));
}

#[tokio::test]
async fn confluence_upload_content_attachment_handler_rejects_oversized_file_before_http_request() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let file_path = oversized_temp_confluence_upload_file("too-large.bin");
    let result = server
        .upload_content_attachment(Parameters(
            confluence_tools::ConfluenceUploadContentAttachmentArgs {
                content_id: "123".to_string(),
                file_path: file_path.clone(),
                comment: None,
                minor_edit: None,
            },
        ))
        .await
        .unwrap();
    remove_temp_confluence_upload_file(&file_path);

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["success"], json!(false));
    assert_eq!(result.is_error, Some(true));
    let error = structured["error"].as_str().unwrap();
    assert!(error.contains("exceeds configured upload limit"));
    assert!(error.contains("too-large.bin"));
    assert!(!error.contains(&file_path));
    assert!(requests.lock().await.is_empty());
}

#[tokio::test]
async fn confluence_upload_content_attachments_handler_returns_partial_success_summary() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let ok_path = temp_confluence_upload_file("batch-1.txt", b"batch");
    let oversized_path = oversized_temp_confluence_upload_file("batch-too-large.bin");
    let missing_path = std::env::temp_dir()
        .join("workhub-rs-missing-upload.txt")
        .to_string_lossy()
        .into_owned();
    let result = server
        .upload_content_attachments(Parameters(
            confluence_tools::ConfluenceUploadContentAttachmentsArgs {
                content_id: "123".to_string(),
                file_paths: format!("{ok_path}, {oversized_path}, {missing_path}"),
                comment: Some("Batch upload".to_string()),
                minor_edit: Some(false),
            },
        ))
        .await
        .unwrap();
    remove_temp_confluence_upload_file(&ok_path);
    remove_temp_confluence_upload_file(&oversized_path);

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["success"], json!(false));
    assert_eq!(structured["partial_success"], json!(true));
    assert_eq!(result.is_error, Some(false));
    assert_eq!(structured["summary"]["total"], json!(3));
    assert_eq!(structured["summary"]["uploaded"], json!(1));
    assert_eq!(structured["summary"]["failed"], json!(2));
    assert_registered_output_schema_declares_properties(
        confluence_tools::CONFLUENCE_UPLOAD_CONTENT_ATTACHMENTS_TOOL_NAME,
        &[
            "success",
            "partial_success",
            "summary",
            "attachments",
            "failed",
        ],
    );
    assert_eq!(
        structured["attachments"][0]["filename"],
        json!("batch-1.txt")
    );
    assert_eq!(
        structured["failed"][0]["filename"],
        json!("batch-too-large.bin")
    );
    assert!(
        structured["failed"][0]["error"]
            .as_str()
            .unwrap()
            .contains("exceeds configured upload limit")
    );
    assert_eq!(
        structured["failed"][1]["filename"],
        json!("workhub-rs-missing-upload.txt")
    );
    assert!(
        structured["failed"][1]["error"]
            .as_str()
            .unwrap()
            .contains("failed to inspect local file")
    );
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].path, "/rest/api/content/123/child/attachment");
}

#[tokio::test]
async fn confluence_delete_attachment_handler_returns_structured_success_and_failure() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let success = server
        .delete_attachment(Parameters(
            confluence_tools::ConfluenceDeleteAttachmentArgs {
                attachment_id: "att-1".to_string(),
            },
        ))
        .await
        .unwrap();
    let failure = server
        .delete_attachment(Parameters(
            confluence_tools::ConfluenceDeleteAttachmentArgs {
                attachment_id: "att-delete-error".to_string(),
            },
        ))
        .await
        .unwrap();

    let success = success.structured_content.as_ref().unwrap();
    assert_eq!(success["success"], json!(true));
    assert_eq!(success["attachment_id"], json!("att-1"));
    assert_eq!(failure.is_error, Some(true));
    let failure = failure.structured_content.as_ref().unwrap();
    assert_eq!(failure["success"], json!(false));
    assert_eq!(failure["attachment_id"], json!("att-delete-error"));
    assert!(
        failure["error"]
            .as_str()
            .unwrap()
            .contains("delete attachment failed")
    );
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].method, Method::DELETE);
    assert_eq!(requests[0].path, "/rest/api/content/att-1");
    assert_eq!(requests[1].method, Method::DELETE);
    assert_eq!(requests[1].path, "/rest/api/content/att-delete-error");
}

#[tokio::test]
async fn confluence_get_page_handler_returns_metadata_by_page_id() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .get_page(Parameters(confluence_tools::ConfluenceGetPageArgs {
            page_id: Some("123".to_string()),
            title: Some("Ignored".to_string()),
            space_key: Some("IGN".to_string()),
            include_metadata: Some(true),
            convert_to_markdown: Some(true),
        }))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["metadata"]["id"], json!("123"));
    assert_eq!(structured["metadata"]["title"], json!("Roadmap"));
    assert_eq!(
        structured["metadata"]["content"],
        json!("Roadmap Hello & welcome")
    );
    assert_eq!(structured["metadata"]["version"]["number"], json!(7));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 1);
    assert!(requests[0].path.starts_with("/rest/api/content/123?"));
    assert!(
        query_value(&requests[0].path, "expand")
            .unwrap()
            .contains("body.storage")
    );
}

#[tokio::test]
async fn confluence_list_content_attachments_handler_returns_metadata_page() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let result = server
        .list_content_attachments(Parameters(
            confluence_tools::ConfluenceListContentAttachmentsArgs {
                content_id: "123".to_string(),
                start: Some(0),
                limit: Some(2),
                filename: None,
                media_type: None,
            },
        ))
        .await
        .unwrap();

    let structured = result.structured_content.as_ref().unwrap();
    assert_eq!(structured["success"], json!(true));
    assert_eq!(structured["content_id"], json!("123"));
    assert_eq!(structured["count"], json!(2));
    assert_eq!(structured["attachments"][0]["id"], json!("att-1"));
    assert_eq!(
        structured["attachments"][0]["media_type"],
        json!("image/png")
    );
    assert_eq!(structured["attachments"][0]["file_size"], json!(42));
    assert_eq!(
        structured["attachments"][1]["media_type"],
        json!("text/plain")
    );
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 1);
    assert!(
        requests[0]
            .path
            .starts_with("/rest/api/content/123/child/attachment?")
    );
    assert_eq!(
        query_value(&requests[0].path, "start").as_deref(),
        Some("0")
    );
    assert_eq!(
        query_value(&requests[0].path, "limit").as_deref(),
        Some("2")
    );
    assert_eq!(
        query_value(&requests[0].path, "expand").as_deref(),
        Some("metadata,extensions,version")
    );
}

#[tokio::test]
async fn confluence_download_attachment_handler_reports_metadata_errors_without_fetching() {
    let (base_url, requests) = mock_confluence_server().await;
    let server = server_with_config(RuntimeConfig {
        confluence: Some(confluence_config_with_base_url(base_url)),
        mcp_enabled_toolsets: tool_registry::all_toolsets(),
        ..runtime_config_all_toolsets()
    });
    let no_url = server
        .download_attachment(Parameters(
            confluence_tools::ConfluenceDownloadAttachmentArgs {
                attachment_id: "att-no-url".to_string(),
                max_bytes: None,
            },
        ))
        .await
        .unwrap();
    let too_large = server
        .download_attachment(Parameters(
            confluence_tools::ConfluenceDownloadAttachmentArgs {
                attachment_id: "att-large".to_string(),
                max_bytes: None,
            },
        ))
        .await
        .unwrap();

    let no_url = no_url.structured_content.as_ref().unwrap();
    assert_eq!(no_url["success"], json!(false));
    assert!(no_url["error"].as_str().unwrap().contains("download URL"));
    let too_large = too_large.structured_content.as_ref().unwrap();
    assert_eq!(too_large["success"], json!(false));
    assert!(
        too_large["error"]
            .as_str()
            .unwrap()
            .contains("exceeds the inline limit")
    );
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert!(
        requests
            .iter()
            .all(|request| !request.path.contains("/download/"))
    );
}
