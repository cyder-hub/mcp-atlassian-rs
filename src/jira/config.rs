use std::collections::BTreeSet;

use reqwest::Url;

use crate::{
    atlassian::{
        auth::AtlassianAuth,
        compat::{
            ENV_JIRA_CLIENT_CERT, ENV_JIRA_CLIENT_KEY, ENV_JIRA_CUSTOM_HEADERS,
            ENV_JIRA_HTTP_PROXY, ENV_JIRA_HTTPS_PROXY, ENV_JIRA_NO_PROXY,
            ENV_JIRA_OAUTH_ACCESS_TOKEN,
        },
        config::{
            AtlassianServiceConfigSpec, ParsedAtlassianServiceConfig,
            parse_atlassian_service_config,
        },
        custom_headers::CustomHeaders,
        mtls::ClientTlsIdentityConfig,
        proxy::ProxyConfig,
    },
    error::ConfigError,
};

pub const ENV_JIRA_URL: &str = "JIRA_URL";
pub const ENV_JIRA_USERNAME: &str = "JIRA_USERNAME";
pub const ENV_JIRA_API_TOKEN: &str = "JIRA_API_TOKEN";
pub const ENV_JIRA_PERSONAL_TOKEN: &str = "JIRA_PERSONAL_TOKEN";
pub const ENV_JIRA_SSL_VERIFY: &str = "JIRA_SSL_VERIFY";
pub const ENV_JIRA_PROJECTS_FILTER: &str = "JIRA_PROJECTS_FILTER";
pub const ENV_JIRA_TIMEOUT: &str = "JIRA_TIMEOUT";

pub const DEFAULT_JIRA_TIMEOUT_SECONDS: u64 = 75;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JiraConfig {
    pub base_url: String,
    pub deployment: JiraDeployment,
    pub auth: AtlassianAuth,
    pub oauth_cloud_id: Option<String>,
    pub ssl_verify: bool,
    pub proxy: ProxyConfig,
    pub custom_headers: CustomHeaders,
    pub mtls: Option<ClientTlsIdentityConfig>,
    pub projects_filter: BTreeSet<String>,
    pub timeout_seconds: u64,
}

impl JiraConfig {
    pub fn from_var_provider<F, E>(get_var: &mut F) -> Result<Option<Self>, ConfigError>
    where
        F: FnMut(&str) -> Result<String, E>,
    {
        let Some(parsed) = parse_atlassian_service_config(get_var, &jira_config_spec())? else {
            return Ok(None);
        };

        Ok(Some(Self::from_parsed(
            parsed,
            parse_project_filter(optional_var(get_var, ENV_JIRA_PROJECTS_FILTER)),
        )))
    }

    pub fn is_auth_configured(&self) -> bool {
        matches!(
            self.auth,
            AtlassianAuth::Basic { .. }
                | AtlassianAuth::Pat { .. }
                | AtlassianAuth::OAuthAccessToken { .. }
        )
    }

    pub fn with_auth_override(&self, auth: AtlassianAuth, oauth_cloud_id: Option<String>) -> Self {
        let mut config = self.clone();
        let effective_cloud_id = if matches!(auth, AtlassianAuth::OAuthAccessToken { .. })
            && self.deployment == JiraDeployment::Cloud
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
        parsed: ParsedAtlassianServiceConfig<JiraDeployment>,
        projects_filter: BTreeSet<String>,
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
            projects_filter,
            timeout_seconds: parsed.timeout_seconds,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JiraDeployment {
    Cloud,
    ServerDataCenter,
}

impl JiraDeployment {
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
        .extend(["ex", "jira", cloud_id]);
    url.to_string().trim_end_matches('/').to_string()
}

fn parse_project_filter(value: Option<String>) -> BTreeSet<String> {
    value
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn jira_config_spec() -> AtlassianServiceConfigSpec<JiraDeployment> {
    AtlassianServiceConfigSpec {
        url_variable: ENV_JIRA_URL,
        username_variable: ENV_JIRA_USERNAME,
        api_token_variable: ENV_JIRA_API_TOKEN,
        personal_token_variable: ENV_JIRA_PERSONAL_TOKEN,
        oauth_access_token_variable: ENV_JIRA_OAUTH_ACCESS_TOKEN,
        ssl_verify_variable: ENV_JIRA_SSL_VERIFY,
        timeout_variable: ENV_JIRA_TIMEOUT,
        http_proxy_variable: ENV_JIRA_HTTP_PROXY,
        https_proxy_variable: ENV_JIRA_HTTPS_PROXY,
        no_proxy_variable: ENV_JIRA_NO_PROXY,
        custom_headers_variable: ENV_JIRA_CUSTOM_HEADERS,
        client_cert_variable: ENV_JIRA_CLIENT_CERT,
        client_key_variable: ENV_JIRA_CLIENT_KEY,
        default_timeout_seconds: DEFAULT_JIRA_TIMEOUT_SECONDS,
        cloud_deployment: JiraDeployment::Cloud,
        deployment_from_url: JiraDeployment::from_base_url,
        cloud_oauth_api_base_url,
        missing_url_error: |credential_variables| ConfigError::MissingJiraUrl {
            credential_variables,
        },
        invalid_url_error: |variable| ConfigError::InvalidJiraUrl { variable },
        missing_cloud_credentials_error: |missing_variables| {
            ConfigError::MissingJiraCloudCredentials { missing_variables }
        },
        missing_personal_token_error: |variable| ConfigError::MissingJiraPersonalToken { variable },
        missing_oauth_cloud_id_error: |access_token_variables, cloud_id_variable| {
            ConfigError::MissingJiraOAuthCloudId {
                access_token_variables,
                cloud_id_variable,
            }
        },
        invalid_timeout_error: |variable, value| ConfigError::InvalidJiraTimeout {
            variable,
            value,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::atlassian::compat::{
        ENV_ATLASSIAN_API_TOKEN, ENV_ATLASSIAN_CLIENT_CERT, ENV_ATLASSIAN_CLIENT_KEY,
        ENV_ATLASSIAN_CUSTOM_HEADERS, ENV_ATLASSIAN_OAUTH_ACCESS_TOKEN,
        ENV_ATLASSIAN_OAUTH_CLOUD_ID, ENV_ATLASSIAN_PERSONAL_TOKEN, ENV_ATLASSIAN_SSL_VERIFY,
        ENV_ATLASSIAN_TIMEOUT, ENV_ATLASSIAN_USERNAME, ENV_HTTP_PROXY, ENV_HTTPS_PROXY,
        ENV_NO_PROXY,
    };

    use super::*;

    fn config_from_pairs(pairs: &[(&str, &str)]) -> Result<Option<JiraConfig>, ConfigError> {
        let vars: BTreeMap<String, String> = pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();

        JiraConfig::from_var_provider(&mut |key| vars.get(key).cloned().ok_or(()))
    }

    #[test]
    fn jira_config_is_disabled_without_url_or_credentials() {
        assert_eq!(config_from_pairs(&[]).unwrap(), None);
    }

    #[test]
    fn jira_credentials_without_url_are_rejected() {
        let error = config_from_pairs(&[(ENV_JIRA_PERSONAL_TOKEN, "test-pat-value")]).unwrap_err();

        assert_eq!(
            error,
            ConfigError::MissingJiraUrl {
                credential_variables: vec![ENV_JIRA_PERSONAL_TOKEN],
            }
        );
        assert!(!error.to_string().contains("test-pat-value"));
    }

    #[test]
    fn cloud_config_requires_username_and_api_token() {
        let error =
            config_from_pairs(&[(ENV_JIRA_URL, "https://example.atlassian.net")]).unwrap_err();

        assert_eq!(
            error,
            ConfigError::MissingJiraCloudCredentials {
                missing_variables: vec![ENV_JIRA_USERNAME, ENV_JIRA_API_TOKEN],
            }
        );
    }

    #[test]
    fn cloud_config_builds_basic_auth() {
        let config = config_from_pairs(&[
            (ENV_JIRA_URL, " https://example.atlassian.net/ "),
            (ENV_JIRA_USERNAME, "user@example.com"),
            (ENV_JIRA_API_TOKEN, "test-api-token"),
            (ENV_JIRA_SSL_VERIFY, "off"),
            (ENV_JIRA_PROJECTS_FILTER, " ABC,XYZ,ABC "),
            (ENV_JIRA_TIMEOUT, "30"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(config.base_url, "https://example.atlassian.net");
        assert_eq!(config.deployment, JiraDeployment::Cloud);
        assert_eq!(
            config.auth,
            AtlassianAuth::Basic {
                username: "user@example.com".to_string(),
                api_token: "test-api-token".to_string(),
            }
        );
        assert_eq!(config.oauth_cloud_id, None);
        assert!(!config.ssl_verify);
        assert_eq!(
            config.projects_filter,
            BTreeSet::from(["ABC".to_string(), "XYZ".to_string()])
        );
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.is_auth_configured());
    }

    #[test]
    fn server_config_requires_pat() {
        let error = config_from_pairs(&[(ENV_JIRA_URL, "https://jira.example")]).unwrap_err();

        assert_eq!(
            error,
            ConfigError::MissingJiraPersonalToken {
                variable: ENV_JIRA_PERSONAL_TOKEN,
            }
        );
    }

    #[test]
    fn server_config_builds_pat_auth_with_defaults() {
        let config = config_from_pairs(&[
            (ENV_JIRA_URL, "https://jira.example"),
            (ENV_JIRA_PERSONAL_TOKEN, "test-pat-value"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(config.base_url, "https://jira.example");
        assert_eq!(config.deployment, JiraDeployment::ServerDataCenter);
        assert_eq!(
            config.auth,
            AtlassianAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            }
        );
        assert_eq!(config.oauth_cloud_id, None);
        assert!(config.ssl_verify);
        assert!(config.proxy.is_empty());
        assert!(config.projects_filter.is_empty());
        assert_eq!(config.timeout_seconds, DEFAULT_JIRA_TIMEOUT_SECONDS);
        assert!(config.is_auth_configured());
    }

    #[test]
    fn atlassian_fallbacks_build_jira_config() {
        let config = config_from_pairs(&[
            (ENV_JIRA_URL, "https://example.atlassian.net"),
            (ENV_ATLASSIAN_USERNAME, "user@example.com"),
            (ENV_ATLASSIAN_API_TOKEN, "global-api-token"),
            (ENV_ATLASSIAN_SSL_VERIFY, "off"),
            (ENV_ATLASSIAN_TIMEOUT, "33"),
            (ENV_ATLASSIAN_CUSTOM_HEADERS, "X-Team=platform"),
            (ENV_ATLASSIAN_CLIENT_CERT, "/tmp/global-client.crt"),
            (ENV_ATLASSIAN_CLIENT_KEY, "/tmp/global-client.key"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.auth,
            AtlassianAuth::Basic {
                username: "user@example.com".to_string(),
                api_token: "global-api-token".to_string(),
            }
        );
        assert!(!config.ssl_verify);
        assert_eq!(config.timeout_seconds, 33);
        assert_eq!(
            config
                .custom_headers
                .iter()
                .map(|(name, value)| (name.as_str(), value.to_str().unwrap()))
                .collect::<Vec<_>>(),
            vec![("x-team", "platform")]
        );
        assert_eq!(
            config.mtls.unwrap().cert_path,
            std::path::PathBuf::from("/tmp/global-client.crt")
        );
    }

    #[test]
    fn jira_specific_values_override_atlassian_fallbacks() {
        let config = config_from_pairs(&[
            (ENV_JIRA_URL, "https://jira.example"),
            (ENV_ATLASSIAN_PERSONAL_TOKEN, "global-pat-value"),
            (ENV_JIRA_PERSONAL_TOKEN, "jira-pat-value"),
            (ENV_ATLASSIAN_TIMEOUT, "0"),
            (ENV_JIRA_TIMEOUT, "20"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.auth,
            AtlassianAuth::Pat {
                personal_token: "jira-pat-value".to_string(),
            }
        );
        assert_eq!(config.timeout_seconds, 20);
    }

    #[test]
    fn proxy_config_prefers_jira_specific_values_before_global() {
        let config = config_from_pairs(&[
            (ENV_JIRA_URL, "https://jira.example"),
            (ENV_JIRA_PERSONAL_TOKEN, "test-pat-value"),
            (ENV_HTTP_PROXY, "http://global-proxy.example:8080"),
            (ENV_JIRA_HTTP_PROXY, "http://jira-proxy.example:8080"),
            (ENV_HTTPS_PROXY, "http://global-secure-proxy.example:8080"),
            (
                ENV_JIRA_HTTPS_PROXY,
                "https://jira-secure-proxy.example:8443",
            ),
            (ENV_NO_PROXY, "global.example"),
            (ENV_JIRA_NO_PROXY, " jira.example,localhost "),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.proxy.http_proxy.as_deref(),
            Some("http://jira-proxy.example:8080")
        );
        assert_eq!(
            config.proxy.https_proxy.as_deref(),
            Some("https://jira-secure-proxy.example:8443")
        );
        assert_eq!(
            config.proxy.no_proxy.as_deref(),
            Some("jira.example,localhost")
        );
    }

    #[test]
    fn invalid_proxy_url_is_rejected_without_leaking_credentials() {
        let error = config_from_pairs(&[
            (ENV_JIRA_URL, "https://jira.example"),
            (ENV_JIRA_PERSONAL_TOKEN, "test-pat-value"),
            (ENV_JIRA_HTTP_PROXY, "ftp://user:secret@proxy.example"),
        ])
        .unwrap_err();

        assert_eq!(
            error,
            ConfigError::InvalidProxyUrl {
                variable: ENV_JIRA_HTTP_PROXY,
            }
        );
        assert!(!error.to_string().contains("secret"));
        assert!(!format!("{error:?}").contains("secret"));
    }

    #[test]
    fn custom_headers_are_parsed_into_typed_config() {
        let config = config_from_pairs(&[
            (ENV_JIRA_URL, "https://jira.example"),
            (ENV_JIRA_PERSONAL_TOKEN, "test-pat-value"),
            (ENV_JIRA_CUSTOM_HEADERS, "X-Team=platform,X-Trace=abc=def"),
        ])
        .unwrap()
        .unwrap();
        let headers = config
            .custom_headers
            .iter()
            .map(|(name, value)| {
                (
                    name.as_str().to_string(),
                    value.to_str().unwrap().to_string(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            headers,
            vec![
                ("x-team".to_string(), "platform".to_string()),
                ("x-trace".to_string(), "abc=def".to_string()),
            ]
        );
        assert!(!format!("{:?}", config.custom_headers).contains("platform"));
    }

    #[test]
    fn mtls_cert_key_pair_is_parsed_into_typed_config() {
        let config = config_from_pairs(&[
            (ENV_JIRA_URL, "https://jira.example"),
            (ENV_JIRA_PERSONAL_TOKEN, "test-pat-value"),
            (ENV_JIRA_CLIENT_CERT, "/tmp/jira-client.crt"),
            (ENV_JIRA_CLIENT_KEY, "/tmp/jira-client.key"),
        ])
        .unwrap()
        .unwrap();
        let mtls = config.mtls.unwrap();

        assert_eq!(
            mtls.cert_path,
            std::path::PathBuf::from("/tmp/jira-client.crt")
        );
        assert_eq!(
            mtls.key_path,
            std::path::PathBuf::from("/tmp/jira-client.key")
        );
    }

    #[test]
    fn mtls_missing_cert_key_pair_is_rejected() {
        let error = config_from_pairs(&[
            (ENV_JIRA_URL, "https://jira.example"),
            (ENV_JIRA_PERSONAL_TOKEN, "test-pat-value"),
            (ENV_JIRA_CLIENT_CERT, "/tmp/jira-client.crt"),
        ])
        .unwrap_err();

        assert_eq!(
            error,
            ConfigError::MissingClientCertKeyPair {
                cert_variable: ENV_JIRA_CLIENT_CERT,
                key_variable: ENV_JIRA_CLIENT_KEY,
            }
        );
    }

    #[test]
    fn invalid_timeout_is_rejected() {
        let error = config_from_pairs(&[
            (ENV_JIRA_URL, "https://jira.example"),
            (ENV_JIRA_PERSONAL_TOKEN, "test-pat-value"),
            (ENV_JIRA_TIMEOUT, "0"),
        ])
        .unwrap_err();

        assert_eq!(
            error,
            ConfigError::InvalidJiraTimeout {
                variable: ENV_JIRA_TIMEOUT,
                value: "0".to_string(),
            }
        );
    }

    #[test]
    fn cloud_config_builds_oauth_access_token_auth_with_cloud_id() {
        let config = config_from_pairs(&[
            (ENV_JIRA_URL, "https://example.atlassian.net"),
            (ENV_ATLASSIAN_OAUTH_CLOUD_ID, " cloud-123 "),
            (ENV_JIRA_OAUTH_ACCESS_TOKEN, "test-access-token"),
            (ENV_JIRA_USERNAME, "user@example.com"),
            (ENV_JIRA_API_TOKEN, "test-api-token"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(config.deployment, JiraDeployment::Cloud);
        assert_eq!(
            config.base_url,
            "https://api.atlassian.com/ex/jira/cloud-123"
        );
        assert_eq!(
            config.auth,
            AtlassianAuth::OAuthAccessToken {
                access_token: "test-access-token".to_string(),
            }
        );
        assert_eq!(config.oauth_cloud_id.as_deref(), Some("cloud-123"));
        assert!(!format!("{:?}", config.auth).contains("test-access-token"));
    }

    #[test]
    fn shared_oauth_access_token_is_used_when_service_specific_value_is_absent() {
        let config = config_from_pairs(&[
            (ENV_JIRA_URL, "https://example.atlassian.net"),
            (ENV_ATLASSIAN_OAUTH_CLOUD_ID, "cloud-123"),
            (ENV_ATLASSIAN_OAUTH_ACCESS_TOKEN, "shared-access-token"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.auth,
            AtlassianAuth::OAuthAccessToken {
                access_token: "shared-access-token".to_string(),
            }
        );
    }

    #[test]
    fn cloud_oauth_access_token_requires_cloud_id_without_leaking_secret() {
        let error = config_from_pairs(&[
            (ENV_JIRA_URL, "https://example.atlassian.net"),
            (ENV_JIRA_OAUTH_ACCESS_TOKEN, "test-access-token"),
        ])
        .unwrap_err();

        assert_eq!(
            error,
            ConfigError::MissingJiraOAuthCloudId {
                access_token_variables: vec![ENV_JIRA_OAUTH_ACCESS_TOKEN],
                cloud_id_variable: ENV_ATLASSIAN_OAUTH_CLOUD_ID,
            }
        );
        assert!(!error.to_string().contains("test-access-token"));
    }

    #[test]
    fn server_config_prefers_pat_over_oauth_access_token_and_basic() {
        let config = config_from_pairs(&[
            (ENV_JIRA_URL, "https://jira.example"),
            (ENV_JIRA_PERSONAL_TOKEN, "test-pat-value"),
            (ENV_JIRA_OAUTH_ACCESS_TOKEN, "test-access-token"),
            (ENV_JIRA_USERNAME, "user@example.com"),
            (ENV_JIRA_API_TOKEN, "test-api-token"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.auth,
            AtlassianAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            }
        );
    }

    #[test]
    fn server_config_uses_oauth_access_token_without_pat() {
        let config = config_from_pairs(&[
            (ENV_JIRA_URL, "https://jira.example"),
            (ENV_JIRA_OAUTH_ACCESS_TOKEN, "test-access-token"),
            (ENV_JIRA_USERNAME, "user@example.com"),
            (ENV_JIRA_API_TOKEN, "test-api-token"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.auth,
            AtlassianAuth::OAuthAccessToken {
                access_token: "test-access-token".to_string(),
            }
        );
        assert_eq!(config.base_url, "https://jira.example");
    }

    #[test]
    fn cloud_byot_config_rewrites_base_url_to_atlassian_api_gateway() {
        let config = config_from_pairs(&[
            (ENV_JIRA_URL, "https://example.atlassian.net"),
            (ENV_ATLASSIAN_OAUTH_CLOUD_ID, "cloud-123"),
            (ENV_JIRA_OAUTH_ACCESS_TOKEN, "test-access-token"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.base_url,
            "https://api.atlassian.com/ex/jira/cloud-123"
        );
    }

    #[test]
    fn cloud_byot_config_percent_encodes_cloud_id_path_segment() {
        let config = config_from_pairs(&[
            (ENV_JIRA_URL, "https://example.atlassian.net"),
            (ENV_ATLASSIAN_OAUTH_CLOUD_ID, "cloud id/123"),
            (ENV_JIRA_OAUTH_ACCESS_TOKEN, "test-access-token"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.base_url,
            "https://api.atlassian.com/ex/jira/cloud%20id%2F123"
        );
    }

    #[test]
    fn invalid_url_is_rejected() {
        let error = config_from_pairs(&[(ENV_JIRA_URL, "not-a-url")]).unwrap_err();

        assert_eq!(
            error,
            ConfigError::InvalidJiraUrl {
                variable: ENV_JIRA_URL,
            }
        );
    }
}
