use super::*;

impl JiraClient {
    pub async fn get_issue_attachments(
        &self,
        issue_key: String,
    ) -> Result<Vec<JiraAttachment>, UpstreamError> {
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
            .map_err(|error| UpstreamError::unexpected_shape(error.to_string()))
    }

    pub async fn fetch_attachment_content(
        &self,
        content_path: &str,
        max_bytes: u64,
    ) -> Result<DownloadedContent, UpstreamError> {
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
    ) -> Result<Value, UpstreamError> {
        if options.include_content && options.max_bytes == 0 {
            return Err(UpstreamError::invalid_input("max_bytes must be positive"));
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
}
