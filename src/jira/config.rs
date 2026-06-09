use std::collections::BTreeSet;

use reqwest::Url;

use crate::{
    atlassian::{
        auth::AtlassianAuth,
        compat::{
            ENV_ATLASSIAN_OAUTH_ACCESS_TOKEN, ENV_ATLASSIAN_OAUTH_CLOUD_ID, ENV_JIRA_CLIENT_CERT,
            ENV_JIRA_CLIENT_KEY, ENV_JIRA_CUSTOM_HEADERS, ENV_JIRA_HTTP_PROXY,
            ENV_JIRA_HTTPS_PROXY, ENV_JIRA_NO_PROXY, ENV_JIRA_OAUTH_ACCESS_TOKEN,
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
        let base_url = optional_var(get_var, ENV_JIRA_URL);
        let username = optional_var(get_var, ENV_JIRA_USERNAME);
        let api_token = optional_var(get_var, ENV_JIRA_API_TOKEN);
        let personal_token = optional_var(get_var, ENV_JIRA_PERSONAL_TOKEN);
        let service_oauth_access_token = optional_var(get_var, ENV_JIRA_OAUTH_ACCESS_TOKEN);
        let shared_oauth_access_token = optional_var(get_var, ENV_ATLASSIAN_OAUTH_ACCESS_TOKEN);
        let oauth_access_token = service_oauth_access_token
            .clone()
            .or_else(|| shared_oauth_access_token.clone());
        let oauth_access_token_variables = present_variables([
            (
                ENV_JIRA_OAUTH_ACCESS_TOKEN,
                service_oauth_access_token.as_ref(),
            ),
            (
                ENV_ATLASSIAN_OAUTH_ACCESS_TOKEN,
                shared_oauth_access_token.as_ref(),
            ),
        ]);
        let oauth_cloud_id = optional_var(get_var, ENV_ATLASSIAN_OAUTH_CLOUD_ID);

        let Some(base_url) = base_url else {
            let credential_variables = present_variables([
                (ENV_JIRA_USERNAME, username.as_ref()),
                (ENV_JIRA_API_TOKEN, api_token.as_ref()),
                (ENV_JIRA_PERSONAL_TOKEN, personal_token.as_ref()),
                (
                    ENV_JIRA_OAUTH_ACCESS_TOKEN,
                    service_oauth_access_token.as_ref(),
                ),
                (
                    ENV_ATLASSIAN_OAUTH_ACCESS_TOKEN,
                    shared_oauth_access_token.as_ref(),
                ),
            ]);

            if credential_variables.is_empty() {
                return Ok(None);
            }

            return Err(ConfigError::MissingJiraUrl {
                credential_variables,
            });
        };

        let parsed_url = parse_base_url(&base_url)?;
        let deployment = JiraDeployment::from_base_url(&parsed_url);
        let (auth, oauth_cloud_id) = match deployment {
            JiraDeployment::Cloud => {
                if let Some(access_token) = oauth_access_token {
                    let Some(cloud_id) = oauth_cloud_id else {
                        return Err(ConfigError::MissingJiraOAuthCloudId {
                            access_token_variables: oauth_access_token_variables,
                            cloud_id_variable: ENV_ATLASSIAN_OAUTH_CLOUD_ID,
                        });
                    };

                    (
                        AtlassianAuth::OAuthAccessToken { access_token },
                        Some(cloud_id),
                    )
                } else {
                    let missing_variables = missing_variables([
                        (ENV_JIRA_USERNAME, username.as_ref()),
                        (ENV_JIRA_API_TOKEN, api_token.as_ref()),
                    ]);

                    if !missing_variables.is_empty() {
                        return Err(ConfigError::MissingJiraCloudCredentials { missing_variables });
                    }

                    (
                        AtlassianAuth::Basic {
                            username: username.expect("missing variables were checked"),
                            api_token: api_token.expect("missing variables were checked"),
                        },
                        None,
                    )
                }
            }
            JiraDeployment::ServerDataCenter => {
                if let Some(personal_token) = personal_token {
                    (AtlassianAuth::Pat { personal_token }, None)
                } else if let Some(access_token) = oauth_access_token {
                    (AtlassianAuth::OAuthAccessToken { access_token }, None)
                } else if let (Some(username), Some(api_token)) = (username, api_token) {
                    (
                        AtlassianAuth::Basic {
                            username,
                            api_token,
                        },
                        None,
                    )
                } else {
                    return Err(ConfigError::MissingJiraPersonalToken {
                        variable: ENV_JIRA_PERSONAL_TOKEN,
                    });
                }
            }
        };

        let base_url = normalize_effective_base_url(parsed_url, deployment, &auth, &oauth_cloud_id);
        let proxy = ProxyConfig::from_var_provider(
            get_var,
            ENV_JIRA_HTTP_PROXY,
            ENV_JIRA_HTTPS_PROXY,
            ENV_JIRA_NO_PROXY,
        )?;
        let custom_headers = CustomHeaders::from_var_provider(get_var, ENV_JIRA_CUSTOM_HEADERS)?;
        let mtls = ClientTlsIdentityConfig::from_var_provider(
            get_var,
            ENV_JIRA_CLIENT_CERT,
            ENV_JIRA_CLIENT_KEY,
        )?;

        Ok(Some(Self {
            base_url,
            deployment,
            auth,
            oauth_cloud_id,
            ssl_verify: parse_ssl_verify(optional_var(get_var, ENV_JIRA_SSL_VERIFY).as_deref()),
            proxy,
            custom_headers,
            mtls,
            projects_filter: parse_project_filter(optional_var(get_var, ENV_JIRA_PROJECTS_FILTER)),
            timeout_seconds: parse_timeout_seconds(optional_var(get_var, ENV_JIRA_TIMEOUT))?,
        }))
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

fn parse_base_url(value: &str) -> Result<Url, ConfigError> {
    let url = Url::parse(value).map_err(|_| ConfigError::InvalidJiraUrl {
        variable: ENV_JIRA_URL,
    })?;

    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(ConfigError::InvalidJiraUrl {
            variable: ENV_JIRA_URL,
        });
    }

    Ok(url)
}

fn normalize_base_url(mut url: Url) -> String {
    url.set_query(None);
    url.set_fragment(None);
    url.to_string().trim_end_matches('/').to_string()
}

fn normalize_effective_base_url(
    url: Url,
    deployment: JiraDeployment,
    auth: &AtlassianAuth,
    oauth_cloud_id: &Option<String>,
) -> String {
    if deployment == JiraDeployment::Cloud
        && matches!(auth, AtlassianAuth::OAuthAccessToken { .. })
        && let Some(cloud_id) = oauth_cloud_id
    {
        return cloud_oauth_api_base_url(cloud_id);
    }

    normalize_base_url(url)
}

fn cloud_oauth_api_base_url(cloud_id: &str) -> String {
    let mut url = Url::parse("https://api.atlassian.com").expect("static URL is valid");
    url.path_segments_mut()
        .expect("static URL supports path segments")
        .extend(["ex", "jira", cloud_id]);
    url.to_string().trim_end_matches('/').to_string()
}

fn parse_ssl_verify(value: Option<&str>) -> bool {
    !matches!(
        value.map(|value| value.trim().to_ascii_lowercase()),
        Some(value) if matches!(value.as_str(), "false" | "0" | "no" | "off")
    )
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

fn parse_timeout_seconds(value: Option<String>) -> Result<u64, ConfigError> {
    let Some(value) = value else {
        return Ok(DEFAULT_JIRA_TIMEOUT_SECONDS);
    };

    let seconds: u64 = value.parse().map_err(|_| ConfigError::InvalidJiraTimeout {
        variable: ENV_JIRA_TIMEOUT,
        value: value.clone(),
    })?;

    if seconds == 0 {
        return Err(ConfigError::InvalidJiraTimeout {
            variable: ENV_JIRA_TIMEOUT,
            value,
        });
    }

    Ok(seconds)
}

fn present_variables<const N: usize>(
    variables: [(&'static str, Option<&String>); N],
) -> Vec<&'static str> {
    variables
        .into_iter()
        .filter_map(|(name, value)| value.map(|_| name))
        .collect()
}

fn missing_variables<const N: usize>(
    variables: [(&'static str, Option<&String>); N],
) -> Vec<&'static str> {
    variables
        .into_iter()
        .filter_map(|(name, value)| if value.is_none() { Some(name) } else { None })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::atlassian::compat::{ENV_HTTP_PROXY, ENV_HTTPS_PROXY, ENV_NO_PROXY};

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
