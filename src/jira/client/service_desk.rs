use super::*;

impl JiraClient {
    pub async fn get_service_desk_for_project(
        &self,
        project_key: String,
    ) -> Result<Value, UpstreamError> {
        let project_key = safe_path_segment(&project_key, "project_key")?;
        let desks: Value = match self
            .http
            .send_json(self.http.get("/rest/servicedeskapi/servicedesk")?)
            .await
        {
            Ok(desks) => desks,
            Err(error) => return jira_service_management_unavailable(error),
        };
        let desk = desks
            .get("values")
            .and_then(Value::as_array)
            .and_then(|values| {
                values.iter().find(|desk| {
                    desk.get("projectKey")
                        .and_then(Value::as_str)
                        .is_some_and(|key| key == project_key)
                })
            })
            .cloned()
            .unwrap_or(Value::Null);
        Ok(json!({ "project_key": project_key, "service_desk": desk }))
    }

    pub async fn get_service_desk_queues(
        &self,
        service_desk_id: String,
        start_at: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Value, UpstreamError> {
        let service_desk_id = safe_path_segment(&service_desk_id, "service_desk_id")?;
        let query = optional_query_params([
            ("start", start_at.map(|value| value.to_string())),
            ("limit", limit.map(|value| value.to_string())),
        ]);
        self.http
            .send_json(
                self.http
                    .get(&format!(
                        "/rest/servicedeskapi/servicedesk/{service_desk_id}/queue"
                    ))?
                    .query(&query),
            )
            .await
            .or_else(jira_service_management_unavailable)
    }

    pub async fn get_queue_issues(
        &self,
        service_desk_id: String,
        queue_id: String,
        start_at: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Value, UpstreamError> {
        let service_desk_id = safe_path_segment(&service_desk_id, "service_desk_id")?;
        let queue_id = safe_path_segment(&queue_id, "queue_id")?;
        let query = optional_query_params([
            ("start", start_at.map(|value| value.to_string())),
            ("limit", limit.map(|value| value.to_string())),
        ]);
        self.http
            .send_json(
                self.http
                    .get(&format!(
                        "/rest/servicedeskapi/servicedesk/{service_desk_id}/queue/{queue_id}/issue"
                    ))?
                    .query(&query),
            )
            .await
            .or_else(jira_service_management_unavailable)
    }
}
