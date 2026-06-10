use super::*;

const PAGE_CHILD_RECOUNT_LIMIT: u64 = 100;
const MAX_PAGE_CHILD_RECOUNT_REQUESTS: u64 = 20;

impl ConfluenceClient {
    pub async fn get_page_by_id(
        &self,
        page_id: &str,
        expand: &[&str],
    ) -> Result<ConfluencePage, UpstreamError> {
        let page_id = safe_path_segment(page_id, "page_id")?;
        let mut query = Vec::new();
        if !expand.is_empty() {
            query.push(("expand".to_string(), expand.join(",")));
        }

        self.get_json(&format!("/rest/api/content/{page_id}"), query)
            .await
    }

    pub async fn get_page_history(
        &self,
        page_id: &str,
        version: u64,
        expand: &[&str],
    ) -> Result<ConfluencePage, UpstreamError> {
        if version == 0 {
            return Err(UpstreamError::invalid_input("version must be positive"));
        }
        let page_id = safe_path_segment(page_id, "page_id")?;
        let mut query = vec![
            ("status".to_string(), "historical".to_string()),
            ("version".to_string(), version.to_string()),
        ];
        if !expand.is_empty() {
            query.push(("expand".to_string(), expand.join(",")));
        }

        self.get_json(&format!("/rest/api/content/{page_id}"), query)
            .await
    }

    pub async fn get_page_by_title(
        &self,
        space_key: &str,
        title: &str,
        expand: &[&str],
    ) -> Result<Option<ConfluencePage>, UpstreamError> {
        let space_key = required_non_empty_input(space_key, "space_key")?;
        let title = required_non_empty_input(title, "title")?;
        let mut query = vec![
            ("spaceKey".to_string(), space_key),
            ("title".to_string(), title),
            ("limit".to_string(), "1".to_string()),
        ];
        if !expand.is_empty() {
            query.push(("expand".to_string(), expand.join(",")));
        }

        let response: ConfluencePageListResponse =
            self.get_json("/rest/api/content", query).await?;
        Ok(response.results.into_iter().next())
    }

    pub async fn get_page_children_by_type(
        &self,
        page_id: &str,
        child_type: &str,
        start: Option<u64>,
        limit: Option<u64>,
        expand: &[&str],
    ) -> Result<ConfluencePageListResponse, UpstreamError> {
        let page_id = safe_path_segment(page_id, "page_id")?;
        let child_type = safe_path_segment(child_type, "child_type")?;
        let mut query = vec![
            ("start".to_string(), start.unwrap_or(0).to_string()),
            (
                "limit".to_string(),
                limit.unwrap_or(DEFAULT_LIMIT).to_string(),
            ),
        ];
        if !expand.is_empty() {
            query.push(("expand".to_string(), expand.join(",")));
        }

        self.get_json(
            &format!("/rest/api/content/{page_id}/child/{child_type}"),
            query,
        )
        .await
    }

    pub async fn get_page_children(
        &self,
        page_id: &str,
        start: Option<u64>,
        limit: Option<u64>,
        expand: &[&str],
        include_folders: bool,
    ) -> Result<ConfluencePageChildrenResponse, UpstreamError> {
        let requested_start = start.unwrap_or(0);
        let requested_limit = limit.unwrap_or(DEFAULT_LIMIT);
        let page_response = self
            .get_page_children_by_type(page_id, "page", start, Some(requested_limit), expand)
            .await?;
        let page_query =
            children_query_stats("page", requested_start, requested_limit, &page_response);
        let page_start = page_response.start.unwrap_or(requested_start);
        let mut children = page_response.results;
        children.truncate(requested_limit as usize);
        let page_results = children.len();
        let mut folder_query = None;
        let mut folder_results = 0;

        let remaining_limit = requested_limit.saturating_sub(children.len() as u64);
        let page_end = page_start.saturating_add(page_results as u64);
        if include_folders && remaining_limit > 0 {
            let folder_start = if page_results == 0 && requested_start > 0 {
                let page_count = self
                    .count_page_children_before(page_id, requested_start)
                    .await?;
                requested_start.saturating_sub(page_count)
            } else {
                requested_start.saturating_sub(page_end)
            };
            if let Ok(folder_response) = self
                .get_page_children_by_type(
                    page_id,
                    "folder",
                    Some(folder_start),
                    Some(remaining_limit),
                    expand,
                )
                .await
            {
                let stats =
                    children_query_stats("folder", folder_start, remaining_limit, &folder_response);
                let before_len = children.len();
                children.extend(folder_response.results);
                children.truncate(requested_limit as usize);
                folder_results = children.len().saturating_sub(before_len);
                folder_query = Some(stats);
            }
        }

        Ok(ConfluencePageChildrenResponse {
            results: children,
            page_results,
            folder_results,
            page_query,
            folder_query,
        })
    }

    async fn count_page_children_before(
        &self,
        page_id: &str,
        before_start: u64,
    ) -> Result<u64, UpstreamError> {
        let mut counted = 0;
        let mut start = 0;
        let mut request_count = 0;

        while counted < before_start {
            if request_count >= MAX_PAGE_CHILD_RECOUNT_REQUESTS {
                return Err(UpstreamError::invalid_input(format!(
                    "start is too large to combine page and folder children exactly; page-child recount is capped at {MAX_PAGE_CHILD_RECOUNT_REQUESTS} requests"
                )));
            }
            let remaining = before_start - counted;
            let fetch_limit = PAGE_CHILD_RECOUNT_LIMIT.min(remaining);
            let response = self
                .get_page_children_by_type(page_id, "page", Some(start), Some(fetch_limit), &[])
                .await?;
            request_count += 1;
            let result_count = response.results.len() as u64;
            counted = counted.saturating_add(result_count);

            if result_count == 0 || !response.has_next_link() {
                break;
            }

            match response.next_start() {
                Some(next_start) if next_start > start => start = next_start,
                _ => start = start.saturating_add(result_count),
            }
        }

        Ok(counted.min(before_start))
    }

    pub async fn get_space_pages(
        &self,
        space_key: &str,
        start: Option<u64>,
        limit: Option<u64>,
        expand: &[&str],
    ) -> Result<ConfluencePageListResponse, UpstreamError> {
        let space_key = required_non_empty_input(space_key, "space_key")?;
        let mut query = vec![
            ("spaceKey".to_string(), space_key),
            ("type".to_string(), "page".to_string()),
            ("start".to_string(), start.unwrap_or(0).to_string()),
            (
                "limit".to_string(),
                limit.unwrap_or(DEFAULT_LIMIT).to_string(),
            ),
        ];
        if !expand.is_empty() {
            query.push(("expand".to_string(), expand.join(",")));
        }

        self.get_json("/rest/api/content", query).await
    }

    pub async fn create_page(
        &self,
        space_key: &str,
        title: &str,
        storage_body: &str,
        parent_id: Option<&str>,
    ) -> Result<ConfluencePage, UpstreamError> {
        let payload = page_write_payload(
            None,
            required_non_empty_input(space_key, "space_key")?,
            required_non_empty_input(title, "title")?,
            storage_body,
            parent_id,
            None,
            None,
        )?;

        self.http
            .send_json(self.http.post_json("/rest/api/content", &payload)?)
            .await
    }

    pub async fn update_page(
        &self,
        page_id: &str,
        title: &str,
        storage_body: &str,
        parent_id: Option<&str>,
        is_minor_edit: bool,
        version_comment: Option<&str>,
    ) -> Result<ConfluencePage, UpstreamError> {
        let current = self
            .get_page_by_id(page_id, &["version", "space", "body.storage"])
            .await?;
        let space_key = current
            .space
            .as_ref()
            .and_then(|space| space.key.as_deref())
            .ok_or_else(|| UpstreamError::unexpected_shape("page response is missing space.key"))?;
        let next_version = current
            .version
            .as_ref()
            .and_then(|version| version.number)
            .unwrap_or(1)
            + 1;

        self.update_page_with_space(ConfluenceUpdatePageWithSpaceRequest {
            page_id,
            space_key,
            title,
            storage_body,
            parent_id,
            version: next_version,
            is_minor_edit,
            version_comment,
        })
        .await
    }

    pub async fn delete_page(&self, page_id: &str) -> Result<serde_json::Value, UpstreamError> {
        let page_id = safe_path_segment(page_id, "page_id")?;
        self.http
            .send_json_value_or_null(self.http.delete(&format!("/rest/api/content/{page_id}"))?)
            .await
    }

    pub async fn set_page_emoji_best_effort(
        &self,
        page_id: &str,
        emoji: Option<&str>,
    ) -> ConfluenceEmojiStatus {
        let Some(emoji) = optional_non_empty_input(emoji) else {
            return ConfluenceEmojiStatus::not_requested();
        };
        let page_id = match safe_path_segment(page_id, "page_id") {
            Ok(page_id) => page_id,
            Err(error) => return ConfluenceEmojiStatus::failed(emoji, error.to_string()),
        };
        let payload = json!({ "value": emoji });
        let builder = match self.http.put_json(
            &format!("/rest/api/content/{page_id}/property/emoji-title-published"),
            &payload,
        ) {
            Ok(builder) => builder,
            Err(error) => return ConfluenceEmojiStatus::failed(emoji, error.to_string()),
        };

        match self.http.send_json_value_or_null(builder).await {
            Ok(_) => ConfluenceEmojiStatus::applied(emoji),
            Err(error) => ConfluenceEmojiStatus::failed(emoji, error.to_string()),
        }
    }

    pub async fn move_page(
        &self,
        page_id: &str,
        target_parent_id: Option<&str>,
        target_space_key: Option<&str>,
        position: Option<&str>,
    ) -> Result<ConfluencePage, UpstreamError> {
        let page_id = safe_path_segment(page_id, "page_id")?;
        let target_parent_id = optional_non_empty_input(target_parent_id);
        let target_space_key = optional_non_empty_input(target_space_key);
        let position = position
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("append");
        if !matches!(position, "append" | "above" | "below") {
            return Err(UpstreamError::invalid_input(
                "position must be append, above, or below",
            ));
        }
        if target_parent_id.is_none() && target_space_key.is_none() {
            return Err(UpstreamError::invalid_input(
                "At least one of target_parent_id or target_space_key must be provided",
            ));
        }

        if matches!(position, "above" | "below") {
            let target_id = target_parent_id.ok_or_else(|| {
                UpstreamError::invalid_input(
                    "target_parent_id is required when position is above or below",
                )
            })?;
            let target_id = safe_path_segment(&target_id, "target_parent_id")?;
            self.http
                .send_json_value_or_null(self.http.put_json(
                    &format!("/rest/api/content/{page_id}/move/{position}/{target_id}"),
                    &serde_json::json!({}),
                )?)
                .await?;
            return self
                .get_page_by_id(&page_id, &["body.storage", "version", "space", "ancestors"])
                .await;
        }

        let current = self
            .get_page_by_id(&page_id, &["body.storage", "version", "space"])
            .await?;
        let title = current
            .title
            .as_deref()
            .ok_or_else(|| UpstreamError::unexpected_shape("page response is missing title"))?;
        let storage_body = body_value_as_storage(&current.body).unwrap_or_default();
        let current_space_key = current
            .space
            .as_ref()
            .and_then(|space| space.key.as_deref())
            .ok_or_else(|| UpstreamError::unexpected_shape("page response is missing space.key"))?;
        let space_key = target_space_key.as_deref().unwrap_or(current_space_key);
        let next_version = current
            .version
            .as_ref()
            .and_then(|version| version.number)
            .unwrap_or(1)
            + 1;

        self.update_page_with_space(ConfluenceUpdatePageWithSpaceRequest {
            page_id: &page_id,
            space_key,
            title,
            storage_body: &storage_body,
            parent_id: target_parent_id.as_deref(),
            version: next_version,
            is_minor_edit: false,
            version_comment: None,
        })
        .await
    }

    async fn update_page_with_space(
        &self,
        request: ConfluenceUpdatePageWithSpaceRequest<'_>,
    ) -> Result<ConfluencePage, UpstreamError> {
        let page_id = safe_path_segment(request.page_id, "page_id")?;
        let payload = page_write_payload(
            Some(page_id.as_str()),
            required_non_empty_input(request.space_key, "space_key")?,
            required_non_empty_input(request.title, "title")?,
            request.storage_body,
            request.parent_id,
            Some(request.version),
            Some((request.is_minor_edit, request.version_comment)),
        )?;

        self.http
            .send_json(
                self.http
                    .put_json(&format!("/rest/api/content/{page_id}"), &payload)?,
            )
            .await
    }

    pub async fn search_cql(
        &self,
        cql: &str,
        start: Option<u64>,
        limit: Option<u64>,
    ) -> Result<ConfluenceSearchResponse, UpstreamError> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT);
        let start = start.unwrap_or(0);
        self.get_json(
            "/rest/api/content/search",
            vec![
                ("cql".to_string(), cql.to_string()),
                ("start".to_string(), start.to_string()),
                ("limit".to_string(), limit.to_string()),
            ],
        )
        .await
    }

    pub async fn search_content(
        &self,
        query: &str,
        limit: Option<u64>,
        spaces_filter: Option<&str>,
    ) -> Result<ConfluenceSearchResponse, UpstreamError> {
        let query = required_non_empty_input(query, "query")?;
        let limit = search_limit(limit)?;

        if is_simple_search_query(&query) {
            let escaped_query = escape_cql_string(&query);
            let site_search_cql = format!("siteSearch ~ \"{escaped_query}\"");

            return match self
                .search_cql_with_spaces_filter(&site_search_cql, Some(limit), spaces_filter)
                .await
            {
                Ok(response) => Ok(response),
                Err(UpstreamError::HttpStatus { status: 400, .. }) => {
                    let text_search_cql = format!("text ~ \"{escaped_query}\"");
                    self.search_cql_with_spaces_filter(&text_search_cql, Some(limit), spaces_filter)
                        .await
                }
                Err(error) => Err(error),
            };
        }

        self.search_cql_with_spaces_filter(&query, Some(limit), spaces_filter)
            .await
    }

    async fn search_cql_with_spaces_filter(
        &self,
        cql: &str,
        limit: Option<u64>,
        spaces_filter: Option<&str>,
    ) -> Result<ConfluenceSearchResponse, UpstreamError> {
        let cql = apply_spaces_filter(cql, spaces_filter_values(spaces_filter, &self.config));
        self.search_cql(&cql, Some(0), limit).await
    }
}
