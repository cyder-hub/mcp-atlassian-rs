use super::*;

impl JiraClient {
    pub async fn get_issue_dates(
        &self,
        issue_key: String,
        include_status_changes: bool,
        include_status_summary: bool,
    ) -> Result<Value, UpstreamError> {
        let expand = (include_status_changes || include_status_summary)
            .then(|| vec!["changelog".to_string()]);
        let issue = self
            .get_issue_model(GetIssueRequest {
                issue_key: issue_key.clone(),
                fields: Some(vec![
                    "created".to_string(),
                    "updated".to_string(),
                    "duedate".to_string(),
                    "resolutiondate".to_string(),
                    "status".to_string(),
                ]),
                expand,
                ..Default::default()
            })
            .await?;
        let issue_value = issue.to_simplified_value();
        let status_changes = jira_issue_status_changes(&issue);
        let mut result = json!({
            "issue_key": issue_key,
            "include_status_changes": include_status_changes,
            "include_status_summary": include_status_summary,
            "issue": issue_value,
        });
        if include_status_changes {
            result["status_changes"] = Value::Array(status_changes.clone());
        }
        if include_status_summary {
            result["status_summary"] = jira_issue_status_summary(&issue, &status_changes);
        }

        Ok(result)
    }

    pub async fn get_issue_sla(
        &self,
        issue_key: String,
        metrics: Option<Vec<String>>,
        include_raw_dates: bool,
    ) -> Result<Value, UpstreamError> {
        let requested_fields = metrics
            .clone()
            .filter(|metrics| !metrics.is_empty())
            .unwrap_or_else(|| vec!["*all".to_string()]);
        let issue = self
            .get_issue(GetIssueRequest {
                issue_key: issue_key.clone(),
                fields: Some(requested_fields),
                ..Default::default()
            })
            .await?;
        let metric_values = extract_sla_metric_values(
            issue.get("fields").and_then(Value::as_object),
            metrics.as_deref(),
            include_raw_dates,
        );

        Ok(json!({
            "success": true,
            "issue_key": issue_key,
            "requested_metrics": metrics,
            "include_raw_dates": include_raw_dates,
            "count": metric_values.len(),
            "metrics": metric_values,
            "parsing_limitations": {
                "source": "jira_issue_fields",
                "working_hours_filtering": "not_supported",
                "message": "SLA metrics are parsed from Jira/JSM issue fields; this tool does not apply a local working-hours calendar or recompute SLA timers."
            },
            "product_dependency": {
                "product": "Jira Service Management SLA",
                "available": true,
                "message": "SLA fields were parsed from Jira issue fields; real Jira schema validation is not available in this release."
            },
        }))
    }
}
