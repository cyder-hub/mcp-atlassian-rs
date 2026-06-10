#![allow(dead_code)]

use std::time::Duration;

use reqwest::{
    Client, ClientBuilder, Method, NoProxy, Proxy, RequestBuilder, Url, header::CONTENT_TYPE,
    multipart::Form, redirect::Policy,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::atlassian::{
    auth::UpstreamAuth, custom_headers::CustomHeaders, error::UpstreamError,
    mtls::ClientTlsIdentityConfig, proxy::ProxyConfig, redaction::redact_text,
    security::MAX_SAME_ORIGIN_REDIRECTS,
};

#[derive(Clone, Debug)]
pub struct UpstreamHttpClient {
    base_url: Url,
    client: Client,
    auth: UpstreamAuth,
    proxy: ProxyConfig,
    custom_headers: CustomHeaders,
    mtls: Option<ClientTlsIdentityConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadedContent {
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

impl UpstreamHttpClient {
    pub fn new(
        base_url: &str,
        auth: UpstreamAuth,
        timeout_seconds: u64,
        ssl_verify: bool,
    ) -> Result<Self, UpstreamError> {
        Self::new_with_proxy(
            base_url,
            auth,
            timeout_seconds,
            ssl_verify,
            ProxyConfig::default(),
        )
    }

    pub fn new_with_proxy(
        base_url: &str,
        auth: UpstreamAuth,
        timeout_seconds: u64,
        ssl_verify: bool,
        proxy: ProxyConfig,
    ) -> Result<Self, UpstreamError> {
        Self::new_with_proxy_and_headers(
            base_url,
            auth,
            timeout_seconds,
            ssl_verify,
            proxy,
            CustomHeaders::default(),
        )
    }

    pub fn new_with_proxy_and_headers(
        base_url: &str,
        auth: UpstreamAuth,
        timeout_seconds: u64,
        ssl_verify: bool,
        proxy: ProxyConfig,
        custom_headers: CustomHeaders,
    ) -> Result<Self, UpstreamError> {
        Self::new_with_proxy_headers_and_mtls(
            base_url,
            auth,
            timeout_seconds,
            ssl_verify,
            proxy,
            custom_headers,
            None,
        )
    }

    pub fn new_with_proxy_headers_and_mtls(
        base_url: &str,
        auth: UpstreamAuth,
        timeout_seconds: u64,
        ssl_verify: bool,
        proxy: ProxyConfig,
        custom_headers: CustomHeaders,
        mtls: Option<ClientTlsIdentityConfig>,
    ) -> Result<Self, UpstreamError> {
        let base_url = Url::parse(base_url)
            .map_err(|error| UpstreamError::invalid_base_url(error.to_string()))?;
        let builder = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .danger_accept_invalid_certs(!ssl_verify)
            .redirect(same_origin_redirect_policy())
            .no_proxy();
        let builder = apply_proxy_config(builder, &proxy)?;
        let client = apply_mtls_identity(builder, mtls.as_ref())?
            .build()
            .map_err(UpstreamError::transport)?;

        Ok(Self {
            base_url,
            client,
            auth,
            proxy,
            custom_headers,
            mtls,
        })
    }

    pub fn get(&self, path: &str) -> Result<RequestBuilder, UpstreamError> {
        self.request(Method::GET, path)
    }

    pub fn get_same_origin_or_relative_url(
        &self,
        value: &str,
        field_name: &'static str,
    ) -> Result<RequestBuilder, UpstreamError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(UpstreamError::invalid_input(format!(
                "{field_name} must not be empty"
            )));
        }

        if let Ok(url) = Url::parse(trimmed) {
            if !same_origin(&self.base_url, &url) {
                return Err(UpstreamError::invalid_input(format!(
                    "{field_name} absolute URL must use the configured upstream base origin"
                )));
            }

            return Ok(self.authorized_request(Method::GET, url));
        }

        self.get(trimmed)
    }

    pub fn post_json<T>(&self, path: &str, body: &T) -> Result<RequestBuilder, UpstreamError>
    where
        T: Serialize + ?Sized,
    {
        Ok(self.request(Method::POST, path)?.json(body))
    }

    pub fn put_json<T>(&self, path: &str, body: &T) -> Result<RequestBuilder, UpstreamError>
    where
        T: Serialize + ?Sized,
    {
        Ok(self.request(Method::PUT, path)?.json(body))
    }

    pub fn put_body_with_headers(
        &self,
        path: &str,
        body: Vec<u8>,
        content_type: &str,
        headers: &[(&'static str, &'static str)],
    ) -> Result<RequestBuilder, UpstreamError> {
        let mut builder = self
            .request(Method::PUT, path)?
            .header(CONTENT_TYPE, content_type)
            .body(body);
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        Ok(builder)
    }

    pub fn put_multipart_with_headers(
        &self,
        path: &str,
        form: Form,
        headers: &[(&'static str, &'static str)],
    ) -> Result<RequestBuilder, UpstreamError> {
        let mut builder = self.request(Method::PUT, path)?.multipart(form);
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        Ok(builder)
    }

    pub fn delete(&self, path: &str) -> Result<RequestBuilder, UpstreamError> {
        self.request(Method::DELETE, path)
    }

    pub async fn send_json<T>(&self, builder: RequestBuilder) -> Result<T, UpstreamError>
    where
        T: DeserializeOwned,
    {
        let request_context = request_context(&builder);
        let response = builder.send().await.map_err(UpstreamError::transport)?;
        let status = response.status();

        if !status.is_success() {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "failed to read error response".to_string());
            return Err(UpstreamError::http_status(status, message));
        }

        let bytes = response.bytes().await.map_err(UpstreamError::transport)?;
        serde_json::from_slice(&bytes)
            .map_err(|error| UpstreamError::json_decode_body(error, request_context.as_deref()))
    }

    pub async fn send_json_value_or_null(
        &self,
        builder: RequestBuilder,
    ) -> Result<Value, UpstreamError> {
        let request_context = request_context(&builder);
        let response = builder.send().await.map_err(UpstreamError::transport)?;
        let status = response.status();

        if !status.is_success() {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "failed to read error response".to_string());
            return Err(UpstreamError::http_status(status, message));
        }

        let bytes = response.bytes().await.map_err(UpstreamError::transport)?;
        if bytes.is_empty() {
            return Ok(Value::Null);
        }

        serde_json::from_slice(&bytes)
            .map_err(|error| UpstreamError::json_decode_body(error, request_context.as_deref()))
    }

    pub async fn send_bytes_limited(
        &self,
        builder: RequestBuilder,
        max_bytes: u64,
    ) -> Result<DownloadedContent, UpstreamError> {
        let response = builder.send().await.map_err(UpstreamError::transport)?;
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
            return Err(UpstreamError::http_status(status, message));
        }

        if response
            .content_length()
            .is_some_and(|content_length| content_length > max_bytes)
        {
            return Err(UpstreamError::invalid_input(format!(
                "response body exceeds configured limit of {max_bytes} bytes"
            )));
        }

        let mut response = response;
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(UpstreamError::transport)? {
            if bytes.len() as u64 + chunk.len() as u64 > max_bytes {
                return Err(UpstreamError::invalid_input(format!(
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

    fn request(&self, method: Method, path: &str) -> Result<RequestBuilder, UpstreamError> {
        let url = self.join_api_path(path);
        Ok(self.authorized_request(method, url))
    }

    fn authorized_request(&self, method: Method, url: Url) -> RequestBuilder {
        self.auth
            .apply(self.apply_custom_headers(self.client.request(method, url)))
    }

    fn apply_custom_headers(&self, mut builder: RequestBuilder) -> RequestBuilder {
        for (name, value) in self.custom_headers.iter() {
            builder = builder.header(name, value);
        }
        builder
    }
}

fn apply_proxy_config(
    mut builder: ClientBuilder,
    proxy: &ProxyConfig,
) -> Result<ClientBuilder, UpstreamError> {
    let no_proxy = proxy.no_proxy.as_deref().and_then(NoProxy::from_string);

    if let Some(http_proxy) = proxy.http_proxy.as_deref() {
        builder = builder.proxy(
            Proxy::http(http_proxy)
                .map_err(UpstreamError::transport)?
                .no_proxy(no_proxy.clone()),
        );
    }
    if let Some(https_proxy) = proxy.https_proxy.as_deref() {
        builder = builder.proxy(
            Proxy::https(https_proxy)
                .map_err(UpstreamError::transport)?
                .no_proxy(no_proxy),
        );
    }

    Ok(builder)
}

fn apply_mtls_identity(
    builder: ClientBuilder,
    mtls: Option<&ClientTlsIdentityConfig>,
) -> Result<ClientBuilder, UpstreamError> {
    match mtls {
        Some(mtls) => Ok(builder.identity(mtls.load_identity()?)),
        None => Ok(builder),
    }
}

fn same_origin(left: &Url, right: &Url) -> bool {
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

fn same_origin_redirect_policy() -> Policy {
    Policy::custom(|attempt| {
        if attempt.previous().len() > MAX_SAME_ORIGIN_REDIRECTS {
            return attempt.error("too many same-origin redirects");
        }

        let Some(previous) = attempt.previous().last() else {
            return attempt.error("redirect chain is missing previous URL");
        };

        if matches!(attempt.url().scheme(), "http" | "https")
            && same_origin(previous, attempt.url())
        {
            attempt.follow()
        } else {
            attempt.error("blocked unsafe redirect")
        }
    })
}

fn request_context(builder: &RequestBuilder) -> Option<String> {
    let request = builder.try_clone()?.build().ok()?;
    Some(format!(
        "{} {}",
        request.method(),
        redacted_path_and_query(request.url())
    ))
}

fn redacted_path_and_query(url: &Url) -> String {
    let mut value = url.path().to_string();
    if let Some(query) = url.query() {
        value.push('?');
        value.push_str(query);
    }
    redact_text(&value)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        process::Command,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };

    use axum::{
        Json, Router,
        http::{StatusCode, header::LOCATION},
        response::{IntoResponse, Redirect},
        routing::{any, get},
    };
    use serde_json::json;

    use crate::atlassian::{
        auth::UpstreamAuth, compat::ENV_JIRA_CUSTOM_HEADERS, custom_headers::CustomHeaders,
        mtls::ClientTlsIdentityConfig, proxy::ProxyConfig,
    };

    use super::*;

    fn client() -> UpstreamHttpClient {
        UpstreamHttpClient::new(
            "https://jira.example/base/",
            UpstreamAuth::Pat {
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
    fn joins_api_paths_under_jira_cloud_byot_gateway_base_url() {
        let client = UpstreamHttpClient::new(
            "https://api.atlassian.com/ex/jira/cloud-123",
            UpstreamAuth::OAuthAccessToken {
                access_token: "test-access-token".to_string(),
            },
            75,
            true,
        )
        .unwrap();

        assert_eq!(
            client.join_api_path("/rest/api/3/issue/ABC-1").as_str(),
            "https://api.atlassian.com/ex/jira/cloud-123/rest/api/3/issue/ABC-1"
        );
    }

    #[test]
    fn joins_api_paths_under_confluence_cloud_byot_gateway_base_url() {
        let client = UpstreamHttpClient::new(
            "https://api.atlassian.com/ex/confluence/cloud-123/wiki",
            UpstreamAuth::OAuthAccessToken {
                access_token: "test-access-token".to_string(),
            },
            75,
            true,
        )
        .unwrap();

        assert_eq!(
            client.join_api_path("/rest/api/content/123").as_str(),
            "https://api.atlassian.com/ex/confluence/cloud-123/wiki/rest/api/content/123"
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
        assert!(blocked.contains("absolute URL must use the configured upstream base origin"));
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

    #[test]
    fn request_helpers_apply_custom_headers_without_overwriting_auth() {
        let expected_header = format!("Bearer {}", "test-pat-value");
        let client = UpstreamHttpClient::new_with_proxy_and_headers(
            "https://jira.example/base/",
            UpstreamAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            },
            75,
            true,
            ProxyConfig::default(),
            custom_headers("X-Test=seven,X-Trace=abc=def"),
        )
        .unwrap();
        let request = client
            .get("/rest/api/2/issue/ABC-1")
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(
            request
                .headers()
                .get("x-test")
                .and_then(|value| value.to_str().ok()),
            Some("seven")
        );
        assert_eq!(
            request
                .headers()
                .get("x-trace")
                .and_then(|value| value.to_str().ok()),
            Some("abc=def")
        );
        assert_eq!(
            request
                .headers()
                .get(reqwest::header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok()),
            Some(expected_header.as_str())
        );
    }

    #[test]
    fn body_helpers_keep_content_type_and_apply_custom_headers() {
        let client = UpstreamHttpClient::new_with_proxy_and_headers(
            "https://jira.example/base/",
            UpstreamAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            },
            75,
            true,
            ProxyConfig::default(),
            custom_headers("X-Test=seven"),
        )
        .unwrap();
        let request = client
            .put_body_with_headers(
                "/rest/api/2/issue/ABC-1",
                Vec::new(),
                "application/octet-stream",
                &[("X-Upload", "enabled")],
            )
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(
            request
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("application/octet-stream")
        );
        assert_eq!(
            request
                .headers()
                .get("x-test")
                .and_then(|value| value.to_str().ok()),
            Some("seven")
        );
        assert_eq!(
            request
                .headers()
                .get("x-upload")
                .and_then(|value| value.to_str().ok()),
            Some("enabled")
        );
    }

    #[test]
    fn client_debug_redacts_proxy_credentials() {
        let client = UpstreamHttpClient::new_with_proxy(
            "https://jira.example/base/",
            UpstreamAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            },
            75,
            true,
            ProxyConfig {
                http_proxy: Some("http://user:secret@proxy.example:8080".to_string()),
                https_proxy: None,
                no_proxy: Some("jira.example".to_string()),
            },
        )
        .unwrap();
        let debug = format!("{client:?}");

        assert!(!debug.contains("user:secret"));
        assert!(!debug.contains("secret"));
        assert!(debug.contains("<redacted>"));
    }

    #[test]
    fn client_builder_accepts_mtls_identity_without_debug_key_leakage() {
        let mtls = generate_temp_mtls_identity("client-builder");
        let client = UpstreamHttpClient::new_with_proxy_headers_and_mtls(
            "https://jira.example/base/",
            UpstreamAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            },
            75,
            true,
            ProxyConfig::default(),
            CustomHeaders::default(),
            Some(mtls),
        )
        .unwrap();
        let debug = format!("{client:?}");

        assert!(!debug.contains("BEGIN RSA PRIVATE KEY"));
        assert!(!debug.contains("END RSA PRIVATE KEY"));
    }

    #[test]
    fn request_context_redacts_sensitive_query_values() {
        let builder = client()
            .get("/rest/api/2/issue/ABC-1?token=secret&client=abc")
            .unwrap();
        let context = request_context(&builder).unwrap();

        assert_eq!(
            context,
            "GET /base/rest/api/2/issue/ABC-1?token=<redacted>&client=abc"
        );
        assert!(!context.contains("secret"));
    }

    #[tokio::test]
    async fn send_json_follows_same_origin_redirects() {
        let app = Router::new()
            .route(
                "/redirect-safe",
                get(|| async { Redirect::temporary("/final") }),
            )
            .route("/final", get(|| async { Json(json!({ "ok": true })) }));
        let base_url = serve(app).await;
        let client = UpstreamHttpClient::new(
            &base_url,
            UpstreamAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            },
            75,
            true,
        )
        .unwrap();

        let value: Value = client
            .send_json(client.get("/redirect-safe").unwrap())
            .await
            .unwrap();

        assert_eq!(value, json!({ "ok": true }));
    }

    #[tokio::test]
    async fn send_json_blocks_cross_origin_redirects_without_contacting_target() {
        let target_hits = Arc::new(AtomicUsize::new(0));
        let target_hits_for_route = target_hits.clone();
        let target = serve(Router::new().route(
            "/target",
            get(move || {
                let target_hits = target_hits_for_route.clone();
                async move {
                    target_hits.fetch_add(1, Ordering::SeqCst);
                    Json(json!({ "reached": true }))
                }
            }),
        ))
        .await;
        let location = format!("{target}/target?token=secret");
        let redirector = serve(Router::new().route(
            "/redirect-unsafe",
            get(move || {
                let location = location.clone();
                async move { (StatusCode::FOUND, [(LOCATION, location)]).into_response() }
            }),
        ))
        .await;
        let client = UpstreamHttpClient::new(
            &redirector,
            UpstreamAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            },
            75,
            true,
        )
        .unwrap();

        let error = client
            .send_json::<Value>(client.get("/redirect-unsafe").unwrap())
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains("redirect"));
        assert!(!error.contains("token=secret"));
        assert_eq!(target_hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn send_json_uses_http_proxy_and_honors_no_proxy() {
        let proxy_hits = Arc::new(AtomicUsize::new(0));
        let target_hits = Arc::new(AtomicUsize::new(0));
        let proxy = counted_json_server("proxy", proxy_hits.clone()).await;
        let target = counted_json_server("target", target_hits.clone()).await;
        let proxied = UpstreamHttpClient::new_with_proxy(
            &target,
            UpstreamAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            },
            75,
            true,
            ProxyConfig {
                http_proxy: Some(proxy.clone()),
                https_proxy: None,
                no_proxy: None,
            },
        )
        .unwrap();

        let value: Value = proxied
            .send_json(proxied.get("/via-proxy").unwrap())
            .await
            .unwrap();

        assert_eq!(value["server"], "proxy");
        assert_eq!(proxy_hits.load(Ordering::SeqCst), 1);
        assert_eq!(target_hits.load(Ordering::SeqCst), 0);

        let bypassed = UpstreamHttpClient::new_with_proxy(
            &target,
            UpstreamAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            },
            75,
            true,
            ProxyConfig {
                http_proxy: Some(proxy),
                https_proxy: None,
                no_proxy: Some("127.0.0.1".to_string()),
            },
        )
        .unwrap();
        let value: Value = bypassed
            .send_json(bypassed.get("/direct").unwrap())
            .await
            .unwrap();

        assert_eq!(value["server"], "target");
        assert_eq!(proxy_hits.load(Ordering::SeqCst), 1);
        assert_eq!(target_hits.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn send_json_enforces_same_origin_redirect_limit() {
        let app = Router::new().route(
            "/redirect/{step}",
            get(
                |axum::extract::Path(step): axum::extract::Path<u8>| async move {
                    if step < 4 {
                        (
                            StatusCode::FOUND,
                            [(LOCATION, format!("/redirect/{}", step + 1))],
                        )
                            .into_response()
                    } else {
                        Json(json!({ "ok": true })).into_response()
                    }
                },
            ),
        );
        let base_url = serve(app).await;
        let client = UpstreamHttpClient::new(
            &base_url,
            UpstreamAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            },
            75,
            true,
        )
        .unwrap();

        let error = client
            .send_json::<Value>(client.get("/redirect/0").unwrap())
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains("redirect"));
    }

    async fn counted_json_server(name: &'static str, hits: Arc<AtomicUsize>) -> String {
        let app = Router::new().fallback(any(move || {
            let hits = hits.clone();
            async move {
                hits.fetch_add(1, Ordering::SeqCst);
                Json(json!({ "server": name }))
            }
        }));
        serve(app).await
    }

    async fn serve(app: Router) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        format!("http://{address}")
    }

    fn custom_headers(value: &str) -> CustomHeaders {
        let vars = BTreeMap::from([(ENV_JIRA_CUSTOM_HEADERS.to_string(), value.to_string())]);
        CustomHeaders::from_var_provider(
            &mut |key| vars.get(key).cloned().ok_or(()),
            ENV_JIRA_CUSTOM_HEADERS,
        )
        .unwrap()
    }

    fn generate_temp_mtls_identity(name: &str) -> ClientTlsIdentityConfig {
        let base =
            std::env::temp_dir().join(format!("mcp-atlassian-rs-{name}-{}", std::process::id()));
        fs::create_dir_all(&base).unwrap();
        let cert_path = base.join("client.crt");
        let key_path = base.join("client.key");
        let output = Command::new("openssl")
            .args(["req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout"])
            .arg(&key_path)
            .arg("-out")
            .arg(&cert_path)
            .args(["-subj", "/CN=localhost", "-days", "1"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "openssl mTLS fixture generation failed"
        );

        ClientTlsIdentityConfig {
            cert_path,
            key_path,
        }
    }
}
