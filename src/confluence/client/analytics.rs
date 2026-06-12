use super::*;

impl ConfluenceClient {
    pub async fn get_page_views(
        &self,
        page_id: &str,
        include_title: bool,
        from_date: Option<&str>,
        to_date: Option<&str>,
    ) -> Result<ConfluencePageViews, UpstreamError> {
        if self.config.deployment != ConfluenceDeployment::Cloud {
            return Err(UpstreamError::invalid_input(
                "Page view analytics is only available for Confluence Cloud. Server/Data Center instances do not support the Analytics API.",
            ));
        }
        let page_id = safe_path_segment(page_id, "page_id")?;
        let title = if include_title {
            self.get_page_by_id(&page_id, &["title"])
                .await
                .ok()
                .and_then(|page| page.title)
        } else {
            None
        };
        let mut query = Vec::new();
        if let Some(from_date) = from_date.filter(|value| !value.trim().is_empty()) {
            query.push(("from".to_string(), from_date.to_string()));
        }
        if let Some(to_date) = to_date.filter(|value| !value.trim().is_empty()) {
            query.push(("to".to_string(), to_date.to_string()));
        }
        let mut views: ConfluencePageViews = self
            .get_json(
                &format!("/rest/api/analytics/content/{page_id}/views"),
                query,
            )
            .await?;
        views.page_id = Some(page_id);
        views.title = title;

        Ok(views)
    }
}
