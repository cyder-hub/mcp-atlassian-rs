use serde::{Deserialize, Deserializer, Serialize, de::Error as DeError};
use serde_json::{Value, json};

use crate::confluence::formatting::{body_value_as_markdown, body_value_as_storage};

fn optional_string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<Value>::deserialize(deserializer)? {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        Some(Value::Number(value)) => Ok(Some(value.to_string())),
        _ => Err(D::Error::custom("expected string, number, or null")),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluencePage {
    #[serde(default, deserialize_with = "optional_string_or_number")]
    pub id: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    pub status: Option<String>,
    #[serde(default)]
    pub space: Option<ConfluenceSpace>,
    #[serde(default)]
    pub body: Value,
    #[serde(default)]
    pub version: Option<ConfluenceVersion>,
    #[serde(default)]
    pub ancestors: Vec<ConfluencePageSummary>,
    #[serde(default)]
    pub metadata: Value,
    #[serde(default)]
    pub extensions: Value,
    #[serde(rename = "_links", default)]
    pub links: Value,
    #[serde(flatten)]
    pub extra: Value,
}

impl ConfluencePage {
    pub fn to_simplified_value(&self, convert_to_markdown: bool) -> Value {
        let content = if convert_to_markdown {
            body_value_as_markdown(&self.body)
        } else {
            body_value_as_storage(&self.body)
        };

        json!({
            "id": self.id,
            "title": self.title,
            "type": self.content_type,
            "status": self.status,
            "space": self.space.as_ref().map(ConfluenceSpace::to_simplified_value),
            "version": self.version.as_ref().map(ConfluenceVersion::to_simplified_value),
            "ancestors": self.ancestors.iter().map(ConfluencePageSummary::to_simplified_value).collect::<Vec<_>>(),
            "content": content,
            "metadata": self.metadata,
            "links": self.links,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluencePageSummary {
    #[serde(default, deserialize_with = "optional_string_or_number")]
    pub id: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    pub status: Option<String>,
    #[serde(default)]
    pub space: Option<ConfluenceSpace>,
    #[serde(rename = "_links", default)]
    pub links: Value,
    #[serde(flatten)]
    pub extra: Value,
}

impl ConfluencePageSummary {
    pub fn to_simplified_value(&self) -> Value {
        json!({
            "id": self.id,
            "title": self.title,
            "type": self.content_type,
            "status": self.status,
            "space": self.space.as_ref().map(ConfluenceSpace::to_simplified_value),
            "links": self.links,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluencePageListResponse {
    #[serde(default)]
    pub results: Vec<ConfluencePage>,
    pub start: Option<u64>,
    pub limit: Option<u64>,
    pub size: Option<u64>,
    #[serde(rename = "_links", default)]
    pub links: Value,
    #[serde(flatten)]
    pub extra: Value,
}

impl ConfluencePageListResponse {
    pub fn has_next_link(&self) -> bool {
        has_next_link(&self.links)
    }

    pub fn next_start(&self) -> Option<u64> {
        next_start_from_links(&self.links)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceSpace {
    #[serde(default, deserialize_with = "optional_string_or_number")]
    pub id: Option<String>,
    pub key: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub space_type: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(rename = "_links", default)]
    pub links: Value,
    #[serde(flatten)]
    pub extra: Value,
}

impl ConfluenceSpace {
    pub fn to_simplified_value(&self) -> Value {
        json!({
            "id": self.id,
            "key": self.key,
            "name": self.name,
            "type": self.space_type,
            "status": self.status,
            "links": self.links,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceVersion {
    pub number: Option<u64>,
    pub message: Option<String>,
    pub minor_edit: Option<bool>,
    pub when: Option<String>,
    #[serde(default)]
    pub by: Value,
    #[serde(flatten)]
    pub extra: Value,
}

impl ConfluenceVersion {
    pub fn to_simplified_value(&self) -> Value {
        json!({
            "number": self.number,
            "message": self.message,
            "minor_edit": self.minor_edit,
            "when": self.when,
            "by": simplify_user_value(&self.by),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceSearchResponse {
    #[serde(default)]
    pub results: Vec<ConfluenceSearchResult>,
    pub start: Option<u64>,
    pub limit: Option<u64>,
    pub size: Option<u64>,
    pub total_size: Option<u64>,
    #[serde(rename = "_links", default)]
    pub links: Value,
    #[serde(flatten)]
    pub extra: Value,
}

impl ConfluenceSearchResponse {
    pub fn to_simplified_value(&self) -> Value {
        json!({
            "results": self.results.iter().map(ConfluenceSearchResult::to_simplified_value).collect::<Vec<_>>(),
            "start": self.start,
            "limit": self.limit,
            "size": self.size,
            "total_size": self.total_size,
            "links": self.links,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceSearchResult {
    #[serde(default, deserialize_with = "optional_string_or_number")]
    pub id: Option<String>,
    pub title: Option<String>,
    pub excerpt: Option<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub content: Option<ConfluencePageSummary>,
    #[serde(default)]
    pub space: Option<ConfluenceSpace>,
    #[serde(flatten)]
    pub extra: Value,
}

impl ConfluenceSearchResult {
    pub fn to_simplified_value(&self) -> Value {
        json!({
            "id": self.id,
            "title": self.title,
            "excerpt": self.excerpt,
            "url": self.url,
            "content": self.content.as_ref().map(ConfluencePageSummary::to_simplified_value),
            "space": self.space.as_ref().map(ConfluenceSpace::to_simplified_value),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceComment {
    #[serde(default, deserialize_with = "optional_string_or_number")]
    pub id: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    #[serde(default)]
    pub body: Value,
    #[serde(default)]
    pub version: Option<ConfluenceVersion>,
    #[serde(default)]
    pub author: Value,
    pub created: Option<String>,
    pub updated: Option<String>,
    #[serde(default, deserialize_with = "optional_string_or_number")]
    pub parent_comment_id: Option<String>,
    #[serde(default)]
    pub ancestors: Vec<ConfluencePageSummary>,
    #[serde(default)]
    pub container: Option<ConfluencePageSummary>,
    #[serde(default)]
    pub extensions: Value,
    #[serde(rename = "_links", default)]
    pub links: Value,
    #[serde(flatten)]
    pub extra: Value,
}

impl ConfluenceComment {
    pub fn to_simplified_value(&self) -> Value {
        let parent_comment_id = self.parent_comment_id.as_deref().or_else(|| {
            self.container
                .as_ref()
                .filter(|container| container.content_type.as_deref() == Some("comment"))
                .and_then(|container| container.id.as_deref())
        });
        let author = if self.author.is_null() {
            self.version
                .as_ref()
                .map(|version| simplify_user_value(&version.by))
                .unwrap_or(Value::Null)
        } else {
            simplify_user_value(&self.author)
        };

        json!({
            "id": self.id,
            "title": self.title,
            "type": self.content_type,
            "body": body_value_as_markdown(&self.body),
            "created": self.created,
            "updated": self.updated,
            "author": author,
            "version": self.version.as_ref().map(ConfluenceVersion::to_simplified_value),
            "parent_comment_id": parent_comment_id,
            "location": self.extensions.get("location").and_then(Value::as_str),
            "container": self.container.as_ref().map(ConfluencePageSummary::to_simplified_value),
            "ancestors": self.ancestors.iter().map(ConfluencePageSummary::to_simplified_value).collect::<Vec<_>>(),
            "links": self.links,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceCommentListResponse {
    #[serde(default)]
    pub results: Vec<ConfluenceComment>,
    pub start: Option<u64>,
    pub limit: Option<u64>,
    pub size: Option<u64>,
    #[serde(rename = "_links", default)]
    pub links: Value,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceLabel {
    #[serde(default, deserialize_with = "optional_string_or_number")]
    pub id: Option<String>,
    pub name: Option<String>,
    pub prefix: Option<String>,
    pub label: Option<String>,
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

impl ConfluenceLabel {
    pub fn to_simplified_value(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "prefix": self.prefix,
            "label": self.label,
            "type": self.content_type,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceLabelListResponse {
    #[serde(default)]
    pub results: Vec<ConfluenceLabel>,
    pub start: Option<u64>,
    pub limit: Option<u64>,
    pub size: Option<u64>,
    #[serde(rename = "_links", default)]
    pub links: Value,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceUser {
    pub account_id: Option<String>,
    pub username: Option<String>,
    pub user_key: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    #[serde(default)]
    pub active: Option<bool>,
    pub account_status: Option<String>,
    #[serde(default)]
    pub profile_picture: Value,
    pub locale: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

impl ConfluenceUser {
    pub fn to_simplified_value(&self) -> Value {
        let active = self.active.or_else(|| {
            self.account_status
                .as_ref()
                .map(|status| status.eq_ignore_ascii_case("active"))
        });

        json!({
            "account_id": self.account_id,
            "username": self.username,
            "user_key": self.user_key,
            "display_name": self.display_name,
            "email": self.email,
            "profile_picture": self.profile_picture.get("path").and_then(Value::as_str),
            "active": active,
            "locale": self.locale,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceUserSearchResponse {
    #[serde(default)]
    pub results: Vec<ConfluenceUserSearchResult>,
    pub start: Option<u64>,
    pub limit: Option<u64>,
    pub size: Option<u64>,
    pub total_size: Option<u64>,
    pub cql_query: Option<String>,
    pub search_duration: Option<u64>,
    #[serde(rename = "_links", default)]
    pub links: Value,
    #[serde(flatten)]
    pub extra: Value,
}

impl ConfluenceUserSearchResponse {
    pub fn to_simplified_value(&self) -> Value {
        json!({
            "results": self.results.iter().map(ConfluenceUserSearchResult::to_simplified_value).collect::<Vec<_>>(),
            "start": self.start,
            "limit": self.limit,
            "size": self.size,
            "total_size": self.total_size,
            "cql_query": self.cql_query,
            "search_duration": self.search_duration,
            "links": self.links,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceUserSearchResult {
    #[serde(default)]
    pub user: Option<ConfluenceUser>,
    pub title: Option<String>,
    pub excerpt: Option<String>,
    pub url: Option<String>,
    pub entity_type: Option<String>,
    pub last_modified: Option<String>,
    pub score: Option<f64>,
    #[serde(flatten)]
    pub extra: Value,
}

impl ConfluenceUserSearchResult {
    pub fn from_user(mut user: ConfluenceUser, title: Option<String>) -> Self {
        if user.active.is_none() && user.account_status.is_none() {
            user.active = Some(true);
        }
        Self {
            title,
            entity_type: Some("user".to_string()),
            user: Some(user),
            ..Self::default()
        }
    }

    pub fn to_simplified_value(&self) -> Value {
        json!({
            "entity_type": self.entity_type.as_deref().unwrap_or("user"),
            "title": self.title,
            "score": self.score.unwrap_or(0.0),
            "user": self.user.as_ref().map(ConfluenceUser::to_simplified_value),
            "url": self.url,
            "last_modified": self.last_modified,
            "excerpt": self.excerpt,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceUserListResponse {
    #[serde(default)]
    pub results: Vec<ConfluenceUser>,
    pub start: Option<u64>,
    pub limit: Option<u64>,
    pub size: Option<u64>,
    #[serde(rename = "_links", default)]
    pub links: Value,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceAttachment {
    #[serde(default, deserialize_with = "optional_string_or_number")]
    pub id: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    pub status: Option<String>,
    #[serde(default)]
    pub metadata: Value,
    #[serde(default)]
    pub extensions: Value,
    #[serde(rename = "_links", default)]
    pub links: Value,
    #[serde(flatten)]
    pub extra: Value,
}

impl ConfluenceAttachment {
    pub fn media_type(&self) -> Option<&str> {
        self.extensions
            .get("mediaType")
            .or_else(|| self.metadata.get("mediaType"))
            .or_else(|| self.extra.get("mediaType"))
            .and_then(Value::as_str)
    }

    pub fn file_size(&self) -> Option<u64> {
        self.extensions
            .get("fileSize")
            .or_else(|| self.metadata.get("fileSize"))
            .or_else(|| self.extra.get("fileSize"))
            .and_then(Value::as_u64)
    }

    pub fn to_simplified_value(&self) -> Value {
        json!({
            "id": self.id,
            "title": self.title,
            "type": self.content_type,
            "status": self.status,
            "media_type": self.media_type(),
            "file_size": self.file_size(),
            "download": self.links.get("download").and_then(Value::as_str),
            "metadata": self.metadata,
            "extensions": self.extensions,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceAttachmentListResponse {
    #[serde(default)]
    pub results: Vec<ConfluenceAttachment>,
    pub start: Option<u64>,
    pub limit: Option<u64>,
    pub size: Option<u64>,
    #[serde(rename = "_links", default)]
    pub links: Value,
    #[serde(flatten)]
    pub extra: Value,
}

impl ConfluenceAttachmentListResponse {
    pub fn has_next_link(&self) -> bool {
        has_next_link(&self.links)
    }

    pub fn next_start(&self) -> Option<u64> {
        next_start_from_links(&self.links)
    }
}

pub fn has_next_link(links: &Value) -> bool {
    links
        .get("next")
        .and_then(Value::as_str)
        .is_some_and(|link| !link.trim().is_empty())
}

pub fn next_start_from_links(links: &Value) -> Option<u64> {
    let next = links.get("next").and_then(Value::as_str)?;
    let query = next.split_once('?')?.1;
    query.split('&').find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        (name == "start")
            .then(|| value.parse::<u64>().ok())
            .flatten()
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfluencePageViews {
    pub count: Option<u64>,
    pub last_seen: Option<String>,
    pub unique_viewers: Option<u64>,
    #[serde(default, deserialize_with = "optional_string_or_number")]
    pub page_id: Option<String>,
    pub title: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

impl ConfluencePageViews {
    pub fn to_simplified_value(&self) -> Value {
        json!({
            "page_id": self.page_id,
            "page_title": self.title,
            "total_views": self.count.unwrap_or(0),
            "unique_viewers": self.unique_viewers,
            "last_viewed": self.last_seen,
            "raw": self.extra,
        })
    }
}

pub fn simplify_user_value(value: &Value) -> Value {
    json!({
        "account_id": value.get("accountId").and_then(Value::as_str),
        "username": value.get("username").and_then(Value::as_str),
        "user_key": value.get("userKey").and_then(Value::as_str),
        "display_name": value.get("displayName").and_then(Value::as_str),
        "email": value.get("email").and_then(Value::as_str),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn page_model_tolerates_missing_fields() {
        let page: ConfluencePage = serde_json::from_value(json!({})).unwrap();
        let simplified = page.to_simplified_value(true);

        assert!(simplified.get("id").is_some_and(Value::is_null));
        assert!(simplified.get("content").is_some_and(Value::is_null));
    }

    #[test]
    fn page_model_simplifies_common_fields() {
        let page: ConfluencePage = serde_json::from_value(json!({
            "id": "123",
            "title": "Roadmap",
            "type": "page",
            "space": {"id": "10", "key": "ENG", "name": "Engineering"},
            "version": {"number": 2, "by": {"displayName": "Ada"}},
            "body": {"storage": {"value": "<p>Hello</p>"}},
            "_links": {"webui": "/spaces/ENG/pages/123"}
        }))
        .unwrap();
        let simplified = page.to_simplified_value(true);

        assert_eq!(simplified["id"], "123");
        assert_eq!(simplified["space"]["key"], "ENG");
        assert_eq!(simplified["version"]["number"], 2);
        assert_eq!(simplified["content"], "Hello");
    }

    #[test]
    fn confluence_ids_accept_numbers_from_cloud_responses() {
        let page: ConfluencePage = serde_json::from_value(json!({
            "id": 123,
            "title": "Numeric ids",
            "space": {"id": 10, "key": "ENG"},
            "ancestors": [{"id": 99, "title": "Parent"}]
        }))
        .unwrap();
        let page = page.to_simplified_value(false);

        assert_eq!(page["id"], "123");
        assert_eq!(page["space"]["id"], "10");
        assert_eq!(page["ancestors"][0]["id"], "99");

        let comment: ConfluenceComment = serde_json::from_value(json!({
            "id": 456,
            "parentCommentId": 455,
            "container": {"id": 123, "type": "page"}
        }))
        .unwrap();
        let comment = comment.to_simplified_value();

        assert_eq!(comment["id"], "456");
        assert_eq!(comment["parent_comment_id"], "455");

        let attachment: ConfluenceAttachment = serde_json::from_value(json!({
            "id": 789,
            "title": "probe.txt"
        }))
        .unwrap();
        let attachment = attachment.to_simplified_value();

        assert_eq!(attachment["id"], "789");
    }

    #[test]
    fn comment_model_simplifies_author_parent_and_location() {
        let comment: ConfluenceComment = serde_json::from_value(json!({
            "id": "c-2",
            "type": "comment",
            "body": {"storage": {"value": "<p>Reply</p>"}},
            "version": {"number": 3, "by": {"displayName": "Ada"}},
            "container": {"id": "c-1", "type": "comment", "title": "Parent"},
            "extensions": {"location": "footer"},
            "_links": {"webui": "/spaces/ENG/pages/123?focusedCommentId=c-2"}
        }))
        .unwrap();
        let simplified = comment.to_simplified_value();

        assert_eq!(simplified["id"], "c-2");
        assert_eq!(simplified["body"], "Reply");
        assert_eq!(simplified["author"]["display_name"], "Ada");
        assert_eq!(simplified["parent_comment_id"], "c-1");
        assert_eq!(simplified["location"], "footer");
        assert_eq!(simplified["version"]["number"], 3);
    }

    #[test]
    fn label_model_simplifies_common_fields() {
        let label: ConfluenceLabel = serde_json::from_value(json!({
            "id": "label-1",
            "name": "draft",
            "prefix": "global",
            "label": "draft",
            "type": "label"
        }))
        .unwrap();
        let simplified = label.to_simplified_value();

        assert_eq!(simplified["id"], "label-1");
        assert_eq!(simplified["name"], "draft");
        assert_eq!(simplified["prefix"], "global");
        assert_eq!(simplified["label"], "draft");
        assert_eq!(simplified["type"], "label");
    }

    #[test]
    fn user_search_model_simplifies_user_fields() {
        let response: ConfluenceUserSearchResponse = serde_json::from_value(json!({
            "results": [{
                "title": "Ada Lovelace",
                "entityType": "user",
                "score": 0.9,
                "user": {
                    "accountId": "abc",
                    "displayName": "Ada Lovelace",
                    "email": "ada@example.com",
                    "accountStatus": "active",
                    "profilePicture": {"path": "/avatar/ada.png"},
                    "locale": "en-US"
                }
            }],
            "start": 0,
            "limit": 10,
            "totalSize": 1,
            "cqlQuery": "user.fullname ~ \"Ada\""
        }))
        .unwrap();
        let simplified = response.to_simplified_value();

        assert_eq!(simplified["results"][0]["title"], "Ada Lovelace");
        assert_eq!(simplified["results"][0]["user"]["account_id"], "abc");
        assert_eq!(simplified["results"][0]["user"]["active"], true);
        assert_eq!(
            simplified["results"][0]["user"]["profile_picture"],
            "/avatar/ada.png"
        );
        assert_eq!(simplified["cql_query"], "user.fullname ~ \"Ada\"");
    }

    #[test]
    fn attachment_model_extracts_media_metadata() {
        let attachment: ConfluenceAttachment = serde_json::from_value(json!({
            "id": "att-1",
            "title": "image.png",
            "type": "attachment",
            "extensions": {"mediaType": "image/png", "fileSize": 42},
            "_links": {"download": "/download/attachments/att-1/image.png"}
        }))
        .unwrap();
        let simplified = attachment.to_simplified_value();

        assert_eq!(simplified["type"], "attachment");
        assert_eq!(simplified["media_type"], "image/png");
        assert_eq!(simplified["file_size"], 42);
        assert_eq!(
            simplified["download"],
            "/download/attachments/att-1/image.png"
        );
    }

    #[test]
    fn attachment_model_falls_back_to_metadata_media_type() {
        let attachment: ConfluenceAttachment = serde_json::from_value(json!({
            "id": "att-2",
            "title": "notes.txt",
            "metadata": {"mediaType": "text/plain", "fileSize": 12}
        }))
        .unwrap();
        let simplified = attachment.to_simplified_value();

        assert_eq!(simplified["media_type"], "text/plain");
        assert_eq!(simplified["file_size"], 12);
    }

    #[test]
    fn page_views_model_normalizes_python_shape() {
        let views: ConfluencePageViews = serde_json::from_value(json!({
            "count": 42,
            "lastSeen": "2026-06-04T12:00:00Z",
            "uniqueViewers": 7,
            "unexpected": "kept"
        }))
        .unwrap();
        let simplified = views.to_simplified_value();

        assert_eq!(simplified["total_views"], 42);
        assert_eq!(simplified["unique_viewers"], 7);
        assert_eq!(simplified["last_viewed"], "2026-06-04T12:00:00Z");
        assert_eq!(simplified["raw"]["unexpected"], "kept");
    }
}
