use crate::{
    confluence::{
        client::{ConfluenceEmojiStatus, MAX_SEARCH_LIMIT},
        formatting::ConfluenceContentFormat,
        models::ConfluencePage,
    },
    mcp_errors::upstream_error,
    upstream::error::UpstreamError,
};
use rmcp::ErrorData;
use serde_json::{Value, json};

use super::optional_non_empty_arg;

pub(super) fn optional_confluence_search_limit_arg(
    value: Option<u64>,
) -> Result<Option<u64>, ErrorData> {
    match value {
        Some(0) => Err(upstream_error(UpstreamError::invalid_input(
            "limit must be positive",
        ))),
        Some(value) if value > MAX_SEARCH_LIMIT => {
            Err(upstream_error(UpstreamError::invalid_input(format!(
                "limit must be less than or equal to {}",
                MAX_SEARCH_LIMIT
            ))))
        }
        value => Ok(value),
    }
}

pub(super) fn optional_u64_range_arg(
    value: Option<u64>,
    default: u64,
    max: u64,
    field_name: &'static str,
) -> Result<u64, ErrorData> {
    match value.unwrap_or(default) {
        0 => Err(upstream_error(UpstreamError::invalid_input(format!(
            "{field_name} must be positive"
        )))),
        value if value > max => Err(upstream_error(UpstreamError::invalid_input(format!(
            "{field_name} must be less than or equal to {max}"
        )))),
        value => Ok(value),
    }
}

pub(super) fn confluence_page_tool_value(
    page: &ConfluencePage,
    include_metadata: bool,
    convert_to_markdown: bool,
) -> Value {
    let simplified = page.to_simplified_value(convert_to_markdown);
    if include_metadata {
        json!({ "metadata": simplified })
    } else {
        json!({ "content": { "value": simplified.get("content").cloned().unwrap_or(Value::Null) } })
    }
}

pub(super) fn parse_confluence_write_content_format(
    value: Option<&str>,
) -> Result<ConfluenceContentFormat, ErrorData> {
    let format = ConfluenceContentFormat::parse(value).map_err(upstream_error)?;
    if format == ConfluenceContentFormat::Html {
        return Err(upstream_error(UpstreamError::invalid_input(
            "content_format must be markdown, wiki, or storage",
        )));
    }
    Ok(format)
}

pub(super) fn confluence_user_search_limit(value: Option<u64>) -> Result<u64, ErrorData> {
    match value.unwrap_or(10) {
        0 => Err(upstream_error(UpstreamError::invalid_input(
            "limit must be positive",
        ))),
        value if value > 50 => Err(upstream_error(UpstreamError::invalid_input(
            "limit must be less than or equal to 50",
        ))),
        value => Ok(value),
    }
}

pub(super) fn confluence_positive_version_arg(
    value: u64,
    field_name: &'static str,
) -> Result<u64, ErrorData> {
    if value == 0 {
        Err(upstream_error(UpstreamError::invalid_input(format!(
            "{field_name} must be positive"
        ))))
    } else {
        Ok(value)
    }
}

pub(super) fn normalize_confluence_user_search_query(query: &str) -> String {
    if ["=", "~", ">", "<", " AND ", " OR ", "user."]
        .iter()
        .any(|token| query.contains(token))
    {
        query.to_string()
    } else {
        format!(
            "user.fullname ~ \"{}\"",
            query.replace('\\', "\\\\").replace('"', "\\\"")
        )
    }
}

pub(super) fn confluence_page_markdown_content(page: &ConfluencePage) -> String {
    page.to_simplified_value(true)
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

pub(super) fn confluence_unified_diff(
    from_content: &str,
    to_content: &str,
    from_version: u64,
    to_version: u64,
) -> String {
    if from_content == to_content {
        return String::new();
    }

    let from_lines = from_content.lines().collect::<Vec<_>>();
    let to_lines = to_content.lines().collect::<Vec<_>>();
    let mut output = vec![
        format!("--- v{from_version}"),
        format!("+++ v{to_version}"),
        format!(
            "@@ -{} +{} @@",
            confluence_diff_range(from_lines.len()),
            confluence_diff_range(to_lines.len())
        ),
    ];
    let max_len = from_lines.len().max(to_lines.len());

    for index in 0..max_len {
        match (from_lines.get(index), to_lines.get(index)) {
            (Some(left), Some(right)) if left == right => output.push(format!(" {left}")),
            (Some(left), Some(right)) => {
                output.push(format!("-{left}"));
                output.push(format!("+{right}"));
            }
            (Some(left), None) => output.push(format!("-{left}")),
            (None, Some(right)) => output.push(format!("+{right}")),
            (None, None) => {}
        }
    }

    output.join("\n")
}

pub(super) fn confluence_diff_range(line_count: usize) -> String {
    match line_count {
        0 => "0,0".to_string(),
        1 => "1".to_string(),
        value => format!("1,{value}"),
    }
}

pub(super) fn confluence_split_file_paths(value: &str) -> Result<Vec<String>, ErrorData> {
    let file_paths = value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if file_paths.is_empty() {
        Err(upstream_error(UpstreamError::invalid_input(
            "file_paths must contain at least one local file path",
        )))
    } else {
        Ok(file_paths)
    }
}

pub(super) fn confluence_write_page_value(page: &ConfluencePage, include_content: bool) -> Value {
    let mut value = page.to_simplified_value(false);
    if !include_content && let Some(object) = value.as_object_mut() {
        object.remove("content");
    }
    value
}

pub(super) fn confluence_emoji_missing_page_id_status(
    emoji: Option<&str>,
) -> ConfluenceEmojiStatus {
    let Some(emoji) = optional_non_empty_arg(emoji.map(ToString::to_string)) else {
        return ConfluenceEmojiStatus::not_requested();
    };

    ConfluenceEmojiStatus::failed(emoji, "Confluence page response did not include a page id")
}

pub(super) fn confluence_expand_list(expand: Option<String>, include_content: bool) -> Vec<String> {
    let mut values = expand
        .unwrap_or_else(|| "version".to_string())
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    if values.is_empty() {
        values.push("version".to_string());
    }
    if include_content && !values.iter().any(|value| value.contains("body")) {
        values.push("body.storage".to_string());
    }

    values
}

pub(super) fn confluence_child_page_value(
    page: &ConfluencePage,
    include_content: bool,
    convert_to_markdown: bool,
) -> Value {
    let mut value = page.to_simplified_value(convert_to_markdown);
    if !include_content && let Some(object) = value.as_object_mut() {
        object.remove("content");
    }
    value
}

#[derive(Debug)]
pub(super) struct ConfluenceTreePageSortValue {
    pub(super) depth: usize,
    pub(super) position_sort: i64,
    pub(super) title: String,
    pub(super) value: Value,
}

pub(super) fn confluence_tree_page_sort_value(
    page: &ConfluencePage,
) -> ConfluenceTreePageSortValue {
    let depth = page.ancestors.len();
    let parent_id = page
        .ancestors
        .last()
        .and_then(|ancestor| ancestor.id.clone());
    let position = page.extensions.get("position").and_then(Value::as_i64);
    let title = page
        .title
        .clone()
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| "Untitled".to_string());
    let value = json!({
        "id": page.id.clone(),
        "title": title,
        "parent_id": parent_id,
        "position": position,
        "depth": depth,
    });

    ConfluenceTreePageSortValue {
        depth,
        position_sort: position.unwrap_or(999_999),
        title,
        value,
    }
}
