use super::*;

const DEFAULT_ISSUE_SEARCH_FIELDS: &[&str] = &[
    "key",
    "summary",
    "status",
    "assignee",
    "reporter",
    "issuetype",
    "priority",
    "project",
];

impl JiraClient {
    pub async fn search(&self, request: SearchRequest) -> Result<Value, UpstreamError> {
        let limit = request.limit.unwrap_or(DEFAULT_LIMIT);
        let projects = self.effective_projects(request.projects_filter.as_deref())?;
        let jql = inject_project_filter(&request.jql, &projects);
        let result: JiraSearchResult = match self.config.deployment {
            JiraDeployment::Cloud => {
                if request.start_at.unwrap_or(0) > 0 && request.page_token.is_none() {
                    match self.search_cloud_legacy(&jql, &request, limit).await {
                        Ok(result) => result,
                        Err(error) if is_removed_cloud_legacy_search_error(&error) => {
                            return Err(cloud_offset_pagination_removed_error());
                        }
                        Err(error) => return Err(error),
                    }
                } else {
                    let enhanced_result = self.search_cloud_enhanced(&jql, &request, limit).await;
                    match enhanced_result {
                        Ok(result) => result,
                        Err(error)
                            if request.page_token.is_none()
                                && is_cloud_unbounded_jql_error(&error) =>
                        {
                            match self.search_cloud_legacy(&jql, &request, limit).await {
                                Ok(result) => result,
                                Err(legacy_error)
                                    if is_removed_cloud_legacy_search_error(&legacy_error) =>
                                {
                                    return Err(cloud_unbounded_jql_error(&jql));
                                }
                                Err(legacy_error) => return Err(legacy_error),
                            }
                        }
                        Err(error) => return Err(error),
                    }
                }
            }
            JiraDeployment::ServerDataCenter => {
                let body = json!({
                    "jql": jql,
                    "startAt": request.start_at.unwrap_or(0),
                    "maxResults": limit,
                    "fields": issue_search_fields_or_default(request.fields.clone()),
                    "expand": request.expand.unwrap_or_default(),
                });
                self.http
                    .send_json(self.http.post_json("/rest/api/2/search", &body)?)
                    .await?
            }
        };

        Ok(result.to_simplified_value())
    }

    pub async fn get_project_issues(
        &self,
        project_key: String,
        limit: Option<u64>,
        start_at: Option<u64>,
    ) -> Result<Value, UpstreamError> {
        let project_key = safe_path_segment(&project_key, "project_key")?;
        ensure_project_allowed(&project_key, &self.config)?;
        self.search(SearchRequest {
            jql: format!("project = \"{}\"", project_key.replace('"', "\\\"")),
            limit,
            start_at,
            ..Default::default()
        })
        .await
    }

    async fn search_cloud_enhanced(
        &self,
        jql: &str,
        request: &SearchRequest,
        limit: u64,
    ) -> Result<JiraSearchResult, UpstreamError> {
        let mut body = json!({
            "jql": jql,
            "maxResults": limit,
            "fields": issue_search_fields_or_default(request.fields.clone()),
        });
        insert_optional(
            &mut body,
            "expand",
            request
                .expand
                .clone()
                .map(|expand| Value::String(expand.join(","))),
        );
        insert_optional(
            &mut body,
            "nextPageToken",
            request.page_token.clone().map(Value::String),
        );

        self.http
            .send_json(self.http.post_json("/rest/api/3/search/jql", &body)?)
            .await
    }

    async fn search_cloud_legacy(
        &self,
        jql: &str,
        request: &SearchRequest,
        limit: u64,
    ) -> Result<JiraSearchResult, UpstreamError> {
        let mut body = json!({
            "jql": jql,
            "startAt": request.start_at.unwrap_or(0),
            "maxResults": limit,
            "fields": issue_search_fields_or_default(request.fields.clone()),
        });
        insert_optional(&mut body, "expand", request.expand.clone().map(Value::from));

        self.http
            .send_json(self.http.post_json("/rest/api/3/search", &body)?)
            .await
    }

    fn effective_projects(
        &self,
        request_projects: Option<&[String]>,
    ) -> Result<Vec<String>, UpstreamError> {
        let config_projects = self
            .config
            .projects_filter
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let request_projects = request_projects.unwrap_or_default();
        if config_projects.is_empty() {
            return Ok(request_projects.to_vec());
        }
        if request_projects.is_empty() {
            return Ok(config_projects);
        }

        let request_set = request_projects.iter().collect::<BTreeSet<_>>();
        let intersection = config_projects
            .into_iter()
            .filter(|project| request_set.contains(project))
            .collect::<Vec<_>>();
        if intersection.is_empty() {
            Err(UpstreamError::invalid_input(
                "projects_filter does not overlap with configured Jira project filter",
            ))
        } else {
            Ok(intersection)
        }
    }
}

fn issue_search_fields_or_default(fields: Option<Vec<String>>) -> Vec<String> {
    fields.unwrap_or_else(|| {
        DEFAULT_ISSUE_SEARCH_FIELDS
            .iter()
            .map(|field| (*field).to_string())
            .collect()
    })
}
