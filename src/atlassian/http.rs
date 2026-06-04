#![allow(dead_code)]

use std::time::Duration;

use reqwest::{Client, Method, RequestBuilder, Url, header::CONTENT_TYPE};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::atlassian::{auth::AtlassianAuth, error::AtlassianError};

#[derive(Clone, Debug)]
pub struct AtlassianHttpClient {
    base_url: Url,
    client: Client,
    auth: AtlassianAuth,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadedContent {
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

impl AtlassianHttpClient {
    pub fn new(
        base_url: &str,
        auth: AtlassianAuth,
        timeout_seconds: u64,
        ssl_verify: bool,
    ) -> Result<Self, AtlassianError> {
        let base_url = Url::parse(base_url)
            .map_err(|error| AtlassianError::invalid_base_url(error.to_string()))?;
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .danger_accept_invalid_certs(!ssl_verify)
            .build()
            .map_err(AtlassianError::transport)?;

        Ok(Self {
            base_url,
            client,
            auth,
        })
    }

    pub fn get(&self, path: &str) -> Result<RequestBuilder, AtlassianError> {
        self.request(Method::GET, path)
    }

    pub fn get_same_origin_or_relative_url(
        &self,
        value: &str,
        field_name: &'static str,
    ) -> Result<RequestBuilder, AtlassianError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(AtlassianError::invalid_input(format!(
                "{field_name} must not be empty"
            )));
        }

        if let Ok(url) = Url::parse(trimmed) {
            if !same_origin(&self.base_url, &url) {
                return Err(AtlassianError::invalid_input(format!(
                    "{field_name} absolute URL must use the configured Atlassian base origin"
                )));
            }

            return Ok(self.auth.apply(self.client.request(Method::GET, url)));
        }

        self.get(trimmed)
    }

    pub fn post_json<T>(&self, path: &str, body: &T) -> Result<RequestBuilder, AtlassianError>
    where
        T: Serialize + ?Sized,
    {
        Ok(self.request(Method::POST, path)?.json(body))
    }

    pub fn put_json<T>(&self, path: &str, body: &T) -> Result<RequestBuilder, AtlassianError>
    where
        T: Serialize + ?Sized,
    {
        Ok(self.request(Method::PUT, path)?.json(body))
    }

    pub fn delete(&self, path: &str) -> Result<RequestBuilder, AtlassianError> {
        self.request(Method::DELETE, path)
    }

    pub async fn send_json<T>(&self, builder: RequestBuilder) -> Result<T, AtlassianError>
    where
        T: DeserializeOwned,
    {
        let response = builder.send().await.map_err(AtlassianError::transport)?;
        let status = response.status();

        if !status.is_success() {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "failed to read error response".to_string());
            return Err(AtlassianError::http_status(status, message));
        }

        response.json().await.map_err(AtlassianError::json_decode)
    }

    pub async fn send_json_value_or_null(
        &self,
        builder: RequestBuilder,
    ) -> Result<Value, AtlassianError> {
        let response = builder.send().await.map_err(AtlassianError::transport)?;
        let status = response.status();

        if !status.is_success() {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "failed to read error response".to_string());
            return Err(AtlassianError::http_status(status, message));
        }

        let bytes = response.bytes().await.map_err(AtlassianError::transport)?;
        if bytes.is_empty() {
            return Ok(Value::Null);
        }

        serde_json::from_slice(&bytes).map_err(|error| AtlassianError::JsonDecode {
            message: error.to_string(),
        })
    }

    pub async fn send_bytes_limited(
        &self,
        builder: RequestBuilder,
        max_bytes: u64,
    ) -> Result<DownloadedContent, AtlassianError> {
        let response = builder.send().await.map_err(AtlassianError::transport)?;
        let status = response.status();
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string);

        if !status.is_success() {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "failed to read error response".to_string());
            return Err(AtlassianError::http_status(status, message));
        }

        if response
            .content_length()
            .is_some_and(|content_length| content_length > max_bytes)
        {
            return Err(AtlassianError::invalid_input(format!(
                "response body exceeds configured limit of {max_bytes} bytes"
            )));
        }

        let mut response = response;
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(AtlassianError::transport)? {
            if bytes.len() as u64 + chunk.len() as u64 > max_bytes {
                return Err(AtlassianError::invalid_input(format!(
                    "response body exceeds configured limit of {max_bytes} bytes"
                )));
            }
            bytes.extend_from_slice(&chunk);
        }

        Ok(DownloadedContent {
            content_type,
            bytes,
        })
    }

    pub fn join_api_path(&self, path: &str) -> Url {
        let mut url = self.base_url.clone();
        let base_path = url.path().trim_end_matches('/');
        let (path, query) = path
            .trim_start_matches('/')
            .split_once('?')
            .map_or((path.trim_start_matches('/'), None), |(path, query)| {
                (path, Some(query))
            });
        let joined = if base_path.is_empty() || base_path == "/" {
            format!("/{path}")
        } else {
            format!("{base_path}/{path}")
        };

        url.set_path(&joined);
        url.set_query(query);
        url
    }

    fn request(&self, method: Method, path: &str) -> Result<RequestBuilder, AtlassianError> {
        let url = self.join_api_path(path);
        Ok(self.auth.apply(self.client.request(method, url)))
    }
}

fn same_origin(left: &Url, right: &Url) -> bool {
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::atlassian::auth::AtlassianAuth;

    use super::*;

    fn client() -> AtlassianHttpClient {
        AtlassianHttpClient::new(
            "https://jira.example/base/",
            AtlassianAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            },
            75,
            true,
        )
        .unwrap()
    }

    #[test]
    fn joins_api_paths_under_base_url() {
        let client = client();

        assert_eq!(
            client.join_api_path("/rest/api/2/issue/ABC-1").as_str(),
            "https://jira.example/base/rest/api/2/issue/ABC-1"
        );
    }

    #[test]
    fn joins_api_paths_with_query_under_base_url() {
        let client = client();

        assert_eq!(
            client
                .join_api_path("/secure/attachment/1/file.png?token=secret&client=abc")
                .as_str(),
            "https://jira.example/base/secure/attachment/1/file.png?token=secret&client=abc"
        );
    }

    #[test]
    fn same_origin_absolute_get_allows_configured_origin_only() {
        let allowed = client()
            .get_same_origin_or_relative_url(
                "https://jira.example/base/secure/attachment/1/file.png?token=secret",
                "content",
            )
            .unwrap()
            .build()
            .unwrap();
        let blocked = client()
            .get_same_origin_or_relative_url("https://evil.example/file.png", "content")
            .unwrap_err()
            .to_string();

        assert_eq!(
            allowed.url().as_str(),
            "https://jira.example/base/secure/attachment/1/file.png?token=secret"
        );
        assert!(blocked.contains("absolute URL must use the configured Atlassian base origin"));
    }

    #[test]
    fn request_helpers_apply_auth_header() {
        let expected_header = format!("Bearer {}", "test-pat-value");
        let request = client()
            .post_json("/rest/api/2/comment", &json!({ "body": "hello" }))
            .unwrap()
            .build()
            .unwrap();
        let header = request.headers().get(reqwest::header::AUTHORIZATION);

        assert!(header.is_some());
        assert!(
            header
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value == expected_header)
        );
    }
}
