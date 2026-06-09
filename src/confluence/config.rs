use std::collections::BTreeSet;

use reqwest::Url;

use crate::{
    atlassian::{
        auth::AtlassianAuth,
        compat::{
            ENV_ATLASSIAN_OAUTH_ACCESS_TOKEN, ENV_ATLASSIAN_OAUTH_CLOUD_ID,
            ENV_CONFLUENCE_CLIENT_CERT, ENV_CONFLUENCE_CLIENT_KEY, ENV_CONFLUENCE_CUSTOM_HEADERS,
            ENV_CONFLUENCE_HTTP_PROXY, ENV_CONFLUENCE_HTTPS_PROXY, ENV_CONFLUENCE_NO_PROXY,
            ENV_CONFLUENCE_OAUTH_ACCESS_TOKEN,
        },
        custom_headers::CustomHeaders,
        mtls::ClientTlsIdentityConfig,
        proxy::ProxyConfig,
    },
    error::ConfigError,
};

pub const ENV_CONFLUENCE_URL: &str = "CONFLUENCE_URL";
pub const ENV_CONFLUENCE_USERNAME: &str = "CONFLUENCE_USERNAME";
pub const ENV_CONFLUENCE_API_TOKEN: &str = "CONFLUENCE_API_TOKEN";
pub const ENV_CONFLUENCE_PERSONAL_TOKEN: &str = "CONFLUENCE_PERSONAL_TOKEN";
pub const ENV_CONFLUENCE_SSL_VERIFY: &str = "CONFLUENCE_SSL_VERIFY";
pub const ENV_CONFLUENCE_SPACES_FILTER: &str = "CONFLUENCE_SPACES_FILTER";
pub const ENV_CONFLUENCE_TIMEOUT: &str = "CONFLUENCE_TIMEOUT";

pub const DEFAULT_CONFLUENCE_TIMEOUT_SECONDS: u64 = 75;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfluenceConfig {
    pub base_url: String,
    pub deployment: ConfluenceDeployment,
    pub auth: AtlassianAuth,
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
        let base_url = optional_var(get_var, ENV_CONFLUENCE_URL);
        let username = optional_var(get_var, ENV_CONFLUENCE_USERNAME);
        let api_token = optional_var(get_var, ENV_CONFLUENCE_API_TOKEN);
        let personal_token = optional_var(get_var, ENV_CONFLUENCE_PERSONAL_TOKEN);
        let service_oauth_access_token = optional_var(get_var, ENV_CONFLUENCE_OAUTH_ACCESS_TOKEN);
        let shared_oauth_access_token = optional_var(get_var, ENV_ATLASSIAN_OAUTH_ACCESS_TOKEN);
        let oauth_access_token = service_oauth_access_token
            .clone()
            .or_else(|| shared_oauth_access_token.clone());
        let oauth_access_token_variables = present_variables([
            (
                ENV_CONFLUENCE_OAUTH_ACCESS_TOKEN,
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
                (ENV_CONFLUENCE_USERNAME, username.as_ref()),
                (ENV_CONFLUENCE_API_TOKEN, api_token.as_ref()),
                (ENV_CONFLUENCE_PERSONAL_TOKEN, personal_token.as_ref()),
                (
                    ENV_CONFLUENCE_OAUTH_ACCESS_TOKEN,
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

            return Err(ConfigError::MissingConfluenceUrl {
                credential_variables,
            });
        };

        let parsed_url = parse_base_url(&base_url)?;
        let deployment = ConfluenceDeployment::from_base_url(&parsed_url);
        let (auth, oauth_cloud_id) = match deployment {
            ConfluenceDeployment::Cloud => {
                if let Some(access_token) = oauth_access_token {
                    let Some(cloud_id) = oauth_cloud_id else {
                        return Err(ConfigError::MissingConfluenceOAuthCloudId {
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
                        (ENV_CONFLUENCE_USERNAME, username.as_ref()),
                        (ENV_CONFLUENCE_API_TOKEN, api_token.as_ref()),
                    ]);

                    if !missing_variables.is_empty() {
                        return Err(ConfigError::MissingConfluenceCloudCredentials {
                            missing_variables,
                        });
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
            ConfluenceDeployment::ServerDataCenter => {
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
                    return Err(ConfigError::MissingConfluencePersonalToken {
                        variable: ENV_CONFLUENCE_PERSONAL_TOKEN,
                    });
                }
            }
        };

        let base_url = normalize_effective_base_url(parsed_url, deployment, &auth, &oauth_cloud_id);
        let proxy = ProxyConfig::from_var_provider(
            get_var,
            ENV_CONFLUENCE_HTTP_PROXY,
            ENV_CONFLUENCE_HTTPS_PROXY,
            ENV_CONFLUENCE_NO_PROXY,
        )?;
        let custom_headers =
            CustomHeaders::from_var_provider(get_var, ENV_CONFLUENCE_CUSTOM_HEADERS)?;
        let mtls = ClientTlsIdentityConfig::from_var_provider(
            get_var,
            ENV_CONFLUENCE_CLIENT_CERT,
            ENV_CONFLUENCE_CLIENT_KEY,
        )?;

        Ok(Some(Self {
            base_url,
            deployment,
            auth,
            oauth_cloud_id,
            ssl_verify: parse_ssl_verify(
                optional_var(get_var, ENV_CONFLUENCE_SSL_VERIFY).as_deref(),
            ),
            proxy,
            custom_headers,
            mtls,
            spaces_filter: parse_spaces_filter(optional_var(get_var, ENV_CONFLUENCE_SPACES_FILTER)),
            timeout_seconds: parse_timeout_seconds(optional_var(get_var, ENV_CONFLUENCE_TIMEOUT))?,
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

fn parse_base_url(value: &str) -> Result<Url, ConfigError> {
    let url = Url::parse(value).map_err(|_| ConfigError::InvalidConfluenceUrl {
        variable: ENV_CONFLUENCE_URL,
    })?;

    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(ConfigError::InvalidConfluenceUrl {
            variable: ENV_CONFLUENCE_URL,
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
    deployment: ConfluenceDeployment,
    auth: &AtlassianAuth,
    oauth_cloud_id: &Option<String>,
) -> String {
    if deployment == ConfluenceDeployment::Cloud
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
        .extend(["ex", "confluence", cloud_id, "wiki"]);
    url.to_string().trim_end_matches('/').to_string()
}

fn parse_ssl_verify(value: Option<&str>) -> bool {
    !matches!(
        value.map(|value| value.trim().to_ascii_lowercase()),
        Some(value) if matches!(value.as_str(), "false" | "0" | "no" | "off")
    )
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

fn parse_timeout_seconds(value: Option<String>) -> Result<u64, ConfigError> {
    let Some(value) = value else {
        return Ok(DEFAULT_CONFLUENCE_TIMEOUT_SECONDS);
    };

    let seconds: u64 = value
        .parse()
        .map_err(|_| ConfigError::InvalidConfluenceTimeout {
            variable: ENV_CONFLUENCE_TIMEOUT,
            value: value.clone(),
        })?;

    if seconds == 0 {
        return Err(ConfigError::InvalidConfluenceTimeout {
            variable: ENV_CONFLUENCE_TIMEOUT,
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
            AtlassianAuth::Basic {
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
            AtlassianAuth::Pat {
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
            (ENV_CONFLUENCE_URL, "https://example.atlassian.net/wiki"),
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
            AtlassianAuth::Pat {
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
            AtlassianAuth::OAuthAccessToken {
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
