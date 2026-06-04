use std::collections::BTreeSet;

use reqwest::Url;
use serde_json::{Map, Value, json};

use crate::{
    atlassian::{
        error::AtlassianError,
        http::{AtlassianHttpClient, DownloadedContent},
    },
    jira::{
        config::{JiraConfig, JiraDeployment},
        formatting::{
            base64_encode, comment_body_for_deployment, ensure_issue_allowed,
            ensure_project_allowed, inject_project_filter, merge_optional_objects,
            parse_optional_object, redact_url_query, safe_path_segment,
        },
        models::{
            JiraAttachment, JiraComment, JiraField, JiraFieldOption, JiraFieldOptionsResponse,
            JiraFieldSearchResponse, JiraIssue, JiraOperationResult, JiraPaginatedValues,
            JiraSearchResult, JiraTransitionsResponse, simplify_comment, simplify_fields,
            simplify_options,
        },
    },
};

pub const DEFAULT_LIMIT: u64 = 50;
pub const DEFAULT_ATTACHMENT_MAX_BYTES: u64 = 1_048_576;
const ATLASSIAN_API_BASE_URL: &str = "https://api.atlassian.com";

#[derive(Clone, Debug)]
pub struct JiraClient {
    config: JiraConfig,
    http: AtlassianHttpClient,
    atlassian_api_http: AtlassianHttpClient,
}

#[derive(Debug, Clone, Default)]
pub struct GetIssueRequest {
    pub issue_key: String,
    pub fields: Option<Vec<String>>,
    pub expand: Option<Vec<String>>,
    pub comment_limit: Option<u64>,
    pub properties: Option<Vec<String>>,
    pub update_history: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct AttachmentFetchOptions {
    pub attachment_ids: Option<Vec<String>>,
    pub include_content: bool,
    pub images_only: bool,
    pub max_bytes: u64,
}

impl Default for AttachmentFetchOptions {
    fn default() -> Self {
        Self {
            attachment_ids: None,
            include_content: false,
            images_only: false,
            max_bytes: DEFAULT_ATTACHMENT_MAX_BYTES,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchRequest {
    pub jql: String,
    pub fields: Option<Vec<String>>,
    pub limit: Option<u64>,
    pub start_at: Option<u64>,
    pub projects_filter: Option<Vec<String>>,
    pub expand: Option<Vec<String>>,
    pub page_token: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct FieldOptionsRequest {
    pub field_id: String,
    pub context_id: Option<String>,
    pub project_key: Option<String>,
    pub issue_type: Option<String>,
    pub contains: Option<String>,
    pub return_limit: Option<u64>,
    pub values_only: bool,
}

impl JiraClient {
    pub fn new(config: JiraConfig) -> Result<Self, AtlassianError> {
        let http = AtlassianHttpClient::new(
            &config.base_url,
            config.auth.clone(),
            config.timeout_seconds,
            config.ssl_verify,
        )?;
        let atlassian_api_http = AtlassianHttpClient::new(
            &atlassian_api_base_url(&config),
            config.auth.clone(),
            config.timeout_seconds,
            config.ssl_verify,
        )?;
        Ok(Self {
            config,
            http,
            atlassian_api_http,
        })
    }

    pub async fn get_issue(&self, request: GetIssueRequest) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&request.issue_key, &self.config)?;
        let issue_key = safe_path_segment(&request.issue_key, "issue_key")?;
        let mut query = optional_query_params([
            ("fields", request.fields.map(|fields| fields.join(","))),
            ("expand", request.expand.map(|expand| expand.join(","))),
            (
                "properties",
                request.properties.map(|properties| properties.join(",")),
            ),
            (
                "updateHistory",
                request.update_history.map(|value| value.to_string()),
            ),
        ]);

        if let Some(comment_limit) = request.comment_limit {
            query.push(("commentLimit".to_string(), comment_limit.to_string()));
        }

        let issue: JiraIssue = self
            .http
            .send_json(
                self.http
                    .get(&format!("/rest/api/2/issue/{issue_key}"))?
                    .query(&query),
            )
            .await?;
        Ok(issue.to_simplified_value())
    }

    pub async fn search(&self, request: SearchRequest) -> Result<Value, AtlassianError> {
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
                    "fields": request.fields.unwrap_or_default(),
                    "expand": request.expand.unwrap_or_default().join(","),
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
    ) -> Result<Value, AtlassianError> {
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

    pub async fn search_fields(
        &self,
        keyword: Option<String>,
        limit: Option<u64>,
    ) -> Result<Value, AtlassianError> {
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

    async fn search_cloud_enhanced(
        &self,
        jql: &str,
        request: &SearchRequest,
        limit: u64,
    ) -> Result<JiraSearchResult, AtlassianError> {
        let mut body = json!({
            "jql": jql,
            "maxResults": limit,
        });
        insert_optional(&mut body, "fields", request.fields.clone().map(Value::from));
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
    ) -> Result<JiraSearchResult, AtlassianError> {
        let mut body = json!({
            "jql": jql,
            "startAt": request.start_at.unwrap_or(0),
            "maxResults": limit,
        });
        insert_optional(&mut body, "fields", request.fields.clone().map(Value::from));
        insert_optional(&mut body, "expand", request.expand.clone().map(Value::from));

        self.http
            .send_json(self.http.post_json("/rest/api/3/search", &body)?)
            .await
    }

    pub async fn get_field_options(
        &self,
        request: FieldOptionsRequest,
    ) -> Result<Value, AtlassianError> {
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
                        AtlassianError::invalid_input(
                            "project_key is required for Jira Server/Data Center field options",
                        )
                    })
                    .and_then(|project_key| safe_path_segment(project_key, "project_key"))?;
                let issue_type = request.issue_type.as_deref().ok_or_else(|| {
                    AtlassianError::invalid_input(
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
    ) -> Result<String, AtlassianError> {
        if let Some(context_id) = request.context_id.as_deref() {
            return safe_path_segment(context_id, "context_id");
        }

        let project_key = request.project_key.as_deref().ok_or_else(|| {
            AtlassianError::invalid_input(
                "context_id is required for Jira Cloud field options unless project_key and issue_type are provided",
            )
        })?;
        let issue_type = request.issue_type.as_deref().ok_or_else(|| {
            AtlassianError::invalid_input(
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
            AtlassianError::unexpected_shape(format!(
                "field `{field_id}` context mapping response did not include a mapping"
            ))
        })?;
        let Some(context_id) = field_context_mapping_context_id(mapping)? else {
            return Err(AtlassianError::invalid_input(format!(
                "No Jira Cloud field context applies to field `{field_id}` for project `{project_key}` and issue type `{issue_type}`"
            )));
        };

        safe_path_segment(&context_id, "context_id")
    }

    async fn resolve_cloud_project_issue_type_ids(
        &self,
        project_key: &str,
        issue_type: &str,
    ) -> Result<(String, String), AtlassianError> {
        let project_key = safe_path_segment(project_key, "project_key")?;
        let issue_type = issue_type.trim();
        if issue_type.is_empty() {
            return Err(AtlassianError::invalid_input(
                "issue_type must not be empty",
            ));
        }

        let project: Value = self
            .http
            .send_json(
                self.http
                    .get(&format!("/rest/api/3/project/{project_key}"))?,
            )
            .await?;
        let project_id = field_value_id(&project, "id").ok_or_else(|| {
            AtlassianError::unexpected_shape(format!(
                "project `{project_key}` response did not include an id"
            ))
        })?;
        let issue_types = project
            .get("issueTypes")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                AtlassianError::unexpected_shape(format!(
                    "project `{project_key}` response did not include issueTypes"
                ))
            })?;
        let issue_type_id = issue_types
            .iter()
            .find(|candidate| cloud_issue_type_matches(candidate, issue_type))
            .and_then(|candidate| field_value_id(candidate, "id"))
            .ok_or_else(|| {
                AtlassianError::invalid_input(format!(
                    "issue_type `{issue_type}` was not found in Jira Cloud project `{project_key}`"
                ))
            })?;

        Ok((project_id, issue_type_id))
    }

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

    pub async fn create_issue(&self, fields: Value) -> Result<Value, AtlassianError> {
        let fields = parse_optional_object(Some(fields), "fields")?.unwrap_or_else(|| json!({}));
        let path = match self.config.deployment {
            JiraDeployment::Cloud => "/rest/api/3/issue",
            JiraDeployment::ServerDataCenter => "/rest/api/2/issue",
        };
        let issue: JiraIssue = self
            .http
            .send_json(self.http.post_json(path, &json!({ "fields": fields }))?)
            .await?;
        Ok(
            JiraOperationResult::success("Issue created successfully", issue.to_simplified_value())
                .to_simplified_value(),
        )
    }

    pub async fn batch_create_issues(
        &self,
        issues: Vec<Value>,
        validate_only: bool,
    ) -> Result<Value, AtlassianError> {
        let body = json!({
            "issueUpdates": issues,
            "validateOnly": validate_only,
        });
        let value: Value = self
            .http
            .send_json_value_or_null(self.http.post_json("/rest/api/2/issue/bulk", &body)?)
            .await?;
        Ok(
            JiraOperationResult::success("Issues processed successfully", value)
                .to_simplified_value(),
        )
    }

    pub async fn batch_get_changelogs(
        &self,
        issue_ids_or_keys: Vec<String>,
        fields: Option<Vec<String>>,
        limit: Option<i64>,
    ) -> Result<Value, AtlassianError> {
        if self.config.deployment != JiraDeployment::Cloud {
            return Ok(JiraOperationResult::product_unavailable(
                "Jira Cloud changelog bulk endpoint",
                "Batch get issue changelogs is only available on Jira Cloud.",
            )
            .to_simplified_value());
        }

        let mut body = json!({
            "issueIdsOrKeys": issue_ids_or_keys,
        });
        insert_optional(&mut body, "fieldIds", fields.map(Value::from));
        insert_optional(&mut body, "maxResults", limit.map(Value::from));
        let value: Value = self
            .http
            .send_json(
                self.http
                    .post_json("/rest/api/3/changelog/bulkfetch", &body)?,
            )
            .await?;
        Ok(value)
    }

    pub async fn update_issue(
        &self,
        issue_key: String,
        fields: Value,
        additional_fields: Option<Value>,
        notify_users: Option<bool>,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let fields = merge_optional_objects(fields, additional_fields, "additional_fields")?;
        let mut query = Vec::new();
        if let Some(notify_users) = notify_users {
            query.push(("notifyUsers".to_string(), notify_users.to_string()));
        }
        let path = match self.config.deployment {
            JiraDeployment::Cloud => format!("/rest/api/3/issue/{issue_key}"),
            JiraDeployment::ServerDataCenter => format!("/rest/api/2/issue/{issue_key}"),
        };
        let response = self
            .http
            .send_json_value_or_null(
                self.http
                    .put_json(&path, &json!({ "fields": fields }))?
                    .query(&query),
            )
            .await?;
        Ok(
            JiraOperationResult::success("Issue updated successfully", response)
                .to_simplified_value(),
        )
    }

    pub async fn delete_issue(
        &self,
        issue_key: String,
        delete_subtasks: bool,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let query = vec![("deleteSubtasks".to_string(), delete_subtasks.to_string())];
        let response = self
            .http
            .send_json_value_or_null(
                self.http
                    .delete(&format!("/rest/api/2/issue/{issue_key}"))?
                    .query(&query),
            )
            .await?;
        Ok(json!({
            "success": true,
            "issue_key": issue_key,
            "response": response,
        }))
    }

    pub async fn get_all_projects(&self, include_archived: bool) -> Result<Value, AtlassianError> {
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

    pub async fn get_project_versions(&self, project_key: String) -> Result<Value, AtlassianError> {
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
    ) -> Result<Value, AtlassianError> {
        let project_key = safe_path_segment(&project_key, "project_key")?;
        ensure_project_allowed(&project_key, &self.config)?;
        self.http
            .send_json(
                self.http
                    .get(&format!("/rest/api/2/project/{project_key}/components"))?,
            )
            .await
    }

    pub async fn create_version(&self, mut version: Value) -> Result<Value, AtlassianError> {
        version = parse_optional_object(Some(version), "version")?.unwrap_or_else(|| json!({}));
        self.http
            .send_json(self.http.post_json("/rest/api/2/version", &version)?)
            .await
    }

    pub async fn batch_create_versions(
        &self,
        versions: Vec<Value>,
    ) -> Result<Value, AtlassianError> {
        let mut results = Vec::new();
        for version in versions {
            match self.create_version(version).await {
                Ok(value) => results.push(json!({"success": true, "version": value})),
                Err(error) => results.push(json!({"success": false, "error": error.to_string()})),
            }
        }
        Ok(json!({ "versions": results }))
    }

    pub async fn get_user_profile(&self, user_identifier: String) -> Result<Value, AtlassianError> {
        let query_key = match self.config.deployment {
            JiraDeployment::Cloud => "accountId",
            JiraDeployment::ServerDataCenter => "username",
        };
        self.http
            .send_json(
                self.http
                    .get("/rest/api/2/user")?
                    .query(&[(query_key, user_identifier)]),
            )
            .await
    }

    pub async fn get_issue_watchers(&self, issue_key: String) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        self.http
            .send_json(
                self.http
                    .get(&format!("/rest/api/2/issue/{issue_key}/watchers"))?,
            )
            .await
    }

    pub async fn add_watcher(
        &self,
        issue_key: String,
        user_identifier: String,
    ) -> Result<Value, AtlassianError> {
        self.watcher_mutation(issue_key, user_identifier, true)
            .await
    }

    pub async fn remove_watcher(
        &self,
        issue_key: String,
        user_identifier: String,
    ) -> Result<Value, AtlassianError> {
        self.watcher_mutation(issue_key, user_identifier, false)
            .await
    }

    pub async fn get_worklog(
        &self,
        issue_key: String,
        start_at: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let query = optional_query_params([
            ("startAt", start_at.map(|value| value.to_string())),
            ("maxResults", limit.map(|value| value.to_string())),
        ]);
        self.http
            .send_json(
                self.http
                    .get(&format!("/rest/api/2/issue/{issue_key}/worklog"))?
                    .query(&query),
            )
            .await
    }

    pub async fn add_worklog(
        &self,
        issue_key: String,
        payload: Value,
        query: Vec<(String, String)>,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        self.http
            .send_json(
                self.http
                    .post_json(&format!("/rest/api/2/issue/{issue_key}/worklog"), &payload)?
                    .query(&query),
            )
            .await
    }

    pub async fn get_link_types(&self) -> Result<Value, AtlassianError> {
        self.http
            .send_json(self.http.get("/rest/api/2/issueLinkType")?)
            .await
    }

    pub async fn link_to_epic(
        &self,
        issue_key: String,
        epic_key: String,
    ) -> Result<Value, AtlassianError> {
        self.update_issue(
            issue_key,
            json!({ "parent": { "key": epic_key } }),
            None,
            None,
        )
        .await
    }

    pub async fn create_issue_link(&self, payload: Value) -> Result<Value, AtlassianError> {
        self.http
            .send_json_value_or_null(self.http.post_json("/rest/api/2/issueLink", &payload)?)
            .await
    }

    pub async fn create_remote_issue_link(
        &self,
        issue_key: String,
        payload: Value,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let path = match self.config.deployment {
            JiraDeployment::Cloud => format!("/rest/api/3/issue/{issue_key}/remotelink"),
            JiraDeployment::ServerDataCenter => {
                format!("/rest/api/2/issue/{issue_key}/remotelink")
            }
        };
        self.http
            .send_json_value_or_null(self.http.post_json(&path, &payload)?)
            .await
    }

    pub async fn remove_issue_link(&self, link_id: String) -> Result<Value, AtlassianError> {
        let link_id = safe_path_segment(&link_id, "link_id")?;
        let response = self
            .http
            .send_json_value_or_null(
                self.http
                    .delete(&format!("/rest/api/2/issueLink/{link_id}"))?,
            )
            .await?;
        Ok(json!({ "success": true, "link_id": link_id, "response": response }))
    }

    pub async fn get_issue_attachments(
        &self,
        issue_key: String,
    ) -> Result<Vec<JiraAttachment>, AtlassianError> {
        let issue = self
            .get_issue(GetIssueRequest {
                issue_key,
                fields: Some(vec!["attachment".to_string()]),
                ..Default::default()
            })
            .await?;
        issue
            .get("fields")
            .and_then(|fields| fields.get("attachment"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(serde_json::from_value)
            .collect::<Result<Vec<JiraAttachment>, _>>()
            .map_err(|error| AtlassianError::unexpected_shape(error.to_string()))
    }

    pub async fn fetch_attachment_content(
        &self,
        content_path: &str,
        max_bytes: u64,
    ) -> Result<DownloadedContent, AtlassianError> {
        self.http
            .send_bytes_limited(
                self.http
                    .get_same_origin_or_relative_url(content_path, "attachment content URL")?,
                max_bytes,
            )
            .await
    }

    pub async fn get_safe_issue_attachments(
        &self,
        issue_key: String,
        options: AttachmentFetchOptions,
    ) -> Result<Value, AtlassianError> {
        if options.include_content && options.max_bytes == 0 {
            return Err(AtlassianError::invalid_input("max_bytes must be positive"));
        }

        let attachments = self.get_issue_attachments(issue_key.clone()).await?;
        let mut values = Vec::new();
        for attachment in attachments {
            if options.images_only && !attachment.is_image() {
                continue;
            }
            if let Some(attachment_ids) = options.attachment_ids.as_ref() {
                let Some(id) = attachment.id.as_deref() else {
                    continue;
                };
                if !attachment_ids.iter().any(|selected| selected == id) {
                    continue;
                }
            }

            let mut value = attachment.to_safe_metadata_value();
            if options.include_content {
                match self
                    .safe_attachment_content_value(&attachment, options.max_bytes)
                    .await
                {
                    Ok(content) => value["content"] = content,
                    Err(error) => value["content_error"] = error,
                }
            }
            values.push(value);
        }

        Ok(json!({
            "issue_key": issue_key,
            "count": values.len(),
            "images_only": options.images_only,
            "content_included": options.include_content,
            "attachments": values,
        }))
    }

    async fn safe_attachment_content_value(
        &self,
        attachment: &JiraAttachment,
        max_bytes: u64,
    ) -> Result<Value, Value> {
        let Some(content_path) = attachment.content.as_deref() else {
            return Err(json!({"message": "attachment content URL is missing"}));
        };
        let content = self
            .fetch_attachment_content(content_path, max_bytes)
            .await
            .map_err(|error| json!({"message": redact_url_query(&error.to_string())}))?;
        let content_type = content
            .content_type
            .or_else(|| attachment.mime_type.clone());

        Ok(json!({
            "encoding": "base64",
            "content_type": content_type,
            "size": content.bytes.len(),
            "data": base64_encode(&content.bytes),
        }))
    }

    pub async fn get_agile_boards(
        &self,
        project_key: Option<String>,
        board_type: Option<String>,
        start_at: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Value, AtlassianError> {
        let query = optional_query_params([
            ("projectKeyOrId", project_key),
            ("type", board_type),
            ("startAt", start_at.map(|value| value.to_string())),
            ("maxResults", limit.map(|value| value.to_string())),
        ]);
        self.http
            .send_json(self.http.get("/rest/agile/1.0/board")?.query(&query))
            .await
            .or_else(jira_software_agile_unavailable)
    }

    pub async fn get_board_issues(
        &self,
        board_id: u64,
        jql: Option<String>,
        fields: Option<Vec<String>>,
        start_at: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Value, AtlassianError> {
        let query = optional_query_params([
            ("jql", jql),
            ("fields", fields.map(|fields| fields.join(","))),
            ("startAt", start_at.map(|value| value.to_string())),
            ("maxResults", limit.map(|value| value.to_string())),
        ]);
        self.http
            .send_json(
                self.http
                    .get(&format!("/rest/agile/1.0/board/{board_id}/issue"))?
                    .query(&query),
            )
            .await
            .or_else(jira_software_agile_unavailable)
    }

    pub async fn get_sprints_from_board(
        &self,
        board_id: u64,
        state: Option<Vec<String>>,
        start_at: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Value, AtlassianError> {
        let query = optional_query_params([
            ("state", state.map(|state| state.join(","))),
            ("startAt", start_at.map(|value| value.to_string())),
            ("maxResults", limit.map(|value| value.to_string())),
        ]);
        self.http
            .send_json(
                self.http
                    .get(&format!("/rest/agile/1.0/board/{board_id}/sprint"))?
                    .query(&query),
            )
            .await
            .or_else(jira_software_agile_unavailable)
    }

    pub async fn get_sprint_issues(
        &self,
        sprint_id: u64,
        fields: Option<Vec<String>>,
        start_at: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Value, AtlassianError> {
        let query = optional_query_params([
            ("fields", fields.map(|fields| fields.join(","))),
            ("startAt", start_at.map(|value| value.to_string())),
            ("maxResults", limit.map(|value| value.to_string())),
        ]);
        self.http
            .send_json(
                self.http
                    .get(&format!("/rest/agile/1.0/sprint/{sprint_id}/issue"))?
                    .query(&query),
            )
            .await
            .or_else(jira_software_agile_unavailable)
    }

    pub async fn create_sprint(&self, payload: Value) -> Result<Value, AtlassianError> {
        self.http
            .send_json(self.http.post_json("/rest/agile/1.0/sprint", &payload)?)
            .await
    }

    pub async fn update_sprint(
        &self,
        sprint_id: u64,
        payload: Value,
    ) -> Result<Value, AtlassianError> {
        self.http
            .send_json(
                self.http
                    .put_json(&format!("/rest/agile/1.0/sprint/{sprint_id}"), &payload)?,
            )
            .await
    }

    pub async fn add_issues_to_sprint(
        &self,
        sprint_id: u64,
        issue_keys: Vec<String>,
    ) -> Result<Value, AtlassianError> {
        self.http
            .send_json_value_or_null(self.http.post_json(
                &format!("/rest/agile/1.0/sprint/{sprint_id}/issue"),
                &json!({ "issues": issue_keys }),
            )?)
            .await
    }

    pub async fn get_service_desk_for_project(
        &self,
        project_key: String,
    ) -> Result<Value, AtlassianError> {
        let project_key = safe_path_segment(&project_key, "project_key")?;
        let desks: Value = match self
            .http
            .send_json(self.http.get("/rest/servicedeskapi/servicedesk")?)
            .await
        {
            Ok(desks) => desks,
            Err(error) => return jira_service_management_unavailable(error),
        };
        let desk = desks
            .get("values")
            .and_then(Value::as_array)
            .and_then(|values| {
                values.iter().find(|desk| {
                    desk.get("projectKey")
                        .and_then(Value::as_str)
                        .is_some_and(|key| key == project_key)
                })
            })
            .cloned()
            .unwrap_or(Value::Null);
        Ok(json!({ "project_key": project_key, "service_desk": desk }))
    }

    pub async fn get_service_desk_queues(
        &self,
        service_desk_id: String,
        start_at: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Value, AtlassianError> {
        let service_desk_id = safe_path_segment(&service_desk_id, "service_desk_id")?;
        let query = optional_query_params([
            ("start", start_at.map(|value| value.to_string())),
            ("limit", limit.map(|value| value.to_string())),
        ]);
        self.http
            .send_json(
                self.http
                    .get(&format!(
                        "/rest/servicedeskapi/servicedesk/{service_desk_id}/queue"
                    ))?
                    .query(&query),
            )
            .await
            .or_else(jira_service_management_unavailable)
    }

    pub async fn get_queue_issues(
        &self,
        service_desk_id: String,
        queue_id: String,
        start_at: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Value, AtlassianError> {
        let service_desk_id = safe_path_segment(&service_desk_id, "service_desk_id")?;
        let queue_id = safe_path_segment(&queue_id, "queue_id")?;
        let query = optional_query_params([
            ("start", start_at.map(|value| value.to_string())),
            ("limit", limit.map(|value| value.to_string())),
        ]);
        self.http
            .send_json(
                self.http
                    .get(&format!(
                        "/rest/servicedeskapi/servicedesk/{service_desk_id}/queue/{queue_id}/issue"
                    ))?
                    .query(&query),
            )
            .await
            .or_else(jira_service_management_unavailable)
    }

    pub async fn get_issue_proforma_forms(
        &self,
        issue_key: String,
        cloud_id: Option<&str>,
    ) -> Result<Value, AtlassianError> {
        let Some(cloud_id) = forms_cloud_id_or_unavailable(cloud_id)? else {
            return Ok(forms_cloud_id_missing_result());
        };
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let path = forms_cloud_api_path(&cloud_id, &format!("/issue/{issue_key}/form"));
        self.atlassian_api_http
            .send_json(self.atlassian_api_http.get(&path)?)
            .await
            .or_else(jira_forms_unavailable)
    }

    pub async fn get_proforma_form_details(
        &self,
        issue_key: String,
        form_id: String,
        cloud_id: Option<&str>,
    ) -> Result<Value, AtlassianError> {
        let Some(cloud_id) = forms_cloud_id_or_unavailable(cloud_id)? else {
            return Ok(forms_cloud_id_missing_result());
        };
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let form_id = safe_path_segment(&form_id, "form_id")?;
        let path = forms_cloud_api_path(&cloud_id, &format!("/issue/{issue_key}/form/{form_id}"));
        self.atlassian_api_http
            .send_json(self.atlassian_api_http.get(&path)?)
            .await
            .or_else(jira_forms_unavailable)
    }

    pub async fn update_proforma_form_answers(
        &self,
        issue_key: String,
        form_id: String,
        answers: Vec<Value>,
        cloud_id: Option<&str>,
    ) -> Result<Value, AtlassianError> {
        let Some(cloud_id) = forms_cloud_id_or_unavailable(cloud_id)? else {
            return Ok(forms_cloud_id_missing_result());
        };
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let form_id = safe_path_segment(&form_id, "form_id")?;
        let payload = proforma_answers_payload(answers)?;
        let path = forms_cloud_api_path(&cloud_id, &format!("/issue/{issue_key}/form/{form_id}"));
        self.atlassian_api_http
            .send_json_value_or_null(self.atlassian_api_http.put_json(&path, &payload)?)
            .await
            .or_else(jira_forms_unavailable)
    }

    pub async fn get_issue_dates(
        &self,
        issue_key: String,
        include_status_changes: bool,
        include_status_summary: bool,
    ) -> Result<Value, AtlassianError> {
        let expand = include_status_changes.then(|| vec!["changelog".to_string()]);
        let issue = self
            .get_issue(GetIssueRequest {
                issue_key: issue_key.clone(),
                fields: Some(vec![
                    "created".to_string(),
                    "updated".to_string(),
                    "duedate".to_string(),
                    "resolutiondate".to_string(),
                    "status".to_string(),
                ]),
                expand,
                ..Default::default()
            })
            .await?;
        Ok(json!({
            "issue_key": issue_key,
            "include_status_changes": include_status_changes,
            "include_status_summary": include_status_summary,
            "issue": issue,
        }))
    }

    pub async fn get_issue_sla(
        &self,
        issue_key: String,
        metrics: Option<Vec<String>>,
        working_hours_only: Option<bool>,
        include_raw_dates: bool,
    ) -> Result<Value, AtlassianError> {
        let requested_fields = metrics
            .clone()
            .filter(|metrics| !metrics.is_empty())
            .unwrap_or_else(|| vec!["*all".to_string()]);
        let issue = self
            .get_issue(GetIssueRequest {
                issue_key: issue_key.clone(),
                fields: Some(requested_fields),
                ..Default::default()
            })
            .await?;
        let metric_values = extract_sla_metric_values(
            issue.get("fields").and_then(Value::as_object),
            metrics.as_deref(),
            include_raw_dates,
        );

        Ok(json!({
            "success": true,
            "issue_key": issue_key,
            "requested_metrics": metrics,
            "working_hours_only": working_hours_only,
            "include_raw_dates": include_raw_dates,
            "count": metric_values.len(),
            "metrics": metric_values,
            "product_dependency": {
                "product": "Jira Service Management SLA",
                "available": true,
                "message": "SLA fields were parsed from Jira issue fields; real Jira schema validation remains deferred to Stage 4."
            },
        }))
    }

    pub async fn get_issue_development_info(
        &self,
        issue_key: String,
        application_type: Option<String>,
        data_type: Option<String>,
    ) -> Result<Value, AtlassianError> {
        let issue_id = self.resolve_development_issue_id(&issue_key).await?;
        let query = optional_query_params([
            ("issueId", Some(issue_id)),
            ("applicationType", application_type),
            ("dataType", data_type),
        ]);
        self.http
            .send_json(
                self.http
                    .get("/rest/dev-status/1.0/issue/detail")?
                    .query(&query),
            )
            .await
            .or_else(jira_development_unavailable)
    }

    pub async fn get_issues_development_info(
        &self,
        issue_keys: Vec<String>,
        application_type: Option<String>,
        data_type: Option<String>,
    ) -> Result<Value, AtlassianError> {
        let mut results = Vec::new();
        for issue_key in issue_keys {
            match self
                .get_issue_development_info(
                    issue_key.clone(),
                    application_type.clone(),
                    data_type.clone(),
                )
                .await
            {
                Ok(value) => results.push(json!({"issue_key": issue_key, "development": value})),
                Err(error) => {
                    results.push(json!({"issue_key": issue_key, "error": error.to_string()}))
                }
            }
        }
        Ok(json!({ "issues": results }))
    }

    fn effective_projects(
        &self,
        request_projects: Option<&[String]>,
    ) -> Result<Vec<String>, AtlassianError> {
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
            Err(AtlassianError::invalid_input(
                "projects_filter does not overlap with configured Jira project filter",
            ))
        } else {
            Ok(intersection)
        }
    }

    fn issue_comment_path(&self, issue_key: &str) -> String {
        match self.config.deployment {
            JiraDeployment::Cloud => format!("/rest/api/3/issue/{issue_key}/comment"),
            JiraDeployment::ServerDataCenter => format!("/rest/api/2/issue/{issue_key}/comment"),
        }
    }

    async fn watcher_mutation(
        &self,
        issue_key: String,
        user_identifier: String,
        add: bool,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let path = format!("/rest/api/2/issue/{issue_key}/watchers");
        let builder = if add {
            self.http.post_json(&path, &user_identifier)?
        } else {
            let query_key = match self.config.deployment {
                JiraDeployment::Cloud => "accountId",
                JiraDeployment::ServerDataCenter => "username",
            };
            self.http
                .delete(&path)?
                .query(&[(query_key, user_identifier)])
        };
        let response = self.http.send_json_value_or_null(builder).await?;
        Ok(json!({
            "success": true,
            "issue_key": issue_key,
            "response": response,
        }))
    }

    async fn resolve_development_issue_id(
        &self,
        issue_key_or_id: &str,
    ) -> Result<String, AtlassianError> {
        let issue_key_or_id = issue_key_or_id.trim();
        let is_numeric_id = !issue_key_or_id.is_empty()
            && issue_key_or_id.chars().all(|value| value.is_ascii_digit());
        if is_numeric_id && self.config.projects_filter.is_empty() {
            return Ok(issue_key_or_id.to_string());
        }

        if !is_numeric_id {
            ensure_issue_allowed(issue_key_or_id, &self.config)?;
        }
        let issue_key_or_id = safe_path_segment(issue_key_or_id, "issue_key")?;
        let query = [("fields", "id,key")];
        let issue: JiraIssue = self
            .http
            .send_json(
                self.http
                    .get(&format!("/rest/api/2/issue/{issue_key_or_id}"))?
                    .query(&query),
            )
            .await?;
        let issue_key = issue.key.as_deref().ok_or_else(|| {
            AtlassianError::invalid_input(format!(
                "issue `{issue_key_or_id}` response did not include a key"
            ))
        })?;
        ensure_issue_allowed(issue_key, &self.config)?;
        issue.id.ok_or_else(|| {
            AtlassianError::invalid_input(format!(
                "issue `{issue_key_or_id}` response did not include a numeric id"
            ))
        })
    }
}

fn is_cloud_unbounded_jql_error(error: &AtlassianError) -> bool {
    matches!(
        error,
        AtlassianError::HttpStatus { status: 400, message }
            if message.contains("Unbounded JQL queries are not allowed here")
    )
}

fn is_removed_cloud_legacy_search_error(error: &AtlassianError) -> bool {
    matches!(
        error,
        AtlassianError::HttpStatus { status: 410, message }
            if message.contains("/rest/api/3/search/jql")
    )
}

fn cloud_unbounded_jql_error(jql: &str) -> AtlassianError {
    AtlassianError::invalid_input(format!(
        "Jira Cloud rejected an unbounded JQL query and the legacy search API is removed. Add a search restriction such as `project = \"KEY\"`, `issuekey in (...)`, or a configured JIRA_PROJECTS_FILTER before the order clause. Rejected JQL: {jql}"
    ))
}

fn cloud_offset_pagination_removed_error() -> AtlassianError {
    AtlassianError::invalid_input(
        "Jira Cloud offset pagination with start_at requires the removed /rest/api/3/search API. Use page_token from a previous jira_search response instead.",
    )
}

fn field_context_mapping_context_id(value: &Value) -> Result<Option<String>, AtlassianError> {
    let Some(context_id) = value.get("contextId") else {
        return Err(AtlassianError::unexpected_shape(
            "field context mapping response did not include contextId",
        ));
    };
    if context_id.is_null() {
        return Ok(None);
    }

    field_value_id(value, "contextId")
        .map(Some)
        .ok_or_else(|| AtlassianError::unexpected_shape("contextId must be a string or integer"))
}

fn cloud_issue_type_matches(value: &Value, requested: &str) -> bool {
    value
        .get("id")
        .and_then(Value::as_str)
        .is_some_and(|id| id == requested)
        || value
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|name| name.eq_ignore_ascii_case(requested))
}

fn field_value_id(value: &Value, field_name: &str) -> Option<String> {
    value.get(field_name).and_then(|id| {
        id.as_str()
            .map(ToString::to_string)
            .or_else(|| id.as_u64().map(|id| id.to_string()))
    })
}

fn optional_query_params<const N: usize>(
    pairs: [(&str, Option<String>); N],
) -> Vec<(String, String)> {
    pairs
        .into_iter()
        .filter_map(|(key, value)| value.map(|value| (key.to_string(), value)))
        .collect()
}

fn atlassian_api_base_url(config: &JiraConfig) -> String {
    let Ok(url) = Url::parse(&config.base_url) else {
        return ATLASSIAN_API_BASE_URL.to_string();
    };

    if url.host_str().is_some_and(|host| {
        matches!(
            host.to_ascii_lowercase().as_str(),
            "localhost" | "127.0.0.1" | "::1"
        )
    }) {
        config.base_url.clone()
    } else {
        ATLASSIAN_API_BASE_URL.to_string()
    }
}

fn jira_software_agile_unavailable(error: AtlassianError) -> Result<Value, AtlassianError> {
    match error {
        AtlassianError::HttpStatus { status, message } if matches!(status, 403 | 404) => {
            Ok(JiraOperationResult::product_unavailable(
                "Jira Software Agile REST",
                format!("Jira Software Agile REST is unavailable: {message}"),
            )
            .to_simplified_value())
        }
        error => Err(error),
    }
}

fn jira_service_management_unavailable(error: AtlassianError) -> Result<Value, AtlassianError> {
    match error {
        AtlassianError::HttpStatus { status, message } if matches!(status, 403 | 404) => {
            Ok(JiraOperationResult::product_unavailable(
                "Jira Service Management",
                format!("Jira Service Management REST is unavailable: {message}"),
            )
            .to_simplified_value())
        }
        error => Err(error),
    }
}

fn jira_forms_unavailable(error: AtlassianError) -> Result<Value, AtlassianError> {
    match error {
        AtlassianError::HttpStatus { status, message } if matches!(status, 403 | 404) => {
            Ok(JiraOperationResult::product_unavailable(
                "Jira Forms/ProForma",
                format!("Jira Forms API is unavailable: {message}"),
            )
            .to_simplified_value())
        }
        error => Err(error),
    }
}

fn jira_development_unavailable(error: AtlassianError) -> Result<Value, AtlassianError> {
    match error {
        AtlassianError::HttpStatus { status, message } if matches!(status, 403 | 404) => {
            Ok(JiraOperationResult::product_unavailable(
                "Jira development/dev-status",
                format!("Jira development/dev-status REST is unavailable: {message}"),
            )
            .to_simplified_value())
        }
        error => Err(error),
    }
}

fn forms_cloud_id_or_unavailable(cloud_id: Option<&str>) -> Result<Option<String>, AtlassianError> {
    let Some(cloud_id) = cloud_id.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    safe_path_segment(cloud_id, "cloud_id").map(Some)
}

fn forms_cloud_id_missing_result() -> Value {
    JiraOperationResult::product_unavailable(
        "Jira Forms/ProForma Cloud ID",
        "ATLASSIAN_OAUTH_CLOUD_ID is required before calling the Jira Forms API.",
    )
    .to_simplified_value()
}

fn forms_cloud_api_path(cloud_id: &str, endpoint: &str) -> String {
    format!("/jira/forms/cloud/{cloud_id}{endpoint}")
}

fn extract_sla_metric_values(
    fields: Option<&Map<String, Value>>,
    requested_metrics: Option<&[String]>,
    include_raw_dates: bool,
) -> Vec<Value> {
    let Some(fields) = fields else {
        return Vec::new();
    };
    let requested = requested_metrics
        .unwrap_or_default()
        .iter()
        .map(|metric| normalize_metric_key(metric))
        .filter(|metric| !metric.is_empty())
        .collect::<Vec<_>>();
    let include_all_sla_like = requested.is_empty();

    fields
        .iter()
        .filter(|(field_id, value)| {
            sla_metric_matches(field_id, value, &requested, include_all_sla_like)
        })
        .map(|(field_id, value)| {
            json!({
                "field_id": field_id,
                "name": value.get("name").and_then(Value::as_str),
                "value": simplify_sla_value(value, include_raw_dates),
            })
        })
        .collect()
}

fn sla_metric_matches(
    field_id: &str,
    value: &Value,
    requested_metrics: &[String],
    include_all_sla_like: bool,
) -> bool {
    let field_id = field_id.to_ascii_lowercase();
    let name = value
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let field_id_normalized = normalize_metric_key(&field_id);
    let name_normalized = normalize_metric_key(&name);

    if include_all_sla_like {
        field_id.contains("sla") || name.contains("sla")
    } else {
        requested_metrics.iter().any(|metric| {
            metric == &field_id_normalized
                || metric == &name_normalized
                || (!name_normalized.is_empty() && name_normalized.contains(metric))
                || (!name_normalized.is_empty() && metric.contains(&name_normalized))
        })
    }
}

fn normalize_metric_key(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn simplify_sla_value(value: &Value, include_raw_dates: bool) -> Value {
    if include_raw_dates {
        return value.clone();
    }

    let Some(object) = value.as_object() else {
        return value.clone();
    };
    let mut simplified = object.clone();
    for key in [
        "startTime",
        "stopTime",
        "breachTime",
        "pauseTime",
        "rawStartTime",
        "rawStopTime",
        "rawBreachTime",
    ] {
        simplified.remove(key);
    }
    Value::Object(simplified)
}

fn proforma_answers_payload(answers: Vec<Value>) -> Result<Value, AtlassianError> {
    let mut payload_answers = serde_json::Map::new();

    for answer in answers {
        let Value::Object(answer) = answer else {
            return Err(AtlassianError::invalid_input(
                "answers must be an array of JSON objects",
            ));
        };
        let Some(question_id) = answer
            .get("questionId")
            .or_else(|| answer.get("question_id"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Err(AtlassianError::invalid_input(
                "each answer must include a non-empty questionId",
            ));
        };

        let answer_type = answer
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("TEXT")
            .trim()
            .to_ascii_uppercase();
        let field_name = match answer_type.as_str() {
            "NUMBER" => "number",
            "DATE" | "DATETIME" => "date",
            "TIME" => "time",
            "SELECT" | "MULTI_SELECT" | "CHECKBOX" => "choices",
            "USER" | "MULTI_USER" => "users",
            _ => "text",
        };
        let mut value = answer.get("value").cloned().unwrap_or(Value::Null);
        if matches!(field_name, "choices" | "users") {
            value = match value {
                Value::Array(_) => value,
                Value::Null => Value::Array(Vec::new()),
                value => Value::Array(vec![value]),
            };
        }

        let mut typed_answer = serde_json::Map::new();
        typed_answer.insert(field_name.to_string(), value);
        payload_answers.insert(question_id.to_string(), Value::Object(typed_answer));
    }

    Ok(json!({ "answers": payload_answers }))
}

fn insert_optional(target: &mut Value, key: &'static str, value: Option<Value>) {
    if let Some(value) = value
        && let Some(object) = target.as_object_mut()
    {
        object.insert(key.to_string(), value);
    }
}

fn extract_createmeta_options(
    value: &Value,
    field_id: &str,
) -> Result<Vec<JiraFieldOption>, AtlassianError> {
    let projects = value
        .get("projects")
        .and_then(Value::as_array)
        .ok_or_else(|| AtlassianError::unexpected_shape("createmeta response missing projects"))?;

    for project in projects {
        let Some(issue_types) = project.get("issuetypes").and_then(Value::as_array) else {
            continue;
        };
        for issue_type in issue_types {
            let Some(fields) = issue_type.get("fields").and_then(Value::as_object) else {
                continue;
            };
            let Some(field) = fields.get(field_id) else {
                continue;
            };
            let options = field
                .get("allowedValues")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    AtlassianError::unexpected_shape("createmeta field missing allowedValues")
                })?;
            return options
                .iter()
                .cloned()
                .map(serde_json::from_value)
                .collect::<Result<Vec<JiraFieldOption>, _>>()
                .map_err(|error| AtlassianError::unexpected_shape(error.to_string()));
        }
    }

    Err(AtlassianError::unexpected_shape(format!(
        "createmeta response missing field `{field_id}`"
    )))
}

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, sync::Arc};

    use axum::{
        Json, Router,
        body::Bytes,
        extract::State,
        http::{HeaderMap, Method, StatusCode},
        response::{IntoResponse, Response},
        routing::any,
    };
    use serde_json::json;
    use tokio::sync::Mutex;

    use crate::{
        atlassian::auth::AtlassianAuth,
        jira::config::{DEFAULT_JIRA_TIMEOUT_SECONDS, JiraDeployment},
    };

    use super::*;

    #[derive(Clone, Debug)]
    struct RecordedRequest {
        method: Method,
        path: String,
        authorization: Option<String>,
        body: Value,
    }

    #[derive(Clone)]
    struct MockState {
        response: Value,
        status: StatusCode,
        requests: Arc<Mutex<Vec<RecordedRequest>>>,
    }

    async fn mock_handler(
        State(state): State<MockState>,
        method: Method,
        headers: HeaderMap,
        uri: axum::http::Uri,
        body: Bytes,
    ) -> Response {
        let parsed_body = if body.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&body).unwrap()
        };
        state.requests.lock().await.push(RecordedRequest {
            method,
            path: uri
                .path_and_query()
                .map(ToString::to_string)
                .unwrap_or_else(|| uri.path().to_string()),
            authorization: headers
                .get("authorization")
                .and_then(|value| value.to_str().ok())
                .map(ToString::to_string),
            body: parsed_body,
        });

        (state.status, Json(state.response)).into_response()
    }

    async fn invalid_json_handler(
        State(state): State<MockState>,
        method: Method,
        headers: HeaderMap,
        uri: axum::http::Uri,
        body: Bytes,
    ) -> Response {
        let parsed_body = if body.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&body).unwrap()
        };
        state.requests.lock().await.push(RecordedRequest {
            method,
            path: uri
                .path_and_query()
                .map(ToString::to_string)
                .unwrap_or_else(|| uri.path().to_string()),
            authorization: headers
                .get("authorization")
                .and_then(|value| value.to_str().ok())
                .map(ToString::to_string),
            body: parsed_body,
        });

        (StatusCode::OK, "not-json").into_response()
    }

    async fn cloud_search_fallback_handler(
        State(requests): State<Arc<Mutex<Vec<RecordedRequest>>>>,
        method: Method,
        headers: HeaderMap,
        uri: axum::http::Uri,
        body: Bytes,
    ) -> Response {
        let parsed_body = if body.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&body).unwrap()
        };
        requests.lock().await.push(RecordedRequest {
            method,
            path: uri
                .path_and_query()
                .map(ToString::to_string)
                .unwrap_or_else(|| uri.path().to_string()),
            authorization: headers
                .get("authorization")
                .and_then(|value| value.to_str().ok())
                .map(ToString::to_string),
            body: parsed_body,
        });

        match uri.path() {
            "/rest/api/3/search/jql" => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "errorMessages": [
                        "Unbounded JQL queries are not allowed here. Please add a search restriction to your query."
                    ]
                })),
            )
                .into_response(),
            "/rest/api/3/search" => Json(json!({
                "issues": [{
                    "id": "10001",
                    "key": "ABC-1",
                    "fields": {"summary": "Demo"}
                }],
                "total": 1,
                "startAt": 0,
                "maxResults": 50
            }))
            .into_response(),
            _ => StatusCode::NOT_FOUND.into_response(),
        }
    }

    async fn removed_cloud_legacy_search_handler(
        State(requests): State<Arc<Mutex<Vec<RecordedRequest>>>>,
        method: Method,
        headers: HeaderMap,
        uri: axum::http::Uri,
        body: Bytes,
    ) -> Response {
        let parsed_body = if body.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&body).unwrap()
        };
        requests.lock().await.push(RecordedRequest {
            method,
            path: uri
                .path_and_query()
                .map(ToString::to_string)
                .unwrap_or_else(|| uri.path().to_string()),
            authorization: headers
                .get("authorization")
                .and_then(|value| value.to_str().ok())
                .map(ToString::to_string),
            body: parsed_body,
        });

        match uri.path() {
            "/rest/api/3/search/jql" => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "errorMessages": [
                        "Unbounded JQL queries are not allowed here. Please add a search restriction to your query."
                    ]
                })),
            )
                .into_response(),
            "/rest/api/3/search" => (
                StatusCode::GONE,
                Json(json!({
                    "errorMessages": [
                        "The requested API has been removed. Please migrate to the /rest/api/3/search/jql API."
                    ]
                })),
            )
                .into_response(),
            _ => StatusCode::NOT_FOUND.into_response(),
        }
    }

    async fn cloud_field_options_context_handler(
        State(requests): State<Arc<Mutex<Vec<RecordedRequest>>>>,
        method: Method,
        headers: HeaderMap,
        uri: axum::http::Uri,
        body: Bytes,
    ) -> Response {
        let parsed_body = if body.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&body).unwrap()
        };
        let request_body = parsed_body.clone();
        requests.lock().await.push(RecordedRequest {
            method,
            path: uri
                .path_and_query()
                .map(ToString::to_string)
                .unwrap_or_else(|| uri.path().to_string()),
            authorization: headers
                .get("authorization")
                .and_then(|value| value.to_str().ok())
                .map(ToString::to_string),
            body: parsed_body,
        });

        match uri.path() {
            "/rest/api/3/project/ABC" => Json(json!({
                "id": "10000",
                "key": "ABC",
                "issueTypes": [
                    {"id": "1", "name": "Bug"},
                    {"id": "3", "name": "Task"}
                ]
            }))
            .into_response(),
            "/rest/api/3/field/customfield_10001/context/mapping" => {
                let issue_type_id = request_body
                    .pointer("/mappings/0/issueTypeId")
                    .and_then(Value::as_str);
                let context_id = if issue_type_id == Some("3") {
                    Value::Null
                } else {
                    json!("20001")
                };

                Json(json!({
                    "values": [{
                        "projectId": "10000",
                        "issueTypeId": issue_type_id.unwrap_or(""),
                        "contextId": context_id
                    }],
                    "isLast": true
                }))
                .into_response()
            }
            "/rest/api/3/field/customfield_10001/context/20001/option" => Json(json!({
                "values": [{"id": "1", "value": "High"}],
                "isLast": true
            }))
            .into_response(),
            _ => StatusCode::NOT_FOUND.into_response(),
        }
    }

    async fn stage_three_handler(
        State(requests): State<Arc<Mutex<Vec<RecordedRequest>>>>,
        method: Method,
        headers: HeaderMap,
        uri: axum::http::Uri,
        body: Bytes,
    ) -> Response {
        let parsed_body = if body.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&body).unwrap()
        };
        let path = uri
            .path_and_query()
            .map(ToString::to_string)
            .unwrap_or_else(|| uri.path().to_string());
        requests.lock().await.push(RecordedRequest {
            method: method.clone(),
            path: path.clone(),
            authorization: headers
                .get("authorization")
                .and_then(|value| value.to_str().ok())
                .map(ToString::to_string),
            body: parsed_body,
        });

        if method == Method::DELETE {
            return StatusCode::NO_CONTENT.into_response();
        }

        let path_only = uri.path();
        if path_only == "/secure/attachment/1/file.png" {
            return (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "image/png")],
                "image-bytes",
            )
                .into_response();
        }
        if path_only == "/secure/attachment/2/notes.txt" {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "errorMessages": [
                        "failed /secure/attachment/2/notes.txt?token=secret&client=abc"
                    ]
                })),
            )
                .into_response();
        }
        if method == Method::GET
            && path.starts_with("/rest/agile/1.0/board?")
            && path.contains("projectKeyOrId=NOAGILE")
        {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"errorMessages": ["Jira Software is not available"]})),
            )
                .into_response();
        }
        if method == Method::GET && path_only.starts_with("/jsm-down/rest/servicedeskapi") {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"errorMessages": ["Jira Service Management is not available"]})),
            )
                .into_response();
        }
        if method == Method::GET && path_only.starts_with("/dev-down/rest/dev-status") {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"errorMessages": ["Jira development status is not available"]})),
            )
                .into_response();
        }

        match path_only {
            "/rest/api/2/project" => Json(json!([
                {"id": "10000", "key": "ABC", "name": "Alpha"},
                {"id": "10001", "key": "XYZ", "name": "Other"}
            ]))
            .into_response(),
            path if path.ends_with("/versions") => {
                Json(json!([{"id": "1", "name": "v1"}])).into_response()
            }
            path if path.ends_with("/components") => {
                Json(json!([{"id": "2", "name": "API"}])).into_response()
            }
            "/rest/api/2/version" => Json(json!({"id": "1", "name": "v1"})).into_response(),
            "/rest/api/2/user" => {
                Json(json!({"accountId": "abc", "displayName": "Ada"})).into_response()
            }
            path if path.ends_with("/watchers") && method == Method::GET => {
                Json(json!({"watcherCount": 1, "watchers": [{"displayName": "Ada"}]}))
                    .into_response()
            }
            path if path.ends_with("/worklog") && method == Method::GET => {
                Json(json!({"worklogs": [{"id": "10", "timeSpent": "1h"}]})).into_response()
            }
            path if path.ends_with("/worklog") => {
                Json(json!({"id": "10", "timeSpent": "1h"})).into_response()
            }
            "/rest/api/2/issueLinkType" => {
                Json(json!({"issueLinkTypes": [{"id": "100", "name": "Blocks"}]})).into_response()
            }
            "/rest/api/2/issueLink" => Json(json!({"id": "200"})).into_response(),
            path if path.ends_with("/remotelink") => Json(json!({"id": "300"})).into_response(),
            "/rest/agile/1.0/board" => {
                Json(json!({"values": [{"id": 1, "name": "Board"}]})).into_response()
            }
            path if path.ends_with("/board/1/issue") => {
                Json(json!({"issues": [{"key": "ABC-1", "fields": {}}]})).into_response()
            }
            path if path.ends_with("/board/1/sprint") => {
                Json(json!({"values": [{"id": 2, "name": "Sprint"}]})).into_response()
            }
            path if path.ends_with("/sprint/2/issue") => {
                Json(json!({"issues": [{"key": "ABC-1", "fields": {}}]})).into_response()
            }
            "/rest/agile/1.0/sprint" => Json(json!({"id": 2, "name": "Sprint"})).into_response(),
            path if path.ends_with("/sprint/2") => {
                Json(json!({"id": 2, "name": "Sprint updated"})).into_response()
            }
            path if path.ends_with("/sprint/2/issue") => Json(Value::Null).into_response(),
            "/rest/servicedeskapi/servicedesk" => Json(
                json!({"values": [{"id": "4", "projectKey": "ABC", "serviceDeskName": "Support"}]}),
            )
            .into_response(),
            path if path.ends_with("/servicedesk/4/queue") => {
                Json(json!({"values": [{"id": "47", "name": "Open"}]})).into_response()
            }
            path if path.ends_with("/servicedesk/4/queue/47/issue") => {
                Json(json!({"values": [{"key": "ABC-1"}]})).into_response()
            }
            "/jira/forms/cloud/cloud-123/issue/ABC-1/form" if method == Method::GET => {
                Json(json!({"forms": [{
                    "id": "form-1",
                    "name": "Request form",
                    "state": {"status": "o"},
                    "submitted": false
                }]}))
                .into_response()
            }
            "/jira/forms/cloud/cloud-123/issue/ABC-1/form/form-1" if method == Method::GET => {
                Json(json!({
                    "id": "form-1",
                    "name": "Request form",
                    "state": {"status": "o"},
                    "design": {"content": []},
                    "answers": {"q1": {"text": "Existing"}}
                }))
                .into_response()
            }
            "/jira/forms/cloud/cloud-123/issue/ABC-1/form/form-1" if method == Method::PUT => {
                Json(json!({"id": "form-1", "updated": true})).into_response()
            }
            path if path.starts_with("/jira/forms/cloud/forms-down/") => (
                StatusCode::NOT_FOUND,
                Json(json!({"errorMessages": ["Jira Forms is not available"]})),
            )
                .into_response(),
            "/rest/dev-status/1.0/issue/detail" => {
                Json(json!({"detail": [{"branches": [], "pullRequests": []}]})).into_response()
            }
            path if path.starts_with("/rest/api/2/issue/ABC-1") && method == Method::GET => {
                Json(json!({
                    "id": "10001",
                    "key": "ABC-1",
                    "fields": {
                        "summary": "Demo",
                        "customfield_sla": {
                            "name": "Time to resolution SLA",
                            "ongoingCycle": {
                                "breached": false,
                                "elapsedTime": {"millis": 60000},
                                "remainingTime": {"millis": 120000},
                                "startTime": "2026-01-01T00:00:00.000+0000"
                            }
                        },
                        "attachment": [{
                            "id": "1",
                            "filename": "file.png",
                            "mimeType": "image/png",
                            "size": 11,
                            "content": "/secure/attachment/1/file.png?token=secret"
                        }, {
                            "id": "2",
                            "filename": "notes.txt",
                            "mimeType": "text/plain",
                            "size": 42,
                            "content": "/secure/attachment/2/notes.txt?token=secret&client=abc"
                        }]
                    }
                }))
                .into_response()
            }
            path if path.starts_with("/rest/api/2/issue/ABC-1") => Json(json!({
                "id": "10001",
                "key": "ABC-1",
                "fields": {"summary": "Demo"}
            }))
            .into_response(),
            "/rest/api/2/issue" => Json(json!({
                "id": "10001",
                "key": "ABC-1",
                "fields": {"summary": "Demo"}
            }))
            .into_response(),
            "/rest/api/2/issue/bulk" => Json(json!({"issues": [{"key": "ABC-1"}]})).into_response(),
            "/rest/api/3/changelog/bulkfetch" => {
                Json(json!({"issueChangeLogs": [{"issueId": "10001", "changeHistories": []}]}))
                    .into_response()
            }
            _ => Json(json!({"ok": true, "path": path})).into_response(),
        }
    }

    async fn mock_server(response: Value) -> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
        mock_server_with_status(response, StatusCode::OK).await
    }

    async fn mock_server_with_status(
        response: Value,
        status: StatusCode,
    ) -> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let state = MockState {
            response,
            status,
            requests: requests.clone(),
        };
        let app = Router::new().fallback(any(mock_handler)).with_state(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address: SocketAddr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://{address}"), requests)
    }

    async fn invalid_json_mock_server() -> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let state = MockState {
            response: Value::Null,
            status: StatusCode::OK,
            requests: requests.clone(),
        };
        let app = Router::new()
            .fallback(any(invalid_json_handler))
            .with_state(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address: SocketAddr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://{address}"), requests)
    }

    async fn cloud_search_fallback_mock_server() -> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let app = Router::new()
            .fallback(any(cloud_search_fallback_handler))
            .with_state(requests.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address: SocketAddr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://{address}"), requests)
    }

    async fn removed_cloud_legacy_search_mock_server() -> (String, Arc<Mutex<Vec<RecordedRequest>>>)
    {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let app = Router::new()
            .fallback(any(removed_cloud_legacy_search_handler))
            .with_state(requests.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address: SocketAddr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://{address}"), requests)
    }

    async fn cloud_field_options_context_mock_server() -> (String, Arc<Mutex<Vec<RecordedRequest>>>)
    {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let app = Router::new()
            .fallback(any(cloud_field_options_context_handler))
            .with_state(requests.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address: SocketAddr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://{address}"), requests)
    }

    async fn stage_three_mock_server() -> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let app = Router::new()
            .fallback(any(stage_three_handler))
            .with_state(requests.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address: SocketAddr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://{address}"), requests)
    }

    fn config(base_url: String, deployment: JiraDeployment) -> JiraConfig {
        JiraConfig {
            base_url,
            deployment,
            auth: match deployment {
                JiraDeployment::Cloud => AtlassianAuth::Basic {
                    username: "user@example.com".to_string(),
                    api_token: "test-api-token".to_string(),
                },
                JiraDeployment::ServerDataCenter => AtlassianAuth::Pat {
                    personal_token: "test-pat-value".to_string(),
                },
            },
            ssl_verify: true,
            projects_filter: BTreeSet::new(),
            timeout_seconds: DEFAULT_JIRA_TIMEOUT_SECONDS,
        }
    }

    #[tokio::test]
    async fn get_issue_uses_v2_endpoint_and_auth_header() {
        let (base_url, requests) =
            mock_server(json!({"id": "10001", "key": "ABC-1", "fields": {"summary": "Demo"}}))
                .await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .get_issue(GetIssueRequest {
                issue_key: "ABC-1".to_string(),
                fields: Some(vec!["summary".to_string()]),
                ..Default::default()
            })
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value["key"], "ABC-1");
        assert_eq!(requests[0].method, Method::GET);
        assert!(requests[0].path.starts_with("/rest/api/2/issue/ABC-1"));
        let expected_header = format!("Bearer {}", "test-pat-value");
        assert_eq!(
            requests[0].authorization.as_deref(),
            Some(expected_header.as_str())
        );
    }

    #[tokio::test]
    async fn cloud_search_uses_v3_search_jql_and_basic_auth() {
        let (base_url, requests) =
            mock_server(json!({"issues": [], "nextPageToken": "next", "isLast": false})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
        let value = client
            .search(SearchRequest {
                jql: "status = Done".to_string(),
                limit: Some(10),
                projects_filter: Some(vec!["ABC".to_string()]),
                page_token: Some("token".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value["next_page_token"], "next");
        assert_eq!(requests[0].method, Method::POST);
        assert_eq!(requests[0].path, "/rest/api/3/search/jql");
        assert!(requests[0].authorization.as_deref().is_some_and(|header| {
            header.starts_with("Basic ") && !header.contains("test-api-token")
        }));
        assert_eq!(requests[0].body["maxResults"], 10);
        assert_eq!(requests[0].body["nextPageToken"], "token");
        assert!(
            requests[0].body["jql"]
                .as_str()
                .unwrap()
                .contains("project = \"ABC\"")
        );
    }

    #[tokio::test]
    async fn cloud_search_allows_issue_without_key() {
        let (base_url, requests) = mock_server(json!({
            "issues": [{
                "id": "10001",
                "fields": {"summary": "Demo"}
            }],
            "isLast": true
        }))
        .await;
        let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
        let value = client
            .search(SearchRequest {
                jql: "project = SCRUM".to_string(),
                limit: Some(20),
                ..Default::default()
            })
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(requests[0].path, "/rest/api/3/search/jql");
        assert_eq!(value["issues"][0]["id"], "10001");
        assert!(value["issues"][0]["key"].is_null());
        assert_eq!(value["issues"][0]["summary"], "Demo");
    }

    #[tokio::test]
    async fn cloud_search_retries_legacy_search_when_enhanced_rejects_unbounded_jql() {
        let (base_url, requests) = cloud_search_fallback_mock_server().await;
        let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
        let value = client
            .search(SearchRequest {
                jql: "created >= -30d ORDER BY updated DESC".to_string(),
                limit: Some(50),
                ..Default::default()
            })
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value["total"], 1);
        assert_eq!(value["issues"][0]["key"], "ABC-1");
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].method, Method::POST);
        assert_eq!(requests[0].path, "/rest/api/3/search/jql");
        assert_eq!(requests[1].method, Method::POST);
        assert_eq!(requests[1].path, "/rest/api/3/search");
        assert_eq!(
            requests[1].body["jql"],
            "created >= -30d ORDER BY updated DESC"
        );
        assert_eq!(requests[1].body["startAt"], 0);
        assert_eq!(requests[1].body["maxResults"], 50);
    }

    #[tokio::test]
    async fn cloud_search_reports_unbounded_jql_when_legacy_search_is_removed() {
        let (base_url, requests) = removed_cloud_legacy_search_mock_server().await;
        let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
        let error = client
            .search(SearchRequest {
                jql: "ORDER BY created DESC".to_string(),
                limit: Some(20),
                ..Default::default()
            })
            .await
            .unwrap_err()
            .to_string();
        let requests = requests.lock().await;

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].path, "/rest/api/3/search/jql");
        assert_eq!(requests[1].path, "/rest/api/3/search");
        assert!(error.contains("unbounded JQL"));
        assert!(error.contains("project = \"KEY\""));
        assert!(error.contains("ORDER BY created DESC"));
    }

    #[tokio::test]
    async fn server_search_uses_v2_search_and_start_at() {
        let (base_url, requests) = mock_server(json!({"issues": [], "total": 0})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        client
            .search(SearchRequest {
                jql: "project = ABC".to_string(),
                start_at: Some(20),
                ..Default::default()
            })
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(requests[0].path, "/rest/api/2/search");
        assert_eq!(requests[0].body["startAt"], 20);
    }

    #[tokio::test]
    async fn get_project_issues_builds_project_jql() {
        let (base_url, requests) = mock_server(json!({"issues": [], "total": 0})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        client
            .get_project_issues("ABC".to_string(), Some(5), Some(10))
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(requests[0].path, "/rest/api/2/search");
        assert_eq!(requests[0].body["jql"], "project = \"ABC\"");
        assert_eq!(requests[0].body["maxResults"], 5);
        assert_eq!(requests[0].body["startAt"], 10);
    }

    #[tokio::test]
    async fn search_fields_filters_case_insensitively_and_handles_missing_schema() {
        let (base_url, requests) = mock_server(json!([
            {"id": "summary", "name": "Summary"},
            {"id": "customfield_10001", "name": "Customer Impact", "schema": {"type": "string"}}
        ]))
        .await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .search_fields(Some("CUSTOMER".to_string()), Some(1))
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(requests[0].path, "/rest/api/2/field");
        assert_eq!(value.as_array().unwrap().len(), 1);
        assert_eq!(value[0]["id"], "customfield_10001");
    }

    #[tokio::test]
    async fn cloud_search_fields_uses_paginated_v3_endpoint() {
        let (base_url, requests) = mock_server(json!({
            "values": [
                {"id": "project", "key": "project", "name": "Project", "schema": {"type": "project"}},
                {"id": "summary", "name": "Summary"}
            ]
        }))
        .await;
        let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
        let value = client
            .search_fields(Some("project".to_string()), Some(2))
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert!(requests[0].path.starts_with("/rest/api/3/field/search?"));
        assert!(requests[0].path.contains("maxResults=2"));
        assert!(requests[0].path.contains("query=project"));
        assert_eq!(value.as_array().unwrap().len(), 1);
        assert_eq!(value[0]["id"], "project");
    }

    #[tokio::test]
    async fn field_options_support_cloud_context_options() {
        let (base_url, requests) =
            mock_server(json!({"values": [{"id": "1", "value": "High"}]})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
        let value = client
            .get_field_options(FieldOptionsRequest {
                field_id: "customfield_10001".to_string(),
                context_id: Some("20001".to_string()),
                values_only: true,
                ..Default::default()
            })
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value, json!(["High"]));
        assert!(
            requests[0]
                .path
                .starts_with("/rest/api/3/field/customfield_10001/context/20001/option")
        );
    }

    #[tokio::test]
    async fn field_options_resolves_cloud_context_with_project_and_issue_type() {
        let (base_url, requests) = cloud_field_options_context_mock_server().await;
        let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
        let value = client
            .get_field_options(FieldOptionsRequest {
                field_id: "customfield_10001".to_string(),
                project_key: Some("ABC".to_string()),
                issue_type: Some("Bug".to_string()),
                values_only: true,
                ..Default::default()
            })
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value, json!(["High"]));
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].method, Method::GET);
        assert!(requests[0].path.starts_with("/rest/api/3/project/ABC"));
        assert_eq!(requests[1].method, Method::POST);
        assert!(
            requests[1]
                .path
                .starts_with("/rest/api/3/field/customfield_10001/context/mapping?")
        );
        assert_eq!(
            requests[1].body["mappings"][0],
            json!({"projectId": "10000", "issueTypeId": "1"})
        );
        assert_eq!(requests[2].method, Method::GET);
        assert!(
            requests[2]
                .path
                .starts_with("/rest/api/3/field/customfield_10001/context/20001/option")
        );
    }

    #[tokio::test]
    async fn field_options_requires_cloud_context_or_project_issue_type() {
        let (base_url, requests) = mock_server(json!({})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
        let error = client
            .get_field_options(FieldOptionsRequest {
                field_id: "customfield_10001".to_string(),
                values_only: true,
                ..Default::default()
            })
            .await
            .unwrap_err();
        let requests = requests.lock().await;

        assert!(requests.is_empty());
        let error = error.to_string();
        assert!(error.contains("context_id is required for Jira Cloud field options"));
    }

    #[tokio::test]
    async fn field_options_reports_cloud_context_mapping_miss() {
        let (base_url, requests) = cloud_field_options_context_mock_server().await;
        let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
        let error = client
            .get_field_options(FieldOptionsRequest {
                field_id: "customfield_10001".to_string(),
                project_key: Some("ABC".to_string()),
                issue_type: Some("Task".to_string()),
                values_only: true,
                ..Default::default()
            })
            .await
            .unwrap_err();
        let requests = requests.lock().await;

        assert_eq!(requests.len(), 2);
        let error = error.to_string();
        assert!(error.contains("No Jira Cloud field context applies"));
    }

    #[tokio::test]
    async fn field_options_support_server_createmeta_options() {
        let (base_url, requests) = mock_server(json!({
            "projects": [{
                "issuetypes": [{
                    "fields": {
                        "customfield_10001": {
                            "allowedValues": [{"id": "1", "value": "High"}]
                        }
                    }
                }]
            }]
        }))
        .await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .get_field_options(FieldOptionsRequest {
                field_id: "customfield_10001".to_string(),
                project_key: Some("ABC".to_string()),
                issue_type: Some("Bug".to_string()),
                values_only: false,
                ..Default::default()
            })
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert!(requests[0].path.starts_with("/rest/api/2/issue/createmeta"));
        assert_eq!(value["values"][0]["value"], "High");
    }

    #[tokio::test]
    async fn add_comment_uses_server_string_body() {
        let (base_url, requests) = mock_server(json!({
            "id": "10",
            "body": "Hello",
            "author": {"displayName": "Ada"}
        }))
        .await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .add_comment("ABC-1".to_string(), "Hello".to_string(), None)
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value["body"], "Hello");
        assert_eq!(requests[0].path, "/rest/api/2/issue/ABC-1/comment");
        assert_eq!(requests[0].body["body"], "Hello");
    }

    #[tokio::test]
    async fn edit_comment_uses_put_endpoint_and_visibility() {
        let (base_url, requests) = mock_server(json!({
            "id": "10",
            "body": "Updated",
            "visibility": {"type": "role", "value": "Developers"}
        }))
        .await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .edit_comment(
                "ABC-1".to_string(),
                "10".to_string(),
                "Updated".to_string(),
                Some(json!({"type": "role", "value": "Developers"})),
            )
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value["visibility"]["value"], "Developers");
        assert_eq!(requests[0].method, Method::PUT);
        assert_eq!(requests[0].path, "/rest/api/2/issue/ABC-1/comment/10");
        assert_eq!(requests[0].body["visibility"]["value"], "Developers");
    }

    #[tokio::test]
    async fn get_issue_missing_fields_payload_is_simplified() {
        let (base_url, _requests) = mock_server(json!({"key": "ABC-1"})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .get_issue(GetIssueRequest {
                issue_key: "ABC-1".to_string(),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(value["key"], "ABC-1");
        assert!(value["fields"].is_null());
        assert!(value["summary"].is_null());
    }

    #[tokio::test]
    async fn comment_missing_optional_payload_fields_is_simplified() {
        let (base_url, _requests) = mock_server(json!({"id": "10"})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .add_comment("ABC-1".to_string(), "Hello".to_string(), None)
            .await
            .unwrap();

        assert_eq!(value["id"], "10");
        assert_eq!(value["body"], "");
        assert!(value["author"]["display_name"].is_null());
    }

    #[tokio::test]
    async fn transitions_missing_fields_payload_is_simplified() {
        let (base_url, _requests) =
            mock_server(json!({"transitions": [{"id": "31", "name": "Done"}]})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client.get_transitions("ABC-1".to_string()).await.unwrap();

        assert_eq!(value["transitions"][0]["id"], "31");
        assert!(value["transitions"][0]["fields"].is_null());
        assert!(value["transitions"][0]["to"]["name"].is_null());
    }

    #[tokio::test]
    async fn transition_issue_posts_transition_payload() {
        let (base_url, requests) = mock_server(json!({})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .transition_issue(
                "ABC-1".to_string(),
                "31".to_string(),
                Some(json!({"resolution": {"name": "Done"}})),
                Some("Resolved".to_string()),
            )
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value["transition_id"], "31");
        assert_eq!(requests[0].path, "/rest/api/2/issue/ABC-1/transitions");
        assert_eq!(requests[0].body["transition"]["id"], "31");
        assert_eq!(requests[0].body["fields"]["resolution"]["name"], "Done");
    }

    #[tokio::test]
    async fn transition_issue_accepts_no_content_response() {
        let (base_url, _requests) =
            mock_server_with_status(json!({}), StatusCode::NO_CONTENT).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .transition_issue("ABC-1".to_string(), "31".to_string(), None, None)
            .await
            .unwrap();

        assert_eq!(value["transition_id"], "31");
        assert!(value["response"].is_null());
    }

    #[tokio::test]
    async fn issue_not_found_error_is_safe() {
        let (base_url, _requests) =
            mock_server_with_status(json!({"errorMessages": ["missing"]}), StatusCode::NOT_FOUND)
                .await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let error = client
            .get_issue(GetIssueRequest {
                issue_key: "ABC-1".to_string(),
                ..Default::default()
            })
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains("HTTP 404"));
        assert!(!error.contains("Bearer"));
        assert!(!error.contains("test-pat-value"));
    }

    #[tokio::test]
    async fn unauthorized_error_is_safe() {
        assert_status_error_is_safe(StatusCode::UNAUTHORIZED, "HTTP 401").await;
    }

    #[tokio::test]
    async fn forbidden_error_is_safe() {
        assert_status_error_is_safe(StatusCode::FORBIDDEN, "HTTP 403").await;
    }

    #[tokio::test]
    async fn rate_limit_error_is_safe() {
        assert_status_error_is_safe(StatusCode::TOO_MANY_REQUESTS, "HTTP 429").await;
    }

    async fn assert_status_error_is_safe(status: StatusCode, expected: &str) {
        let (base_url, _requests) =
            mock_server_with_status(json!({"errorMessages": ["safe failure"]}), status).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let error = client
            .get_transitions("ABC-1".to_string())
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains(expected));
        assert!(error.contains("safe failure"));
        assert!(!error.contains("Bearer"));
        assert!(!error.contains("test-pat-value"));
    }

    #[tokio::test]
    async fn invalid_json_response_includes_safe_request_context() {
        let (base_url, requests) = invalid_json_mock_server().await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let error = client
            .get_issue(GetIssueRequest {
                issue_key: "ABC-1".to_string(),
                ..Default::default()
            })
            .await
            .unwrap_err()
            .to_string();
        let requests = requests.lock().await;

        assert!(error.contains("JSON decode error"));
        assert!(error.contains("GET /rest/api/2/issue/ABC-1"));
        assert!(!error.contains("Bearer"));
        assert!(!error.contains("test-pat-value"));
        assert_eq!(requests.len(), 1);
    }

    #[tokio::test]
    async fn project_filter_rejects_issue_without_http_request() {
        let (base_url, requests) =
            mock_server(json!({"id": "10001", "key": "XYZ-1", "fields": {}})).await;
        let mut config = config(base_url, JiraDeployment::ServerDataCenter);
        config.projects_filter = BTreeSet::from(["ABC".to_string()]);
        let client = JiraClient::new(config).unwrap();
        let error = client
            .get_issue(GetIssueRequest {
                issue_key: "XYZ-1".to_string(),
                ..Default::default()
            })
            .await
            .unwrap_err()
            .to_string();
        let requests = requests.lock().await;

        assert!(error.contains("outside the configured Jira project filter"));
        assert!(requests.is_empty());
    }

    #[tokio::test]
    async fn project_filter_rejects_project_metadata_without_http_request() {
        let (base_url, requests) = mock_server(json!([])).await;
        let mut config = config(base_url, JiraDeployment::ServerDataCenter);
        config.projects_filter = BTreeSet::from(["ABC".to_string()]);
        let client = JiraClient::new(config).unwrap();
        let versions_error = client
            .get_project_versions("XYZ".to_string())
            .await
            .unwrap_err()
            .to_string();
        let components_error = client
            .get_project_components("XYZ".to_string())
            .await
            .unwrap_err()
            .to_string();
        let requests = requests.lock().await;

        assert!(versions_error.contains("outside the configured Jira project filter"));
        assert!(components_error.contains("outside the configured Jira project filter"));
        assert!(requests.is_empty());
    }

    #[tokio::test]
    async fn stage_three_issue_helpers_use_expected_endpoints() {
        let (base_url, requests) = stage_three_mock_server().await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

        let created = client
            .create_issue(json!({
                "project": {"key": "ABC"},
                "summary": "Demo",
                "issuetype": {"name": "Task"}
            }))
            .await
            .unwrap();
        client
            .batch_create_issues(
                vec![json!({
                    "fields": {
                        "project": {"key": "ABC"},
                        "summary": "Batch",
                        "issuetype": {"name": "Task"}
                    }
                })],
                false,
            )
            .await
            .unwrap();
        client
            .update_issue(
                "ABC-1".to_string(),
                json!({"summary": "Updated"}),
                Some(json!({"priority": {"name": "High"}})),
                Some(false),
            )
            .await
            .unwrap();
        client
            .delete_issue("ABC-1".to_string(), true)
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(created["data"]["key"], "ABC-1");
        assert_eq!(requests[0].method, Method::POST);
        assert_eq!(requests[0].path, "/rest/api/2/issue");
        assert_eq!(requests[0].body["fields"]["summary"], "Demo");
        assert_eq!(requests[1].path, "/rest/api/2/issue/bulk");
        assert_eq!(
            requests[1].body["issueUpdates"][0]["fields"]["summary"],
            "Batch"
        );
        assert_eq!(requests[2].method, Method::PUT);
        assert_eq!(
            requests[2].path,
            "/rest/api/2/issue/ABC-1?notifyUsers=false"
        );
        assert_eq!(requests[2].body["fields"]["priority"]["name"], "High");
        assert_eq!(requests[3].method, Method::DELETE);
        assert_eq!(
            requests[3].path,
            "/rest/api/2/issue/ABC-1?deleteSubtasks=true"
        );
    }

    #[tokio::test]
    async fn cloud_issue_create_and_update_use_v3_for_adf_fields() {
        let (base_url, requests) =
            mock_server(json!({"id": "10001", "key": "ABC-1", "fields": {}})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
        let description = json!({
            "type": "doc",
            "version": 1,
            "content": [{
                "type": "paragraph",
                "content": [{"type": "text", "text": "Cloud description"}]
            }]
        });

        client
            .create_issue(json!({
                "project": {"key": "ABC"},
                "summary": "Demo",
                "issuetype": {"name": "Task"},
                "description": description.clone()
            }))
            .await
            .unwrap();
        client
            .update_issue(
                "ABC-1".to_string(),
                json!({"description": description}),
                None,
                None,
            )
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(requests[0].method, Method::POST);
        assert_eq!(requests[0].path, "/rest/api/3/issue");
        assert_eq!(requests[0].body["fields"]["description"]["type"], "doc");
        assert_eq!(requests[1].method, Method::PUT);
        assert_eq!(requests[1].path, "/rest/api/3/issue/ABC-1");
        assert_eq!(requests[1].body["fields"]["description"]["type"], "doc");
    }

    #[tokio::test]
    async fn stage_three_changelog_and_product_dependency_helpers_are_safe() {
        let (cloud_url, cloud_requests) = stage_three_mock_server().await;
        let cloud = JiraClient::new(config(cloud_url, JiraDeployment::Cloud)).unwrap();
        let changelog = cloud
            .batch_get_changelogs(
                vec!["ABC-1".to_string()],
                Some(vec!["status".to_string()]),
                Some(50),
            )
            .await
            .unwrap();
        let cloud_requests = cloud_requests.lock().await;

        assert_eq!(changelog["issueChangeLogs"][0]["issueId"], "10001");
        assert_eq!(cloud_requests[0].path, "/rest/api/3/changelog/bulkfetch");
        assert_eq!(cloud_requests[0].body["issueIdsOrKeys"][0], "ABC-1");
        assert_eq!(cloud_requests[0].body["fieldIds"][0], "status");
        assert_eq!(cloud_requests[0].body["maxResults"], 50);

        let (server_url, server_requests) = stage_three_mock_server().await;
        let server = JiraClient::new(config(server_url, JiraDeployment::ServerDataCenter)).unwrap();
        let unsupported = server
            .batch_get_changelogs(vec!["ABC-1".to_string()], None, None)
            .await
            .unwrap();
        let forms = server
            .get_issue_proforma_forms("ABC-1".to_string(), None)
            .await
            .unwrap();
        let sla = server
            .get_issue_sla("ABC-1".to_string(), None, None, false)
            .await
            .unwrap();
        let server_requests = server_requests.lock().await;

        assert_eq!(unsupported["success"], false);
        assert_eq!(unsupported["product_dependency"]["available"], false);
        assert_eq!(forms["product_dependency"]["available"], false);
        assert_eq!(sla["success"], true);
        assert_eq!(sla["product_dependency"]["available"], true);
        assert_eq!(sla["metrics"][0]["field_id"], "customfield_sla");
        assert_eq!(
            server_requests[0].path,
            "/rest/api/2/issue/ABC-1?fields=*all"
        );
    }

    #[tokio::test]
    async fn stage_three_common_extension_helpers_use_expected_endpoints() {
        let (base_url, requests) = stage_three_mock_server().await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

        client.get_all_projects(false).await.unwrap();
        client
            .get_project_versions("ABC".to_string())
            .await
            .unwrap();
        client
            .get_project_components("ABC".to_string())
            .await
            .unwrap();
        client
            .create_version(json!({"project": "ABC", "name": "v1"}))
            .await
            .unwrap();
        client.get_user_profile("ada".to_string()).await.unwrap();
        client
            .get_issue_watchers("ABC-1".to_string())
            .await
            .unwrap();
        client
            .add_watcher("ABC-1".to_string(), "ada".to_string())
            .await
            .unwrap();
        client
            .remove_watcher("ABC-1".to_string(), "ada".to_string())
            .await
            .unwrap();
        client
            .get_worklog("ABC-1".to_string(), Some(0), Some(10))
            .await
            .unwrap();
        client
            .add_worklog(
                "ABC-1".to_string(),
                json!({"timeSpent": "1h"}),
                vec![("adjustEstimate".to_string(), "auto".to_string())],
            )
            .await
            .unwrap();
        client.get_link_types().await.unwrap();
        client
            .link_to_epic("ABC-1".to_string(), "ABC-EPIC".to_string())
            .await
            .unwrap();
        client
            .create_issue_link(json!({
                "type": {"name": "Blocks"},
                "inwardIssue": {"key": "ABC-1"},
                "outwardIssue": {"key": "ABC-2"}
            }))
            .await
            .unwrap();
        client
            .create_remote_issue_link(
                "ABC-1".to_string(),
                json!({"object": {"url": "https://example.invalid", "title": "Example"}}),
            )
            .await
            .unwrap();
        client.remove_issue_link("200".to_string()).await.unwrap();
        let requests = requests.lock().await;

        assert_eq!(
            requests[0].path,
            "/rest/api/2/project?includeArchived=false"
        );
        assert_eq!(requests[1].path, "/rest/api/2/project/ABC/versions");
        assert_eq!(requests[2].path, "/rest/api/2/project/ABC/components");
        assert_eq!(requests[3].path, "/rest/api/2/version");
        assert_eq!(requests[4].path, "/rest/api/2/user?username=ada");
        assert_eq!(requests[5].path, "/rest/api/2/issue/ABC-1/watchers");
        assert_eq!(requests[6].method, Method::POST);
        assert_eq!(requests[7].method, Method::DELETE);
        assert_eq!(
            requests[8].path,
            "/rest/api/2/issue/ABC-1/worklog?startAt=0&maxResults=10"
        );
        assert_eq!(requests[9].body["timeSpent"], "1h");
        assert_eq!(requests[10].path, "/rest/api/2/issueLinkType");
        assert_eq!(requests[11].path, "/rest/api/2/issue/ABC-1");
        assert_eq!(requests[12].path, "/rest/api/2/issueLink");
        assert_eq!(requests[13].path, "/rest/api/2/issue/ABC-1/remotelink");
        assert_eq!(requests[14].path, "/rest/api/2/issueLink/200");
    }

    #[tokio::test]
    async fn cloud_remove_watcher_uses_account_id_query_parameter() {
        let (base_url, requests) = stage_three_mock_server().await;
        let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();

        client
            .remove_watcher("ABC-1".to_string(), "account-1".to_string())
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(requests[0].method, Method::DELETE);
        assert_eq!(
            requests[0].path,
            "/rest/api/2/issue/ABC-1/watchers?accountId=account-1"
        );
    }

    #[tokio::test]
    async fn development_info_resolves_issue_key_to_numeric_id() {
        let (base_url, requests) = stage_three_mock_server().await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

        client
            .get_issue_development_info(
                "ABC-1".to_string(),
                Some("github".to_string()),
                Some("pullrequest".to_string()),
            )
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(requests[0].path, "/rest/api/2/issue/ABC-1?fields=id%2Ckey");
        assert_eq!(
            requests[1].path,
            "/rest/dev-status/1.0/issue/detail?issueId=10001&applicationType=github&dataType=pullrequest"
        );
    }

    #[tokio::test]
    async fn stage_three_product_extension_helpers_use_expected_endpoints() {
        let (base_url, requests) = stage_three_mock_server().await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

        client
            .get_agile_boards(
                Some("ABC".to_string()),
                Some("scrum".to_string()),
                Some(0),
                Some(10),
            )
            .await
            .unwrap();
        client
            .get_board_issues(
                1,
                Some("project = ABC".to_string()),
                Some(vec!["summary".to_string()]),
                Some(0),
                Some(10),
            )
            .await
            .unwrap();
        client
            .get_sprints_from_board(1, Some(vec!["active".to_string()]), Some(0), Some(10))
            .await
            .unwrap();
        client
            .get_sprint_issues(2, Some(vec!["summary".to_string()]), Some(0), Some(10))
            .await
            .unwrap();
        client
            .create_sprint(json!({"name": "Sprint", "originBoardId": 1}))
            .await
            .unwrap();
        client
            .update_sprint(2, json!({"name": "Sprint updated"}))
            .await
            .unwrap();
        client
            .add_issues_to_sprint(2, vec!["ABC-1".to_string()])
            .await
            .unwrap();
        client
            .get_service_desk_for_project("ABC".to_string())
            .await
            .unwrap();
        client
            .get_service_desk_queues("4".to_string(), Some(0), Some(50))
            .await
            .unwrap();
        client
            .get_queue_issues("4".to_string(), "47".to_string(), Some(0), Some(50))
            .await
            .unwrap();
        client
            .get_issue_development_info(
                "10001".to_string(),
                Some("github".to_string()),
                Some("pullrequest".to_string()),
            )
            .await
            .unwrap();
        client
            .get_issues_development_info(vec!["10001".to_string()], None, None)
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert!(requests[0].path.starts_with("/rest/agile/1.0/board?"));
        assert!(
            requests[1]
                .path
                .starts_with("/rest/agile/1.0/board/1/issue?")
        );
        assert!(
            requests[2]
                .path
                .starts_with("/rest/agile/1.0/board/1/sprint?")
        );
        assert!(
            requests[3]
                .path
                .starts_with("/rest/agile/1.0/sprint/2/issue?")
        );
        assert_eq!(requests[4].path, "/rest/agile/1.0/sprint");
        assert_eq!(requests[5].path, "/rest/agile/1.0/sprint/2");
        assert_eq!(requests[6].path, "/rest/agile/1.0/sprint/2/issue");
        assert_eq!(requests[7].path, "/rest/servicedeskapi/servicedesk");
        assert_eq!(
            requests[8].path,
            "/rest/servicedeskapi/servicedesk/4/queue?start=0&limit=50"
        );
        assert_eq!(
            requests[9].path,
            "/rest/servicedeskapi/servicedesk/4/queue/47/issue?start=0&limit=50"
        );
        assert!(
            requests[10]
                .path
                .starts_with("/rest/dev-status/1.0/issue/detail?")
        );
        assert!(
            requests[11]
                .path
                .starts_with("/rest/dev-status/1.0/issue/detail?")
        );
    }

    #[tokio::test]
    async fn agile_helpers_return_product_unavailable_when_software_rest_is_missing() {
        let (base_url, requests) = stage_three_mock_server().await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

        let value = client
            .get_agile_boards(Some("NOAGILE".to_string()), None, None, None)
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value["success"], false);
        assert_eq!(value["product_dependency"]["available"], false);
        assert_eq!(
            value["product_dependency"]["product"],
            "Jira Software Agile REST"
        );
        assert!(
            value["message"]
                .as_str()
                .unwrap()
                .contains("Jira Software is not available")
        );
        assert_eq!(
            requests[0].path,
            "/rest/agile/1.0/board?projectKeyOrId=NOAGILE"
        );
    }

    #[tokio::test]
    async fn service_desk_helpers_return_product_unavailable_when_jsm_rest_is_missing() {
        let (base_url, requests) = stage_three_mock_server().await;
        let client = JiraClient::new(config(
            format!("{base_url}/jsm-down"),
            JiraDeployment::ServerDataCenter,
        ))
        .unwrap();

        let value = client
            .get_service_desk_for_project("ABC".to_string())
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value["success"], false);
        assert_eq!(value["product_dependency"]["available"], false);
        assert_eq!(
            value["product_dependency"]["product"],
            "Jira Service Management"
        );
        assert!(
            value["message"]
                .as_str()
                .unwrap()
                .contains("Jira Service Management is not available")
        );
        assert_eq!(
            requests[0].path,
            "/jsm-down/rest/servicedeskapi/servicedesk"
        );
    }

    #[tokio::test]
    async fn development_helper_returns_product_unavailable_when_dev_status_is_missing() {
        let (base_url, requests) = stage_three_mock_server().await;
        let client = JiraClient::new(config(
            format!("{base_url}/dev-down"),
            JiraDeployment::ServerDataCenter,
        ))
        .unwrap();

        let value = client
            .get_issue_development_info("10001".to_string(), None, None)
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value["success"], false);
        assert_eq!(value["product_dependency"]["available"], false);
        assert_eq!(
            value["product_dependency"]["product"],
            "Jira development/dev-status"
        );
        assert!(
            value["message"]
                .as_str()
                .unwrap()
                .contains("Jira development status is not available")
        );
        assert_eq!(
            requests[0].path,
            "/dev-down/rest/dev-status/1.0/issue/detail?issueId=10001"
        );
    }

    #[tokio::test]
    async fn forms_helpers_use_cloud_id_paths_and_config_auth_without_override() {
        let (base_url, requests) = stage_three_mock_server().await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

        let forms = client
            .get_issue_proforma_forms("ABC-1".to_string(), Some("cloud-123"))
            .await
            .unwrap();
        let details = client
            .get_proforma_form_details("ABC-1".to_string(), "form-1".to_string(), Some("cloud-123"))
            .await
            .unwrap();
        let updated = client
            .update_proforma_form_answers(
                "ABC-1".to_string(),
                "form-1".to_string(),
                vec![
                    json!({"questionId": "q1", "type": "TEXT", "value": "Updated"}),
                    json!({"questionId": "q2", "type": "SELECT", "value": "Product A"}),
                    json!({"question_id": "q3", "type": "MULTI_USER", "value": ["abc"]}),
                ],
                Some("cloud-123"),
            )
            .await
            .unwrap();
        let requests = requests.lock().await;
        let expected_header = format!("Bearer {}", "test-pat-value");

        assert_eq!(forms["forms"][0]["id"], "form-1");
        assert_eq!(details["answers"]["q1"]["text"], "Existing");
        assert_eq!(updated["updated"], true);
        assert_eq!(
            requests[0].path,
            "/jira/forms/cloud/cloud-123/issue/ABC-1/form"
        );
        assert_eq!(
            requests[1].path,
            "/jira/forms/cloud/cloud-123/issue/ABC-1/form/form-1"
        );
        assert_eq!(requests[2].method, Method::PUT);
        assert_eq!(
            requests[2].path,
            "/jira/forms/cloud/cloud-123/issue/ABC-1/form/form-1"
        );
        for request in requests.iter() {
            assert_eq!(
                request.authorization.as_deref(),
                Some(expected_header.as_str())
            );
        }
        assert_eq!(requests[2].body["answers"]["q1"]["text"], "Updated");
        assert_eq!(
            requests[2].body["answers"]["q2"]["choices"],
            json!(["Product A"])
        );
        assert_eq!(requests[2].body["answers"]["q3"]["users"], json!(["abc"]));
    }

    #[tokio::test]
    async fn forms_helpers_return_product_unavailable_when_cloud_id_missing_without_http() {
        let (base_url, requests) = stage_three_mock_server().await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

        let missing = client
            .get_issue_proforma_forms("ABC-1".to_string(), None)
            .await
            .unwrap();
        let blank = client
            .get_proforma_form_details("ABC-1".to_string(), "form-1".to_string(), Some(" "))
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(missing["success"], false);
        assert_eq!(missing["product_dependency"]["available"], false);
        assert_eq!(
            missing["product_dependency"]["product"],
            "Jira Forms/ProForma Cloud ID"
        );
        assert_eq!(blank["product_dependency"]["available"], false);
        assert!(requests.is_empty());
    }

    #[tokio::test]
    async fn forms_helpers_return_product_unavailable_when_forms_api_is_missing() {
        let (base_url, requests) = stage_three_mock_server().await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

        let value = client
            .get_issue_proforma_forms("ABC-1".to_string(), Some("forms-down"))
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value["success"], false);
        assert_eq!(value["product_dependency"]["available"], false);
        assert_eq!(
            value["product_dependency"]["product"],
            "Jira Forms/ProForma"
        );
        assert!(
            value["message"]
                .as_str()
                .unwrap()
                .contains("Jira Forms is not available")
        );
        assert_eq!(
            requests[0].path,
            "/jira/forms/cloud/forms-down/issue/ABC-1/form"
        );
    }

    #[tokio::test]
    async fn stage_three_attachment_helpers_use_bounded_content_fetch() {
        let (base_url, requests) = stage_three_mock_server().await;
        let client =
            JiraClient::new(config(base_url.clone(), JiraDeployment::ServerDataCenter)).unwrap();

        let attachments = client
            .get_issue_attachments("ABC-1".to_string())
            .await
            .unwrap();
        let content = client
            .fetch_attachment_content("/secure/attachment/1/file.png", 20)
            .await
            .unwrap();
        let oversized = client
            .fetch_attachment_content("/secure/attachment/1/file.png", 2)
            .await
            .unwrap_err()
            .to_string();
        let absolute = client
            .fetch_attachment_content(
                &format!("{base_url}/secure/attachment/1/file.png?token=secret"),
                20,
            )
            .await
            .unwrap();
        let blocked_external = client
            .fetch_attachment_content("https://evil.example/attachment.png?token=secret", 20)
            .await
            .unwrap_err()
            .to_string();
        let requests = requests.lock().await;

        assert_eq!(attachments.len(), 2);
        assert!(attachments[0].is_image());
        assert_eq!(content.content_type.as_deref(), Some("image/png"));
        assert_eq!(content.bytes, b"image-bytes");
        assert_eq!(absolute.bytes, b"image-bytes");
        assert!(oversized.contains("exceeds configured limit"));
        assert!(
            blocked_external.contains("absolute URL must use the configured Atlassian base origin")
        );
        assert!(!blocked_external.contains("token=secret"));
        assert_eq!(
            requests[0].path,
            "/rest/api/2/issue/ABC-1?fields=attachment"
        );
        assert_eq!(requests[1].path, "/secure/attachment/1/file.png");
        assert_eq!(requests[2].path, "/secure/attachment/1/file.png");
        assert_eq!(
            requests[3].path,
            "/secure/attachment/1/file.png?token=secret"
        );
    }

    #[tokio::test]
    async fn safe_attachment_helper_filters_images_and_redacts_content_errors() {
        let (base_url, requests) = stage_three_mock_server().await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

        let with_content = client
            .get_safe_issue_attachments(
                "ABC-1".to_string(),
                AttachmentFetchOptions {
                    attachment_ids: Some(vec!["1".to_string(), "2".to_string()]),
                    include_content: true,
                    images_only: false,
                    max_bytes: 20,
                },
            )
            .await
            .unwrap();
        let images_only = client
            .get_safe_issue_attachments(
                "ABC-1".to_string(),
                AttachmentFetchOptions {
                    images_only: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let oversized = client
            .get_safe_issue_attachments(
                "ABC-1".to_string(),
                AttachmentFetchOptions {
                    attachment_ids: Some(vec!["1".to_string()]),
                    include_content: true,
                    max_bytes: 2,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(with_content["count"], 2);
        assert_eq!(with_content["attachments"][0]["filename"], "file.png");
        assert_eq!(with_content["attachments"][0]["has_content_url"], true);
        assert!(with_content["attachments"][0].get("thumbnail").is_none());
        assert_eq!(
            with_content["attachments"][0]["content"],
            json!({
                "encoding": "base64",
                "content_type": "image/png",
                "size": 11,
                "data": "aW1hZ2UtYnl0ZXM="
            })
        );
        let error = with_content["attachments"][1]["content_error"]["message"]
            .as_str()
            .unwrap();
        assert!(error.contains("/secure/attachment/2/notes.txt?<redacted>"));
        assert!(!error.contains("token=secret"));
        assert!(!error.contains("client=abc"));
        assert_eq!(images_only["count"], 1);
        assert_eq!(images_only["attachments"][0]["filename"], "file.png");
        assert_eq!(oversized["count"], 1);
        assert!(
            oversized["attachments"][0]["content_error"]["message"]
                .as_str()
                .unwrap()
                .contains("exceeds configured limit")
        );
        assert_eq!(
            requests[1].path,
            "/secure/attachment/1/file.png?token=secret"
        );
        assert_eq!(
            requests[2].path,
            "/secure/attachment/2/notes.txt?token=secret&client=abc"
        );
    }
}
