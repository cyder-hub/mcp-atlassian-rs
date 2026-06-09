use super::support::*;
use super::*;

#[tokio::test]
async fn issue_dates_status_summary_uses_available_changelog_transitions() {
    let (base_url, requests) = mock_server(json!({
            "id": "10001",
            "key": "ABC-1",
            "fields": {
                "created": "2026-01-01T00:00:00.000+0000",
                "updated": "2026-01-03T00:00:00.000+0000",
                "duedate": "2026-01-10",
                "resolutiondate": "2026-01-04T00:00:00.000+0000",
                "status": {
                    "id": "3",
                    "name": "Done",
                    "statusCategory": {"id": 3, "key": "done", "name": "Done"}
                }
            },
            "changelog": {
                "histories": [
                    {
                        "id": "h1",
                        "created": "2026-01-01T01:00:00.000+0000",
                        "items": [{"field": "status", "from": "1", "fromString": "To Do", "to": "2", "toString": "In Progress"}]
                    },
                    {
                        "id": "h2",
                        "created": "2026-01-02T01:00:00.000+0000",
                        "items": [{"fieldId": "status", "from": "2", "fromString": "In Progress", "to": "3", "toString": "Done"}]
                    }
                ]
            }
        }))
        .await;
    let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

    let value = client
        .get_issue_dates("ABC-1".to_string(), true, true)
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(value["status_changes"].as_array().unwrap().len(), 2);
    assert_eq!(value["status_summary"]["current_status"]["name"], "Done");
    assert_eq!(
        value["status_summary"]["current_status"]["status_category"]["key"],
        "done"
    );
    assert_eq!(value["status_summary"]["transition_count"], 2);
    assert_eq!(
        value["status_summary"]["first_transition"]["to"]["name"],
        "In Progress"
    );
    assert_eq!(
        value["status_summary"]["last_transition"]["to"]["name"],
        "Done"
    );
    assert_eq!(
        requests[0].path,
        "/rest/api/2/issue/ABC-1?fields=created%2Cupdated%2Cduedate%2Cresolutiondate%2Cstatus&expand=changelog"
    );
}

#[tokio::test]
async fn issue_dates_status_summary_handles_missing_changelog() {
    let (base_url, requests) = mock_server(json!({
        "id": "20001",
        "key": "TXT-1",
        "fields": {
            "status": {"name": "Open"}
        }
    }))
    .await;
    let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

    let value = client
        .get_issue_dates("TXT-1".to_string(), false, true)
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert!(value.get("status_changes").is_none());
    assert_eq!(value["status_summary"]["current_status"]["name"], "Open");
    assert_eq!(value["status_summary"]["created"], Value::Null);
    assert_eq!(value["status_summary"]["has_changelog"], false);
    assert_eq!(value["status_summary"]["transition_count"], 0);
    assert_eq!(value["status_summary"]["first_transition"], Value::Null);
    assert_eq!(value["status_summary"]["last_transition"], Value::Null);
    assert_eq!(
        requests[0].path,
        "/rest/api/2/issue/TXT-1?fields=created%2Cupdated%2Cduedate%2Cresolutiondate%2Cstatus&expand=changelog"
    );
}
