use super::*;

impl JiraClient {
    pub async fn search_fields(
        &self,
        keyword: Option<String>,
        limit: Option<u64>,
    ) -> Result<Value, UpstreamError> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT);
        let fields: Vec<JiraField> = match self.config.deployment {
            JiraDeployment::Cloud => {
                let mut query = vec![("maxResults".to_string(), limit.to_string())];
                if let Some(keyword) = keyword
                    .as_deref()
                    .map(str::trim)
                    .filter(|keyword| !keyword.is_empty())
                {
                    query.push(("query".to_string(), keyword.to_string()));
                }
                let response: JiraFieldSearchResponse = self
                    .http
                    .send_json(self.http.get("/rest/api/3/field/search")?.query(&query))
                    .await?;
                response.values
            }
            JiraDeployment::ServerDataCenter => {
                self.http
                    .send_json(self.http.get("/rest/api/2/field")?)
                    .await?
            }
        };
        let keyword = keyword.map(|keyword| keyword.to_ascii_lowercase());
        let limit = limit as usize;
        let filtered = fields
            .into_iter()
            .filter(|field| {
                keyword.as_ref().is_none_or(|keyword| {
                    field.id.to_ascii_lowercase().contains(keyword)
                        || field.name.to_ascii_lowercase().contains(keyword)
                        || field
                            .key
                            .as_ref()
                            .is_some_and(|key| key.to_ascii_lowercase().contains(keyword))
                })
            })
            .take(limit)
            .collect::<Vec<_>>();

        Ok(simplify_fields(&filtered))
    }

    pub async fn get_field_options(
        &self,
        request: FieldOptionsRequest,
    ) -> Result<Value, UpstreamError> {
        let field_id = safe_path_segment(&request.field_id, "field_id")?;
        let mut options = match self.config.deployment {
            JiraDeployment::Cloud => {
                let context_id = self
                    .resolve_cloud_field_context_id(&field_id, &request)
                    .await?;
                let query = vec![(
                    "maxResults".to_string(),
                    request.return_limit.unwrap_or(DEFAULT_LIMIT).to_string(),
                )];
                let response: JiraFieldOptionsResponse = self
                    .http
                    .send_json(
                        self.http
                            .get(&format!(
                                "/rest/api/3/field/{field_id}/context/{context_id}/option"
                            ))?
                            .query(&query),
                    )
                    .await?;
                response.values
            }
            JiraDeployment::ServerDataCenter => {
                let project_key = request
                    .project_key
                    .as_deref()
                    .ok_or_else(|| {
                        UpstreamError::invalid_input(
                            "project_key is required for Jira Server/Data Center field options",
                        )
                    })
                    .and_then(|project_key| safe_path_segment(project_key, "project_key"))?;
                let issue_type = request.issue_type.as_deref().ok_or_else(|| {
                    UpstreamError::invalid_input(
                        "issue_type is required for Jira Server/Data Center field options",
                    )
                })?;
                let query = vec![
                    ("projectKeys".to_string(), project_key),
                    ("issuetypeNames".to_string(), issue_type.to_string()),
                    (
                        "expand".to_string(),
                        "projects.issuetypes.fields".to_string(),
                    ),
                ];
                let value: Value = self
                    .http
                    .send_json(self.http.get("/rest/api/2/issue/createmeta")?.query(&query))
                    .await?;
                extract_createmeta_options(&value, &field_id)?
            }
        };

        if let Some(contains) = request.contains {
            let contains = contains.to_ascii_lowercase();
            options.retain(|option| {
                option
                    .label()
                    .is_some_and(|label| label.to_ascii_lowercase().contains(&contains))
            });
        }
        options.truncate(request.return_limit.unwrap_or(DEFAULT_LIMIT) as usize);

        Ok(simplify_options(&options, request.values_only))
    }

    async fn resolve_cloud_field_context_id(
        &self,
        field_id: &str,
        request: &FieldOptionsRequest,
    ) -> Result<String, UpstreamError> {
        if let Some(context_id) = request.context_id.as_deref() {
            return safe_path_segment(context_id, "context_id");
        }

        let project_key = request.project_key.as_deref().ok_or_else(|| {
            UpstreamError::invalid_input(
                "context_id is required for Jira Cloud field options unless project_key and issue_type are provided",
            )
        })?;
        let issue_type = request.issue_type.as_deref().ok_or_else(|| {
            UpstreamError::invalid_input(
                "context_id is required for Jira Cloud field options unless project_key and issue_type are provided",
            )
        })?;
        let (project_id, issue_type_id) = self
            .resolve_cloud_project_issue_type_ids(project_key, issue_type)
            .await?;
        let query = vec![("maxResults".to_string(), "1".to_string())];
        let body = json!({
            "mappings": [{
                "projectId": project_id,
                "issueTypeId": issue_type_id,
            }]
        });
        let response: JiraPaginatedValues = self
            .http
            .send_json(
                self.http
                    .post_json(
                        &format!("/rest/api/3/field/{field_id}/context/mapping"),
                        &body,
                    )?
                    .query(&query),
            )
            .await?;

        let mapping = response.values.first().ok_or_else(|| {
            UpstreamError::unexpected_shape(format!(
                "field `{field_id}` context mapping response did not include a mapping"
            ))
        })?;
        let Some(context_id) = field_context_mapping_context_id(mapping)? else {
            return Err(UpstreamError::invalid_input(format!(
                "No Jira Cloud field context applies to field `{field_id}` for project `{project_key}` and issue type `{issue_type}`"
            )));
        };

        safe_path_segment(&context_id, "context_id")
    }

    async fn resolve_cloud_project_issue_type_ids(
        &self,
        project_key: &str,
        issue_type: &str,
    ) -> Result<(String, String), UpstreamError> {
        let project_key = safe_path_segment(project_key, "project_key")?;
        let issue_type = issue_type.trim();
        if issue_type.is_empty() {
            return Err(UpstreamError::invalid_input("issue_type must not be empty"));
        }

        let project: Value = self
            .http
            .send_json(
                self.http
                    .get(&format!("/rest/api/3/project/{project_key}"))?,
            )
            .await?;
        let project_id = field_value_id(&project, "id").ok_or_else(|| {
            UpstreamError::unexpected_shape(format!(
                "project `{project_key}` response did not include an id"
            ))
        })?;
        let issue_types = project
            .get("issueTypes")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                UpstreamError::unexpected_shape(format!(
                    "project `{project_key}` response did not include issueTypes"
                ))
            })?;
        let issue_type_id = issue_types
            .iter()
            .find(|candidate| cloud_issue_type_matches(candidate, issue_type))
            .and_then(|candidate| field_value_id(candidate, "id"))
            .ok_or_else(|| {
                UpstreamError::invalid_input(format!(
                    "issue_type `{issue_type}` was not found in Jira Cloud project `{project_key}`"
                ))
            })?;

        Ok((project_id, issue_type_id))
    }
}
