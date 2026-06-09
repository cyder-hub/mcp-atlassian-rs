use super::*;

impl JiraClient {
    pub async fn get_agile_boards(
        &self,
        project_key: Option<String>,
        board_type: Option<String>,
        start_at: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Value, AtlassianError> {
        let query = optional_query_params([
            ("projectKeyOrId", project_key),
            ("type", board_type),
            ("startAt", start_at.map(|value| value.to_string())),
            ("maxResults", limit.map(|value| value.to_string())),
        ]);
        self.http
            .send_json(self.http.get("/rest/agile/1.0/board")?.query(&query))
            .await
            .or_else(jira_software_agile_unavailable)
    }

    pub async fn get_board_issues(
        &self,
        board_id: u64,
        jql: Option<String>,
        fields: Option<Vec<String>>,
        start_at: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Value, AtlassianError> {
        let query = optional_query_params([
            ("jql", jql),
            ("fields", fields.map(|fields| fields.join(","))),
            ("startAt", start_at.map(|value| value.to_string())),
            ("maxResults", limit.map(|value| value.to_string())),
        ]);
        self.http
            .send_json(
                self.http
                    .get(&format!("/rest/agile/1.0/board/{board_id}/issue"))?
                    .query(&query),
            )
            .await
            .or_else(jira_software_agile_unavailable)
    }

    pub async fn get_sprints_from_board(
        &self,
        board_id: u64,
        state: Option<Vec<String>>,
        start_at: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Value, AtlassianError> {
        let query = optional_query_params([
            ("state", state.map(|state| state.join(","))),
            ("startAt", start_at.map(|value| value.to_string())),
            ("maxResults", limit.map(|value| value.to_string())),
        ]);
        self.http
            .send_json(
                self.http
                    .get(&format!("/rest/agile/1.0/board/{board_id}/sprint"))?
                    .query(&query),
            )
            .await
            .or_else(jira_software_agile_unavailable)
    }

    pub async fn get_sprint_issues(
        &self,
        sprint_id: u64,
        fields: Option<Vec<String>>,
        start_at: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Value, AtlassianError> {
        let query = optional_query_params([
            ("fields", fields.map(|fields| fields.join(","))),
            ("startAt", start_at.map(|value| value.to_string())),
            ("maxResults", limit.map(|value| value.to_string())),
        ]);
        self.http
            .send_json(
                self.http
                    .get(&format!("/rest/agile/1.0/sprint/{sprint_id}/issue"))?
                    .query(&query),
            )
            .await
            .or_else(jira_software_agile_unavailable)
    }

    pub async fn create_sprint(&self, payload: Value) -> Result<Value, AtlassianError> {
        self.http
            .send_json(self.http.post_json("/rest/agile/1.0/sprint", &payload)?)
            .await
    }

    pub async fn update_sprint(
        &self,
        sprint_id: u64,
        payload: Value,
    ) -> Result<Value, AtlassianError> {
        self.http
            .send_json(
                self.http
                    .put_json(&format!("/rest/agile/1.0/sprint/{sprint_id}"), &payload)?,
            )
            .await
    }

    pub async fn add_issues_to_sprint(
        &self,
        sprint_id: u64,
        issue_keys: Vec<String>,
    ) -> Result<Value, AtlassianError> {
        self.http
            .send_json_value_or_null(self.http.post_json(
                &format!("/rest/agile/1.0/sprint/{sprint_id}/issue"),
                &json!({ "issues": issue_keys }),
            )?)
            .await
    }
}
