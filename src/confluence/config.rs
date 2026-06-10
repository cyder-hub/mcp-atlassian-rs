use std::collections::BTreeSet;

use reqwest::Url;

use crate::{
    atlassian::compat::{
        ENV_CONFLUENCE_CLIENT_CERT, ENV_CONFLUENCE_CLIENT_KEY, ENV_CONFLUENCE_CUSTOM_HEADERS,
        ENV_CONFLUENCE_HTTP_PROXY, ENV_CONFLUENCE_HTTPS_PROXY, ENV_CONFLUENCE_NO_PROXY,
        ENV_CONFLUENCE_OAUTH_ACCESS_TOKEN,
    },
    error::ConfigError,
    upstream::{
        auth::UpstreamAuth,
        config::{
            ParsedUpstreamServiceConfig, UpstreamServiceConfigSpec, parse_upstream_service_config,
        },
        custom_headers::CustomHeaders,
        mtls::ClientTlsIdentityConfig,
        proxy::ProxyConfig,
    },
};

pub const ENV_CONFLUENCE_URL: &str = "CONFLUENCE_URL";
pub const ENV_CONFLUENCE_USERNAME: &str = "CONFLUENCE_USERNAME";
pub const ENV_CONFLUENCE_API_TOKEN: &str = "CONFLUENCE_API_TOKEN";
pub const ENV_CONFLUENCE_PASSWORD: &str = "CONFLUENCE_PASSWORD";
pub const ENV_CONFLUENCE_PERSONAL_TOKEN: &str = "CONFLUENCE_PERSONAL_TOKEN";
pub const ENV_CONFLUENCE_SSL_VERIFY: &str = "CONFLUENCE_SSL_VERIFY";
pub const ENV_CONFLUENCE_SPACES_FILTER: &str = "CONFLUENCE_SPACES_FILTER";
pub const ENV_CONFLUENCE_TIMEOUT: &str = "CONFLUENCE_TIMEOUT";

pub const DEFAULT_CONFLUENCE_TIMEOUT_SECONDS: u64 = 75;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceConfig {
    pub base_url: String,
    pub deployment: ConfluenceDeployment,
    pub auth: UpstreamAuth,
    pub oauth_cloud_id: Option<String>,
    pub ssl_verify: bool,
    pub proxy: ProxyConfig,
    pub custom_headers: CustomHeaders,
    pub mtls: Option<ClientTlsIdentityConfig>,
    pub spaces_filter: BTreeSet<String>,
    pub timeout_seconds: u64,
}

impl ConfluenceConfig {
    pub fn from_var_provider<F, E>(get_var: &mut F) -> Result<Option<Self>, ConfigError>
    where
        F: FnMut(&str) -> Result<String, E>,
    {
        let Some(parsed) = parse_upstream_service_config(get_var, &confluence_config_spec())?
        else {
            return Ok(None);
        };

        Ok(Some(Self::from_parsed(
            parsed,
            parse_spaces_filter(optional_var(get_var, ENV_CONFLUENCE_SPACES_FILTER)),
        )))
    }

    pub fn is_auth_configured(&self) -> bool {
        matches!(
            self.auth,
            UpstreamAuth::Basic { .. }
                | UpstreamAuth::Pat { .. }
                | UpstreamAuth::OAuthAccessToken { .. }
        )
    }

    pub fn with_auth_override(&self, auth: UpstreamAuth, oauth_cloud_id: Option<String>) -> Self {
        let mut config = self.clone();
        let effective_cloud_id = if matches!(auth, UpstreamAuth::OAuthAccessToken { .. })
            && self.deployment == ConfluenceDeployment::Cloud
        {
            oauth_cloud_id
        } else {
            None
        };
        if let Some(cloud_id) = effective_cloud_id.as_ref() {
            config.base_url = cloud_oauth_api_base_url(cloud_id);
        }
        config.auth = auth;
        config.oauth_cloud_id = effective_cloud_id;
        config
    }

    fn from_parsed(
        parsed: ParsedUpstreamServiceConfig<ConfluenceDeployment>,
        spaces_filter: BTreeSet<String>,
    ) -> Self {
        Self {
            base_url: parsed.base_url,
            deployment: parsed.deployment,
            auth: parsed.auth,
            oauth_cloud_id: parsed.oauth_cloud_id,
            ssl_verify: parsed.ssl_verify,
            proxy: parsed.proxy,
            custom_headers: parsed.custom_headers,
            mtls: parsed.mtls,
            spaces_filter,
            timeout_seconds: parsed.timeout_seconds,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfluenceDeployment {
    Cloud,
    ServerDataCenter,
}

impl ConfluenceDeployment {
    fn from_base_url(url: &Url) -> Self {
        if url
            .host_str()
            .is_some_and(|host| host.to_ascii_lowercase().ends_with(".atlassian.net"))
        {
            Self::Cloud
        } else {
            Self::ServerDataCenter
        }
    }
}

fn optional_var<F, E>(get_var: &mut F, key: &'static str) -> Option<String>
where
    F: FnMut(&str) -> Result<String, E>,
{
    get_var(key).ok().and_then(non_empty_trimmed)
}

fn non_empty_trimmed(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn cloud_oauth_api_base_url(cloud_id: &str) -> String {
    let mut url = Url::parse("https://api.atlassian.com").expect("static URL is valid");
    url.path_segments_mut()
        .expect("static URL supports path segments")
        .extend(["ex", "confluence", cloud_id, "wiki"]);
    url.to_string().trim_end_matches('/').to_string()
}

fn parse_spaces_filter(value: Option<String>) -> BTreeSet<String> {
    value
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn confluence_config_spec() -> UpstreamServiceConfigSpec<ConfluenceDeployment> {
    UpstreamServiceConfigSpec {
        url_variable: ENV_CONFLUENCE_URL,
        username_variable: ENV_CONFLUENCE_USERNAME,
        api_token_variable: ENV_CONFLUENCE_API_TOKEN,
        password_variable: ENV_CONFLUENCE_PASSWORD,
        personal_token_variable: ENV_CONFLUENCE_PERSONAL_TOKEN,
        oauth_access_token_variable: ENV_CONFLUENCE_OAUTH_ACCESS_TOKEN,
        ssl_verify_variable: ENV_CONFLUENCE_SSL_VERIFY,
        timeout_variable: ENV_CONFLUENCE_TIMEOUT,
        http_proxy_variable: ENV_CONFLUENCE_HTTP_PROXY,
        https_proxy_variable: ENV_CONFLUENCE_HTTPS_PROXY,
        no_proxy_variable: ENV_CONFLUENCE_NO_PROXY,
        custom_headers_variable: ENV_CONFLUENCE_CUSTOM_HEADERS,
        client_cert_variable: ENV_CONFLUENCE_CLIENT_CERT,
        client_key_variable: ENV_CONFLUENCE_CLIENT_KEY,
        default_timeout_seconds: DEFAULT_CONFLUENCE_TIMEOUT_SECONDS,
        cloud_deployment: ConfluenceDeployment::Cloud,
        deployment_from_url: ConfluenceDeployment::from_base_url,
        cloud_oauth_api_base_url,
        missing_url_error: |credential_variables| ConfigError::MissingConfluenceUrl {
            credential_variables,
        },
        invalid_url_error: |variable| ConfigError::InvalidConfluenceUrl { variable },
        missing_cloud_credentials_error: |missing_variables| {
            ConfigError::MissingConfluenceCloudCredentials { missing_variables }
        },
        missing_personal_token_error: |variable| ConfigError::MissingConfluencePersonalToken {
            variable,
        },
        missing_oauth_cloud_id_error: |access_token_variables, cloud_id_variable| {
            ConfigError::MissingConfluenceOAuthCloudId {
                access_token_variables,
                cloud_id_variable,
            }
        },
        invalid_timeout_error: |variable, value| ConfigError::InvalidConfluenceTimeout {
            variable,
            value,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::atlassian::compat::{
        ENV_ATLASSIAN_API_TOKEN, ENV_ATLASSIAN_OAUTH_ACCESS_TOKEN, ENV_ATLASSIAN_OAUTH_CLOUD_ID,
        ENV_ATLASSIAN_PASSWORD, ENV_ATLASSIAN_PERSONAL_TOKEN, ENV_ATLASSIAN_USERNAME,
        ENV_HTTP_PROXY, ENV_HTTPS_PROXY, ENV_NO_PROXY,
    };

    use super::*;

    fn config_from_pairs(pairs: &[(&str, &str)]) -> Result<Option<ConfluenceConfig>, ConfigError> {
        let vars: BTreeMap<String, String> = pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();

        ConfluenceConfig::from_var_provider(&mut |key| vars.get(key).cloned().ok_or(()))
    }

    #[test]
    fn confluence_config_is_disabled_without_url_or_credentials() {
        assert_eq!(config_from_pairs(&[]).unwrap(), None);
    }

    #[test]
    fn credentials_without_url_are_rejected_without_secret_leakage() {
        let error =
            config_from_pairs(&[(ENV_CONFLUENCE_PERSONAL_TOKEN, "test-pat-value")]).unwrap_err();

        assert_eq!(
            error,
            ConfigError::MissingConfluenceUrl {
                credential_variables: vec![ENV_CONFLUENCE_PERSONAL_TOKEN],
            }
        );
        assert!(!error.to_string().contains("test-pat-value"));
    }

    #[test]
    fn confluence_password_without_url_is_rejected() {
        let error = config_from_pairs(&[(ENV_CONFLUENCE_PASSWORD, "test-password")]).unwrap_err();

        assert_eq!(
            error,
            ConfigError::MissingConfluenceUrl {
                credential_variables: vec![ENV_CONFLUENCE_PASSWORD],
            }
        );
        assert!(!error.to_string().contains("test-password"));
    }

    #[test]
    fn cloud_config_requires_username_and_api_token() {
        let error =
            config_from_pairs(&[(ENV_CONFLUENCE_URL, "https://example.atlassian.net/wiki")])
                .unwrap_err();

        assert_eq!(
            error,
            ConfigError::MissingConfluenceCloudCredentials {
                missing_variables: vec![ENV_CONFLUENCE_USERNAME, ENV_CONFLUENCE_API_TOKEN],
            }
        );
    }

    #[test]
    fn cloud_config_builds_basic_auth_and_preserves_wiki_base_path() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, " https://example.atlassian.net/wiki/ "),
            (ENV_CONFLUENCE_USERNAME, "user@example.com"),
            (ENV_CONFLUENCE_API_TOKEN, "test-api-token"),
            (ENV_CONFLUENCE_SSL_VERIFY, "off"),
            (ENV_CONFLUENCE_SPACES_FILTER, " ENG, DOCS,ENG "),
            (ENV_CONFLUENCE_TIMEOUT, "30"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(config.base_url, "https://example.atlassian.net/wiki");
        assert_eq!(config.deployment, ConfluenceDeployment::Cloud);
        assert_eq!(
            config.auth,
            UpstreamAuth::Basic {
                username: "user@example.com".to_string(),
                api_token: "test-api-token".to_string(),
            }
        );
        assert_eq!(config.oauth_cloud_id, None);
        assert!(!config.ssl_verify);
        assert_eq!(
            config.spaces_filter,
            BTreeSet::from(["DOCS".to_string(), "ENG".to_string()])
        );
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.is_auth_configured());
    }

    #[test]
    fn cloud_config_does_not_accept_password_in_place_of_api_token() {
        let error = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://example.atlassian.net/wiki"),
            (ENV_CONFLUENCE_USERNAME, "user@example.com"),
            (ENV_CONFLUENCE_PASSWORD, "test-password"),
        ])
        .unwrap_err();

        assert_eq!(
            error,
            ConfigError::MissingConfluenceCloudCredentials {
                missing_variables: vec![ENV_CONFLUENCE_API_TOKEN],
            }
        );
        assert!(!error.to_string().contains("test-password"));
    }

    #[test]
    fn server_config_requires_pat() {
        let error =
            config_from_pairs(&[(ENV_CONFLUENCE_URL, "https://confluence.example")]).unwrap_err();

        assert_eq!(
            error,
            ConfigError::MissingConfluencePersonalToken {
                variable: ENV_CONFLUENCE_PERSONAL_TOKEN,
            }
        );
    }

    #[test]
    fn server_config_builds_pat_auth_with_defaults() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://confluence.example"),
            (ENV_CONFLUENCE_PERSONAL_TOKEN, "test-pat-value"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(config.base_url, "https://confluence.example");
        assert_eq!(config.deployment, ConfluenceDeployment::ServerDataCenter);
        assert_eq!(
            config.auth,
            UpstreamAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            }
        );
        assert_eq!(config.oauth_cloud_id, None);
        assert!(config.ssl_verify);
        assert!(config.proxy.is_empty());
        assert!(config.spaces_filter.is_empty());
        assert_eq!(config.timeout_seconds, DEFAULT_CONFLUENCE_TIMEOUT_SECONDS);
        assert!(config.is_auth_configured());
    }

    #[test]
    fn server_config_builds_basic_auth_from_password() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://confluence.example"),
            (ENV_CONFLUENCE_USERNAME, "confluence-user"),
            (ENV_CONFLUENCE_PASSWORD, "test-password"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(config.base_url, "https://confluence.example");
        assert_eq!(config.deployment, ConfluenceDeployment::ServerDataCenter);
        assert_eq!(
            config.auth,
            UpstreamAuth::Basic {
                username: "confluence-user".to_string(),
                api_token: "test-password".to_string(),
            }
        );
        assert_eq!(config.oauth_cloud_id, None);
        assert!(config.is_auth_configured());
        assert!(!format!("{:?}", config.auth).contains("test-password"));
    }

    #[test]
    fn server_config_prefers_password_over_api_token_basic_auth() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://confluence.example"),
            (ENV_CONFLUENCE_USERNAME, "confluence-user"),
            (ENV_CONFLUENCE_API_TOKEN, "legacy-api-token"),
            (ENV_CONFLUENCE_PASSWORD, "test-password"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.auth,
            UpstreamAuth::Basic {
                username: "confluence-user".to_string(),
                api_token: "test-password".to_string(),
            }
        );
    }

    #[test]
    fn server_config_prefers_service_api_token_over_shared_password() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://confluence.example"),
            (ENV_CONFLUENCE_USERNAME, "confluence-user"),
            (ENV_CONFLUENCE_API_TOKEN, "service-api-token"),
            (ENV_ATLASSIAN_PASSWORD, "global-password"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.auth,
            UpstreamAuth::Basic {
                username: "confluence-user".to_string(),
                api_token: "service-api-token".to_string(),
            }
        );
    }

    #[test]
    fn atlassian_basic_fallbacks_build_confluence_config() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://example.atlassian.net/wiki"),
            (ENV_ATLASSIAN_USERNAME, "user@example.com"),
            (ENV_ATLASSIAN_API_TOKEN, "global-api-token"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.auth,
            UpstreamAuth::Basic {
                username: "user@example.com".to_string(),
                api_token: "global-api-token".to_string(),
            }
        );
    }

    #[test]
    fn atlassian_password_fallback_builds_confluence_server_config() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://confluence.example"),
            (ENV_ATLASSIAN_USERNAME, "global-user"),
            (ENV_ATLASSIAN_PASSWORD, "global-password"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.auth,
            UpstreamAuth::Basic {
                username: "global-user".to_string(),
                api_token: "global-password".to_string(),
            }
        );
    }

    #[test]
    fn confluence_specific_pat_overrides_atlassian_fallback() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://confluence.example"),
            (ENV_ATLASSIAN_PERSONAL_TOKEN, "global-pat-value"),
            (ENV_CONFLUENCE_PERSONAL_TOKEN, "confluence-pat-value"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.auth,
            UpstreamAuth::Pat {
                personal_token: "confluence-pat-value".to_string(),
            }
        );
    }

    #[test]
    fn proxy_config_uses_global_fallback_values() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://confluence.example"),
            (ENV_CONFLUENCE_PERSONAL_TOKEN, "test-pat-value"),
            (ENV_HTTP_PROXY, "http://global-proxy.example:8080"),
            (ENV_HTTPS_PROXY, "http://global-secure-proxy.example:8080"),
            (ENV_NO_PROXY, " confluence.example,localhost "),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.proxy.http_proxy.as_deref(),
            Some("http://global-proxy.example:8080")
        );
        assert_eq!(
            config.proxy.https_proxy.as_deref(),
            Some("http://global-secure-proxy.example:8080")
        );
        assert_eq!(
            config.proxy.no_proxy.as_deref(),
            Some("confluence.example,localhost")
        );
    }

    #[test]
    fn proxy_config_prefers_confluence_specific_no_proxy() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://confluence.example"),
            (ENV_CONFLUENCE_PERSONAL_TOKEN, "test-pat-value"),
            (ENV_NO_PROXY, "global.example"),
            (ENV_CONFLUENCE_NO_PROXY, "confluence.example"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(config.proxy.no_proxy.as_deref(), Some("confluence.example"));
    }

    #[test]
    fn reserved_custom_headers_are_rejected_without_leaking_value() {
        let error = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://confluence.example"),
            (ENV_CONFLUENCE_PERSONAL_TOKEN, "test-pat-value"),
            (ENV_CONFLUENCE_CUSTOM_HEADERS, "Cookie=session-secret"),
        ])
        .unwrap_err();

        assert_eq!(
            error,
            ConfigError::ReservedCustomHeader {
                variable: ENV_CONFLUENCE_CUSTOM_HEADERS,
                header: "cookie".to_string(),
            }
        );
        assert!(!error.to_string().contains("session-secret"));
    }

    #[test]
    fn invalid_timeout_is_rejected() {
        let error = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://confluence.example"),
            (ENV_CONFLUENCE_PERSONAL_TOKEN, "test-pat-value"),
            (ENV_CONFLUENCE_TIMEOUT, "0"),
        ])
        .unwrap_err();

        assert_eq!(
            error,
            ConfigError::InvalidConfluenceTimeout {
                variable: ENV_CONFLUENCE_TIMEOUT,
                value: "0".to_string(),
            }
        );
    }

    #[test]
    fn cloud_config_builds_oauth_access_token_auth_with_cloud_id() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://example.atlassian.net/wiki"),
            (ENV_ATLASSIAN_OAUTH_CLOUD_ID, " cloud-123 "),
            (ENV_CONFLUENCE_OAUTH_ACCESS_TOKEN, "test-access-token"),
            (ENV_CONFLUENCE_USERNAME, "user@example.com"),
            (ENV_CONFLUENCE_API_TOKEN, "test-api-token"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(config.deployment, ConfluenceDeployment::Cloud);
        assert_eq!(
            config.auth,
            UpstreamAuth::OAuthAccessToken {
                access_token: "test-access-token".to_string(),
            }
        );
        assert_eq!(config.oauth_cloud_id.as_deref(), Some("cloud-123"));
        assert!(!format!("{:?}", config.auth).contains("test-access-token"));
    }

    #[test]
    fn shared_oauth_access_token_is_used_when_service_specific_value_is_absent() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://example.atlassian.net/wiki"),
            (ENV_ATLASSIAN_OAUTH_CLOUD_ID, "cloud-123"),
            (ENV_ATLASSIAN_OAUTH_ACCESS_TOKEN, "shared-access-token"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.auth,
            UpstreamAuth::OAuthAccessToken {
                access_token: "shared-access-token".to_string(),
            }
        );
    }

    #[test]
    fn cloud_oauth_access_token_requires_cloud_id_without_leaking_secret() {
        let error = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://example.atlassian.net/wiki"),
            (ENV_CONFLUENCE_OAUTH_ACCESS_TOKEN, "test-access-token"),
        ])
        .unwrap_err();

        assert_eq!(
            error,
            ConfigError::MissingConfluenceOAuthCloudId {
                access_token_variables: vec![ENV_CONFLUENCE_OAUTH_ACCESS_TOKEN],
                cloud_id_variable: ENV_ATLASSIAN_OAUTH_CLOUD_ID,
            }
        );
        assert!(!error.to_string().contains("test-access-token"));
    }

    #[test]
    fn server_config_prefers_pat_over_oauth_access_token_and_basic() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://confluence.example"),
            (ENV_CONFLUENCE_PERSONAL_TOKEN, "test-pat-value"),
            (ENV_CONFLUENCE_OAUTH_ACCESS_TOKEN, "test-access-token"),
            (ENV_CONFLUENCE_USERNAME, "user@example.com"),
            (ENV_CONFLUENCE_API_TOKEN, "test-api-token"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.auth,
            UpstreamAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            }
        );
    }

    #[test]
    fn server_config_uses_oauth_access_token_without_pat() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://confluence.example"),
            (ENV_CONFLUENCE_OAUTH_ACCESS_TOKEN, "test-access-token"),
            (ENV_CONFLUENCE_USERNAME, "user@example.com"),
            (ENV_CONFLUENCE_API_TOKEN, "test-api-token"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.auth,
            UpstreamAuth::OAuthAccessToken {
                access_token: "test-access-token".to_string(),
            }
        );
        assert_eq!(config.base_url, "https://confluence.example");
    }

    #[test]
    fn cloud_byot_config_rewrites_base_url_to_atlassian_api_gateway() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://example.atlassian.net/wiki"),
            (ENV_ATLASSIAN_OAUTH_CLOUD_ID, "cloud-123"),
            (ENV_CONFLUENCE_OAUTH_ACCESS_TOKEN, "test-access-token"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.base_url,
            "https://api.atlassian.com/ex/confluence/cloud-123/wiki"
        );
    }

    #[test]
    fn cloud_byot_config_percent_encodes_cloud_id_path_segment() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://example.atlassian.net/wiki"),
            (ENV_ATLASSIAN_OAUTH_CLOUD_ID, "cloud id/123"),
            (ENV_CONFLUENCE_OAUTH_ACCESS_TOKEN, "test-access-token"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.base_url,
            "https://api.atlassian.com/ex/confluence/cloud%20id%2F123/wiki"
        );
    }

    #[test]
    fn invalid_url_is_rejected() {
        let error = config_from_pairs(&[(ENV_CONFLUENCE_URL, "not-a-url")]).unwrap_err();

        assert_eq!(
            error,
            ConfigError::InvalidConfluenceUrl {
                variable: ENV_CONFLUENCE_URL,
            }
        );
    }
}
