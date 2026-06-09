use super::*;

impl JiraClient {
    pub async fn get_transitions(&self, issue_key: String) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let response: JiraTransitionsResponse = self
            .http
            .send_json(
                self.http
                    .get(&format!("/rest/api/2/issue/{issue_key}/transitions"))?,
            )
            .await?;

        Ok(response.to_simplified_value())
    }

    pub async fn transition_issue(
        &self,
        issue_key: String,
        transition_id: String,
        fields: Option<Value>,
        comment: Option<String>,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let transition_id = safe_path_segment(&transition_id, "transition_id")?;
        let fields = parse_optional_object(fields, "fields")?;
        let mut payload = json!({
            "transition": {
                "id": transition_id,
            }
        });
        insert_optional(&mut payload, "fields", fields);
        if let Some(comment) = comment {
            insert_optional(
                &mut payload,
                "update",
                Some(json!({
                    "comment": [
                        {
                            "add": {
                                "body": comment_body_for_deployment(self.config.deployment, &comment)
                            }
                        }
                    ]
                })),
            );
        }

        let value: Value = self
            .http
            .send_json_value_or_null(self.http.post_json(
                &format!("/rest/api/2/issue/{issue_key}/transitions"),
                &payload,
            )?)
            .await?;
        Ok(json!({
            "issue_key": issue_key,
            "transition_id": transition_id,
            "response": value,
        }))
    }
}
