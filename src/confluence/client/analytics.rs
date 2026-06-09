use super::*;

impl ConfluenceClient {
    pub async fn get_page_views(
        &self,
        page_id: &str,
        include_title: bool,
    ) -> Result<ConfluencePageViews, AtlassianError> {
        if self.config.deployment != ConfluenceDeployment::Cloud {
            return Err(AtlassianError::invalid_input(
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
        let mut views: ConfluencePageViews = self
            .get_json(
                &format!("/rest/api/analytics/content/{page_id}/views"),
                Vec::new(),
            )
            .await?;
        views.page_id = Some(page_id);
        views.title = title;

        Ok(views)
    }
}
