use super::*;

impl ConfluenceClient {
    pub async fn get_attachments(
        &self,
        content_id: &str,
        start: Option<u64>,
        limit: Option<u64>,
        filename: Option<&str>,
        media_type: Option<&str>,
    ) -> Result<ConfluenceAttachmentListResponse, AtlassianError> {
        let content_id = safe_path_segment(content_id, "content_id")?;
        let start = start.unwrap_or(0);
        let limit = attachment_list_limit(limit)?;
        let filename = optional_non_empty_input(filename);
        let media_type = optional_non_empty_input(media_type);
        let mut response: ConfluenceAttachmentListResponse = self
            .get_json(
                &format!("/rest/api/content/{content_id}/child/attachment"),
                vec![
                    ("start".to_string(), start.to_string()),
                    ("limit".to_string(), limit.to_string()),
                    (
                        "expand".to_string(),
                        "metadata,extensions,version".to_string(),
                    ),
                ],
            )
            .await?;

        if filename.is_some() || media_type.is_some() {
            response.results.retain(|attachment| {
                filename
                    .as_deref()
                    .is_none_or(|value| attachment.title.as_deref() == Some(value))
                    && media_type
                        .as_deref()
                        .is_none_or(|value| attachment.media_type() == Some(value))
            });
            response.size = Some(response.results.len() as u64);
        }

        Ok(response)
    }

    pub async fn get_attachment_by_id(
        &self,
        attachment_id: &str,
    ) -> Result<crate::confluence::models::ConfluenceAttachment, AtlassianError> {
        let attachment_id = safe_path_segment(attachment_id, "attachment_id")?;
        self.get_json(
            &format!("/rest/api/content/{attachment_id}"),
            vec![(
                "expand".to_string(),
                "metadata,extensions,version".to_string(),
            )],
        )
        .await
    }

    pub async fn upload_attachment(
        &self,
        content_id: &str,
        file_path: &str,
        comment: Option<&str>,
        minor_edit: bool,
    ) -> Result<ConfluenceAttachment, AtlassianError> {
        let content_id = safe_path_segment(content_id, "content_id")?;
        let file_path = required_non_empty_input(file_path, "file_path")?;
        let path = Path::new(&file_path);
        let filename = path
            .file_name()
            .and_then(|value| value.to_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| AtlassianError::invalid_input("file_path must include a filename"))?
            .to_string();
        let metadata = fs::metadata(path).map_err(|error| {
            AtlassianError::invalid_input(format!(
                "failed to inspect local file `{filename}`: {error}"
            ))
        })?;
        if !metadata.is_file() {
            return Err(AtlassianError::invalid_input(format!(
                "local upload target `{filename}` must be a file"
            )));
        }
        if metadata.len() > DEFAULT_UPLOAD_ATTACHMENT_MAX_BYTES {
            return Err(AtlassianError::invalid_input(format!(
                "local file `{filename}` size {} bytes exceeds configured upload limit of {} bytes",
                metadata.len(),
                DEFAULT_UPLOAD_ATTACHMENT_MAX_BYTES
            )));
        }
        let bytes = fs::read(path).map_err(|error| {
            AtlassianError::invalid_input(format!(
                "failed to read local file `{filename}`: {error}"
            ))
        })?;
        let form = attachment_multipart_form(&filename, bytes, comment, minor_edit)?;
        let value = self
            .http
            .send_json_value_or_null(self.http.put_multipart_with_headers(
                &format!("/rest/api/content/{content_id}/child/attachment"),
                form,
                &[("x-atlassian-token", "nocheck")],
            )?)
            .await?;

        confluence_attachment_from_upload_response(value)
    }

    pub async fn delete_attachment(
        &self,
        attachment_id: &str,
    ) -> Result<serde_json::Value, AtlassianError> {
        let attachment_id = safe_path_segment(attachment_id, "attachment_id")?;
        self.http
            .send_json_value_or_null(
                self.http
                    .delete(&format!("/rest/api/content/{attachment_id}"))?,
            )
            .await
    }
}
