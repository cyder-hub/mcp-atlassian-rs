use super::*;

impl JiraClient {
    pub(super) async fn get_issue_model(
        &self,
        request: GetIssueRequest,
    ) -> Result<JiraIssue, AtlassianError> {
        ensure_issue_allowed(&request.issue_key, &self.config)?;
        let issue_key = safe_path_segment(&request.issue_key, "issue_key")?;
        let mut query = optional_query_params([
            ("fields", request.fields.map(|fields| fields.join(","))),
            ("expand", request.expand.map(|expand| expand.join(","))),
            (
                "properties",
                request.properties.map(|properties| properties.join(",")),
            ),
            (
                "updateHistory",
                request.update_history.map(|value| value.to_string()),
            ),
        ]);

        if let Some(comment_limit) = request.comment_limit {
            query.push(("commentLimit".to_string(), comment_limit.to_string()));
        }

        self.http
            .send_json(
                self.http
                    .get(&format!("/rest/api/2/issue/{issue_key}"))?
                    .query(&query),
            )
            .await
    }

    pub async fn get_issue(&self, request: GetIssueRequest) -> Result<Value, AtlassianError> {
        let issue = self.get_issue_model(request).await?;
        Ok(issue.to_simplified_value())
    }

    pub async fn create_issue(&self, fields: Value) -> Result<Value, AtlassianError> {
        let fields = parse_optional_object(Some(fields), "fields")?.unwrap_or_else(|| json!({}));
        let path = match self.config.deployment {
            JiraDeployment::Cloud => "/rest/api/3/issue",
            JiraDeployment::ServerDataCenter => "/rest/api/2/issue",
        };
        let issue: JiraIssue = self
            .http
            .send_json(self.http.post_json(path, &json!({ "fields": fields }))?)
            .await?;
        Ok(
            JiraOperationResult::success("Issue created successfully", issue.to_simplified_value())
                .to_simplified_value(),
        )
    }

    pub async fn batch_create_issues(
        &self,
        issues: Vec<Value>,
        validate_only: bool,
    ) -> Result<Value, AtlassianError> {
        let body = json!({
            "issueUpdates": issues,
            "validateOnly": validate_only,
        });
        let value: Value = self
            .http
            .send_json_value_or_null(self.http.post_json("/rest/api/2/issue/bulk", &body)?)
            .await?;
        Ok(
            JiraOperationResult::success("Issues processed successfully", value)
                .to_simplified_value(),
        )
    }

    pub async fn batch_get_changelogs(
        &self,
        issue_ids_or_keys: Vec<String>,
        fields: Option<Vec<String>>,
        limit: Option<i64>,
    ) -> Result<Value, AtlassianError> {
        if self.config.deployment != JiraDeployment::Cloud {
            return Ok(JiraOperationResult::product_unavailable(
                "Jira Cloud changelog bulk endpoint",
                "Batch get issue changelogs is only available on Jira Cloud.",
            )
            .to_simplified_value());
        }

        let mut body = json!({
            "issueIdsOrKeys": issue_ids_or_keys,
        });
        insert_optional(&mut body, "fieldIds", fields.map(Value::from));
        insert_optional(&mut body, "maxResults", limit.map(Value::from));
        let value: Value = self
            .http
            .send_json(
                self.http
                    .post_json("/rest/api/3/changelog/bulkfetch", &body)?,
            )
            .await?;
        Ok(value)
    }

    pub async fn update_issue(
        &self,
        issue_key: String,
        fields: Value,
        additional_fields: Option<Value>,
        notify_users: Option<bool>,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let fields = merge_optional_objects(fields, additional_fields, "additional_fields")?;
        let mut query = Vec::new();
        if let Some(notify_users) = notify_users {
            query.push(("notifyUsers".to_string(), notify_users.to_string()));
        }
        let path = match self.config.deployment {
            JiraDeployment::Cloud => format!("/rest/api/3/issue/{issue_key}"),
            JiraDeployment::ServerDataCenter => format!("/rest/api/2/issue/{issue_key}"),
        };
        let response = self
            .http
            .send_json_value_or_null(
                self.http
                    .put_json(&path, &json!({ "fields": fields }))?
                    .query(&query),
            )
            .await?;
        Ok(
            JiraOperationResult::success("Issue updated successfully", response)
                .to_simplified_value(),
        )
    }

    pub async fn delete_issue(
        &self,
        issue_key: String,
        delete_subtasks: bool,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let query = vec![("deleteSubtasks".to_string(), delete_subtasks.to_string())];
        let response = self
            .http
            .send_json_value_or_null(
                self.http
                    .delete(&format!("/rest/api/2/issue/{issue_key}"))?
                    .query(&query),
            )
            .await?;
        Ok(json!({
            "success": true,
            "issue_key": issue_key,
            "response": response,
        }))
    }

    pub async fn get_issue_watchers(&self, issue_key: String) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        self.http
            .send_json(
                self.http
                    .get(&format!("/rest/api/2/issue/{issue_key}/watchers"))?,
            )
            .await
    }

    pub async fn add_watcher(
        &self,
        issue_key: String,
        user_identifier: String,
    ) -> Result<Value, AtlassianError> {
        self.watcher_mutation(issue_key, user_identifier, true)
            .await
    }

    pub async fn remove_watcher(
        &self,
        issue_key: String,
        user_identifier: String,
    ) -> Result<Value, AtlassianError> {
        self.watcher_mutation(issue_key, user_identifier, false)
            .await
    }

    pub async fn get_worklog(
        &self,
        issue_key: String,
        start_at: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let query = optional_query_params([
            ("startAt", start_at.map(|value| value.to_string())),
            ("maxResults", limit.map(|value| value.to_string())),
        ]);
        self.http
            .send_json(
                self.http
                    .get(&format!("/rest/api/2/issue/{issue_key}/worklog"))?
                    .query(&query),
            )
            .await
    }

    pub async fn add_worklog(
        &self,
        issue_key: String,
        payload: Value,
        query: Vec<(String, String)>,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        self.http
            .send_json(
                self.http
                    .post_json(&format!("/rest/api/2/issue/{issue_key}/worklog"), &payload)?
                    .query(&query),
            )
            .await
    }

    pub async fn get_link_types(&self) -> Result<Value, AtlassianError> {
        self.http
            .send_json(self.http.get("/rest/api/2/issueLinkType")?)
            .await
    }

    pub async fn link_to_epic(
        &self,
        issue_key: String,
        epic_key: String,
    ) -> Result<Value, AtlassianError> {
        self.update_issue(
            issue_key,
            json!({ "parent": { "key": epic_key } }),
            None,
            None,
        )
        .await
    }

    pub async fn create_issue_link(&self, payload: Value) -> Result<Value, AtlassianError> {
        self.http
            .send_json_value_or_null(self.http.post_json("/rest/api/2/issueLink", &payload)?)
            .await
    }

    pub async fn create_remote_issue_link(
        &self,
        issue_key: String,
        payload: Value,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let path = match self.config.deployment {
            JiraDeployment::Cloud => format!("/rest/api/3/issue/{issue_key}/remotelink"),
            JiraDeployment::ServerDataCenter => {
                format!("/rest/api/2/issue/{issue_key}/remotelink")
            }
        };
        self.http
            .send_json_value_or_null(self.http.post_json(&path, &payload)?)
            .await
    }

    pub async fn remove_issue_link(&self, link_id: String) -> Result<Value, AtlassianError> {
        let link_id = safe_path_segment(&link_id, "link_id")?;
        let response = self
            .http
            .send_json_value_or_null(
                self.http
                    .delete(&format!("/rest/api/2/issueLink/{link_id}"))?,
            )
            .await?;
        Ok(json!({ "success": true, "link_id": link_id, "response": response }))
    }

    async fn watcher_mutation(
        &self,
        issue_key: String,
        user_identifier: String,
        add: bool,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let path = format!("/rest/api/2/issue/{issue_key}/watchers");
        let builder = if add {
            self.http.post_json(&path, &user_identifier)?
        } else {
            let query_key = match self.config.deployment {
                JiraDeployment::Cloud => "accountId",
                JiraDeployment::ServerDataCenter => "username",
            };
            self.http
                .delete(&path)?
                .query(&[(query_key, user_identifier)])
        };
        let response = self.http.send_json_value_or_null(builder).await?;
        Ok(json!({
            "success": true,
            "issue_key": issue_key,
            "response": response,
        }))
    }
}
