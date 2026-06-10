use super::*;

impl JiraClient {
    pub async fn get_all_projects(&self, include_archived: bool) -> Result<Value, UpstreamError> {
        let query = vec![("includeArchived".to_string(), include_archived.to_string())];
        let mut projects: Vec<Value> = self
            .http
            .send_json(self.http.get("/rest/api/2/project")?.query(&query))
            .await?;
        if !self.config.projects_filter.is_empty() {
            projects.retain(|project| {
                project
                    .get("key")
                    .and_then(Value::as_str)
                    .is_some_and(|key| self.config.projects_filter.contains(key))
            });
        }
        Ok(Value::Array(projects))
    }

    pub async fn get_project_versions(&self, project_key: String) -> Result<Value, UpstreamError> {
        let project_key = safe_path_segment(&project_key, "project_key")?;
        ensure_project_allowed(&project_key, &self.config)?;
        self.http
            .send_json(
                self.http
                    .get(&format!("/rest/api/2/project/{project_key}/versions"))?,
            )
            .await
    }

    pub async fn get_project_components(
        &self,
        project_key: String,
    ) -> Result<Value, UpstreamError> {
        let project_key = safe_path_segment(&project_key, "project_key")?;
        ensure_project_allowed(&project_key, &self.config)?;
        self.http
            .send_json(
                self.http
                    .get(&format!("/rest/api/2/project/{project_key}/components"))?,
            )
            .await
    }

    pub async fn create_version(&self, mut version: Value) -> Result<Value, UpstreamError> {
        version = parse_optional_object(Some(version), "version")?.unwrap_or_else(|| json!({}));
        self.http
            .send_json(self.http.post_json("/rest/api/2/version", &version)?)
            .await
    }

    pub async fn batch_create_versions(
        &self,
        versions: Vec<Value>,
    ) -> Result<Value, UpstreamError> {
        let mut results = Vec::new();
        for version in versions {
            match self.create_version(version).await {
                Ok(value) => results.push(json!({"success": true, "version": value})),
                Err(error) => results.push(json!({"success": false, "error": error.to_string()})),
            }
        }
        Ok(json!({ "versions": results }))
    }
}
