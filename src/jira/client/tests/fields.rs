use super::support::*;
use super::*;

#[tokio::test]
async fn search_fields_filters_case_insensitively_and_handles_missing_schema() {
    let (base_url, requests) = mock_server(json!([
        {"id": "summary", "name": "Summary"},
        {"id": "customfield_10001", "name": "Customer Impact", "schema": {"type": "string"}}
    ]))
    .await;
    let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
    let value = client
        .search_fields(Some("CUSTOMER".to_string()), Some(1))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(requests[0].path, "/rest/api/2/field");
    assert_eq!(value.as_array().unwrap().len(), 1);
    assert_eq!(value[0]["id"], "customfield_10001");
}

#[tokio::test]
async fn cloud_search_fields_uses_paginated_v3_endpoint() {
    let (base_url, requests) = mock_server(json!({
        "values": [
            {"id": "project", "key": "project", "name": "Project", "schema": {"type": "project"}},
            {"id": "summary", "name": "Summary"}
        ]
    }))
    .await;
    let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
    let value = client
        .search_fields(Some("project".to_string()), Some(2))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert!(requests[0].path.starts_with("/rest/api/3/field/search?"));
    assert!(requests[0].path.contains("maxResults=2"));
    assert!(requests[0].path.contains("query=project"));
    assert_eq!(value.as_array().unwrap().len(), 1);
    assert_eq!(value[0]["id"], "project");
}

#[tokio::test]
async fn field_options_support_cloud_context_options() {
    let (base_url, requests) = mock_server(json!({"values": [{"id": "1", "value": "High"}]})).await;
    let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
    let value = client
        .get_field_options(FieldOptionsRequest {
            field_id: "customfield_10001".to_string(),
            context_id: Some("20001".to_string()),
            values_only: true,
            ..Default::default()
        })
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(value, json!(["High"]));
    assert!(
        requests[0]
            .path
            .starts_with("/rest/api/3/field/customfield_10001/context/20001/option")
    );
}

#[tokio::test]
async fn field_options_resolves_cloud_context_with_project_and_issue_type() {
    let (base_url, requests) = cloud_field_options_context_mock_server().await;
    let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
    let value = client
        .get_field_options(FieldOptionsRequest {
            field_id: "customfield_10001".to_string(),
            project_key: Some("ABC".to_string()),
            issue_type: Some("Bug".to_string()),
            values_only: true,
            ..Default::default()
        })
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(value, json!(["High"]));
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].method, Method::GET);
    assert!(requests[0].path.starts_with("/rest/api/3/project/ABC"));
    assert_eq!(requests[1].method, Method::POST);
    assert!(
        requests[1]
            .path
            .starts_with("/rest/api/3/field/customfield_10001/context/mapping?")
    );
    assert_eq!(
        requests[1].body["mappings"][0],
        json!({"projectId": "10000", "issueTypeId": "1"})
    );
    assert_eq!(requests[2].method, Method::GET);
    assert!(
        requests[2]
            .path
            .starts_with("/rest/api/3/field/customfield_10001/context/20001/option")
    );
}

#[tokio::test]
async fn field_options_requires_cloud_context_or_project_issue_type() {
    let (base_url, requests) = mock_server(json!({})).await;
    let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
    let error = client
        .get_field_options(FieldOptionsRequest {
            field_id: "customfield_10001".to_string(),
            values_only: true,
            ..Default::default()
        })
        .await
        .unwrap_err();
    let requests = requests.lock().await;

    assert!(requests.is_empty());
    let error = error.to_string();
    assert!(error.contains("context_id is required for Jira Cloud field options"));
}

#[tokio::test]
async fn field_options_reports_cloud_context_mapping_miss() {
    let (base_url, requests) = cloud_field_options_context_mock_server().await;
    let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
    let error = client
        .get_field_options(FieldOptionsRequest {
            field_id: "customfield_10001".to_string(),
            project_key: Some("ABC".to_string()),
            issue_type: Some("Task".to_string()),
            values_only: true,
            ..Default::default()
        })
        .await
        .unwrap_err();
    let requests = requests.lock().await;

    assert_eq!(requests.len(), 2);
    let error = error.to_string();
    assert!(error.contains("No Jira Cloud field context applies"));
}

#[tokio::test]
async fn field_options_support_server_createmeta_options() {
    let (base_url, requests) = mock_server(json!({
        "projects": [{
            "issuetypes": [{
                "fields": {
                    "customfield_10001": {
                        "allowedValues": [{"id": "1", "value": "High"}]
                    }
                }
            }]
        }]
    }))
    .await;
    let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
    let value = client
        .get_field_options(FieldOptionsRequest {
            field_id: "customfield_10001".to_string(),
            project_key: Some("ABC".to_string()),
            issue_type: Some("Bug".to_string()),
            values_only: false,
            ..Default::default()
        })
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert!(requests[0].path.starts_with("/rest/api/2/issue/createmeta"));
    assert_eq!(value["values"][0]["value"], "High");
}
