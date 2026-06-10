use super::*;

impl ConfluenceClient {
    pub async fn get_labels(
        &self,
        content_id: &str,
    ) -> Result<ConfluenceLabelListResponse, UpstreamError> {
        let content_id = safe_path_segment(content_id, "content_id")?;
        self.get_json(&format!("/rest/api/content/{content_id}/label"), Vec::new())
            .await
    }

    pub async fn add_label(
        &self,
        content_id: &str,
        name: &str,
    ) -> Result<ConfluenceLabelListResponse, UpstreamError> {
        let content_id = safe_path_segment(content_id, "content_id")?;
        let name = required_non_empty_input(name, "name")?;
        let payload = json!([{ "prefix": "global", "name": name }]);

        self.http
            .send_json_value_or_null(
                self.http
                    .post_json(&format!("/rest/api/content/{content_id}/label"), &payload)?,
            )
            .await?;
        self.get_labels(&content_id).await
    }
}
