#![allow(dead_code)]

use std::time::Duration;

use reqwest::{Client, Method, RequestBuilder, Url};
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

    pub fn join_api_path(&self, path: &str) -> Url {
        let mut url = self.base_url.clone();
        let base_path = url.path().trim_end_matches('/');
        let path = path.trim_start_matches('/');
        let joined = if base_path.is_empty() || base_path == "/" {
            format!("/{path}")
        } else {
            format!("{base_path}/{path}")
        };

        url.set_path(&joined);
        url
    }

    fn request(&self, method: Method, path: &str) -> Result<RequestBuilder, AtlassianError> {
        let url = self.join_api_path(path);
        Ok(self.auth.apply(self.client.request(method, url)))
    }
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
