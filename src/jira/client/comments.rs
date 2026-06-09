use super::*;

impl JiraClient {
    pub async fn add_comment(
        &self,
        issue_key: String,
        body: String,
        visibility: Option<Value>,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let visibility = parse_optional_object(visibility, "visibility")?;
        let mut payload = json!({
            "body": comment_body_for_deployment(self.config.deployment, &body),
        });
        insert_optional(&mut payload, "visibility", visibility);
        let path = self.issue_comment_path(&issue_key);
        let comment: JiraComment = self
            .http
            .send_json(self.http.post_json(&path, &payload)?)
            .await?;

        Ok(simplify_comment(&comment))
    }

    pub async fn edit_comment(
        &self,
        issue_key: String,
        comment_id: String,
        body: String,
        visibility: Option<Value>,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let comment_id = safe_path_segment(&comment_id, "comment_id")?;
        let visibility = parse_optional_object(visibility, "visibility")?;
        let mut payload = json!({
            "body": comment_body_for_deployment(self.config.deployment, &body),
        });
        insert_optional(&mut payload, "visibility", visibility);
        let path = format!("{}/{}", self.issue_comment_path(&issue_key), comment_id);
        let comment: JiraComment = self
            .http
            .send_json(self.http.put_json(&path, &payload)?)
            .await?;

        Ok(simplify_comment(&comment))
    }

    fn issue_comment_path(&self, issue_key: &str) -> String {
        match self.config.deployment {
            JiraDeployment::Cloud => format!("/rest/api/3/issue/{issue_key}/comment"),
            JiraDeployment::ServerDataCenter => format!("/rest/api/2/issue/{issue_key}/comment"),
        }
    }
}
