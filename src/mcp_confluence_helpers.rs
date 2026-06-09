use std::path::Path;

use serde_json::{Value, json};

use crate::{
    atlassian::http::DownloadedContent,
    confluence::{client::ConfluenceClient, models::ConfluenceAttachment},
    jira::formatting::{base64_encode, redact_url_query},
};

pub(crate) async fn confluence_attachment_with_content_value(
    client: &ConfluenceClient,
    attachment: &ConfluenceAttachment,
    fallback_id: &str,
    max_bytes: u64,
) -> Result<Value, Value> {
    let attachment_id = confluence_attachment_id_with_fallback(attachment, fallback_id);
    let filename = confluence_attachment_filename(attachment, &attachment_id);
    let mut value = attachment.to_simplified_value();

    if let Some(file_size) = attachment.file_size()
        && file_size > max_bytes
    {
        return Err(json!({
            "success": false,
            "attachment_id": attachment_id,
            "filename": filename,
            "file_size": file_size,
            "max_bytes": max_bytes,
            "error": format!("Attachment '{filename}' is {file_size} bytes which exceeds the inline limit of {max_bytes} bytes."),
        }));
    }

    let Some(download_url) = attachment
        .links
        .get("download")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Err(json!({
            "success": false,
            "attachment_id": attachment_id,
            "filename": filename,
            "error": "download URL is missing",
        }));
    };

    let content = client
        .download_relative_or_same_origin(download_url, max_bytes)
        .await
        .map_err(|error| {
            json!({
                "success": false,
                "attachment_id": attachment_id,
                "filename": filename,
                "error": redact_url_query(&error.to_string()),
            })
        })?;
    let content_type = confluence_content_type_for_attachment(&content, attachment, &filename);

    value["content"] = json!({
        "encoding": "base64",
        "content_type": content_type,
        "size": content.bytes.len(),
        "data": base64_encode(&content.bytes),
    });

    Ok(value)
}

pub(crate) fn confluence_file_path_display(value: &str) -> String {
    Path::new(value)
        .file_name()
        .and_then(|value| value.to_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("attachment")
        .to_string()
}

pub(crate) fn confluence_attachment_id(attachment: &ConfluenceAttachment) -> String {
    confluence_attachment_id_with_fallback(attachment, "unknown")
}

pub(crate) fn confluence_attachment_filename(
    attachment: &ConfluenceAttachment,
    fallback_id: &str,
) -> String {
    attachment
        .title
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| fallback_id.to_string())
}

pub(crate) fn confluence_is_image_attachment(
    media_type: Option<&str>,
    filename: &str,
) -> (bool, String) {
    if let Some(media_type) = media_type
        && matches!(
            media_type,
            "image/png" | "image/jpeg" | "image/gif" | "image/webp" | "image/svg+xml" | "image/bmp"
        )
    {
        return (true, media_type.to_string());
    }

    if (media_type.is_none() || confluence_is_ambiguous_mime_type(media_type))
        && let Some(guessed) = confluence_guess_mime_from_filename(filename)
        && guessed.starts_with("image/")
    {
        return (true, guessed.to_string());
    }

    (
        false,
        media_type.unwrap_or("application/octet-stream").to_string(),
    )
}

fn confluence_attachment_id_with_fallback(
    attachment: &ConfluenceAttachment,
    fallback_id: &str,
) -> String {
    attachment
        .id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| fallback_id.to_string())
}

fn confluence_content_type_for_attachment(
    content: &DownloadedContent,
    attachment: &ConfluenceAttachment,
    filename: &str,
) -> String {
    content
        .content_type
        .clone()
        .filter(|content_type| !confluence_is_ambiguous_mime_type(Some(content_type.as_str())))
        .or_else(|| attachment.media_type().map(ToString::to_string))
        .or_else(|| confluence_guess_mime_from_filename(filename).map(ToString::to_string))
        .unwrap_or_else(|| "application/octet-stream".to_string())
}

fn confluence_is_ambiguous_mime_type(media_type: Option<&str>) -> bool {
    matches!(
        media_type,
        Some("application/octet-stream" | "application/binary")
    )
}

fn confluence_guess_mime_from_filename(filename: &str) -> Option<&'static str> {
    let filename = filename.to_ascii_lowercase();
    if filename.ends_with(".png") {
        Some("image/png")
    } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
        Some("image/jpeg")
    } else if filename.ends_with(".gif") {
        Some("image/gif")
    } else if filename.ends_with(".webp") {
        Some("image/webp")
    } else if filename.ends_with(".svg") {
        Some("image/svg+xml")
    } else if filename.ends_with(".bmp") {
        Some("image/bmp")
    } else if filename.ends_with(".txt") {
        Some("text/plain")
    } else if filename.ends_with(".pdf") {
        Some("application/pdf")
    } else {
        None
    }
}
