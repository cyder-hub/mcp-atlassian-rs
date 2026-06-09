use super::*;

impl ConfluenceClient {
    pub async fn get_page_comments(
        &self,
        page_id: &str,
    ) -> Result<ConfluenceCommentListResponse, AtlassianError> {
        let page_id = safe_path_segment(page_id, "page_id")?;
        self.get_json(
            &format!("/rest/api/content/{page_id}/child/comment"),
            vec![
                (
                    "expand".to_string(),
                    "body.storage,body.view,version,container,ancestors,extensions".to_string(),
                ),
                ("depth".to_string(), "all".to_string()),
            ],
        )
        .await
    }

    pub async fn add_comment(
        &self,
        page_id: &str,
        storage_body: &str,
    ) -> Result<ConfluenceComment, AtlassianError> {
        let payload = comment_payload(page_id, "page", storage_body)?;
        self.http
            .send_json(self.http.post_json("/rest/api/content", &payload)?)
            .await
    }

    pub async fn reply_to_comment(
        &self,
        comment_id: &str,
        storage_body: &str,
    ) -> Result<ConfluenceComment, AtlassianError> {
        let payload = comment_payload(comment_id, "comment", storage_body)?;
        self.http
            .send_json(self.http.post_json("/rest/api/content", &payload)?)
            .await
    }
}
