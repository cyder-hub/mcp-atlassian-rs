use super::*;

impl JiraClient {
    pub async fn get_issue_development_info(
        &self,
        issue_key: String,
        application_type: Option<String>,
        data_type: Option<String>,
    ) -> Result<Value, AtlassianError> {
        let issue_id = self.resolve_development_issue_id(&issue_key).await?;
        let query = optional_query_params([
            ("issueId", Some(issue_id)),
            ("applicationType", application_type),
            ("dataType", data_type),
        ]);
        self.http
            .send_json(
                self.http
                    .get("/rest/dev-status/1.0/issue/detail")?
                    .query(&query),
            )
            .await
            .or_else(jira_development_unavailable)
    }

    pub async fn get_issues_development_info(
        &self,
        issue_keys: Vec<String>,
        application_type: Option<String>,
        data_type: Option<String>,
    ) -> Result<Value, AtlassianError> {
        let mut results = Vec::new();
        for issue_key in issue_keys {
            match self
                .get_issue_development_info(
                    issue_key.clone(),
                    application_type.clone(),
                    data_type.clone(),
                )
                .await
            {
                Ok(value) => results.push(json!({"issue_key": issue_key, "development": value})),
                Err(error) => {
                    results.push(json!({"issue_key": issue_key, "error": error.to_string()}))
                }
            }
        }
        Ok(json!({ "issues": results }))
    }

    async fn resolve_development_issue_id(
        &self,
        issue_key_or_id: &str,
    ) -> Result<String, AtlassianError> {
        let issue_key_or_id = issue_key_or_id.trim();
        let is_numeric_id = !issue_key_or_id.is_empty()
            && issue_key_or_id.chars().all(|value| value.is_ascii_digit());
        if is_numeric_id && self.config.projects_filter.is_empty() {
            return Ok(issue_key_or_id.to_string());
        }

        if !is_numeric_id {
            ensure_issue_allowed(issue_key_or_id, &self.config)?;
        }
        let issue_key_or_id = safe_path_segment(issue_key_or_id, "issue_key")?;
        let query = [("fields", "id,key")];
        let issue: JiraIssue = self
            .http
            .send_json(
                self.http
                    .get(&format!("/rest/api/2/issue/{issue_key_or_id}"))?
                    .query(&query),
            )
            .await?;
        let issue_key = issue.key.as_deref().ok_or_else(|| {
            AtlassianError::invalid_input(format!(
                "issue `{issue_key_or_id}` response did not include a key"
            ))
        })?;
        ensure_issue_allowed(issue_key, &self.config)?;
        issue.id.ok_or_else(|| {
            AtlassianError::invalid_input(format!(
                "issue `{issue_key_or_id}` response did not include a numeric id"
            ))
        })
    }
}
