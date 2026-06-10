use std::collections::BTreeSet;

use reqwest::{Url, header::HeaderName};

use crate::{
    error::ConfigError,
    upstream::{
        auth::UpstreamAuth, custom_headers::CustomHeaders, mtls::ClientTlsIdentityConfig,
        proxy::ProxyConfig,
    },
};

pub const ENV_GITLAB_URL: &str = "GITLAB_URL";
pub const ENV_GITLAB_TOKEN: &str = "GITLAB_TOKEN";
pub const ENV_GITLAB_PERSONAL_TOKEN: &str = "GITLAB_PERSONAL_TOKEN";
pub const ENV_GITLAB_OAUTH_ACCESS_TOKEN: &str = "GITLAB_OAUTH_ACCESS_TOKEN";
pub const ENV_GITLAB_PROJECTS_FILTER: &str = "GITLAB_PROJECTS_FILTER";
pub const ENV_GITLAB_SSL_VERIFY: &str = "GITLAB_SSL_VERIFY";
pub const ENV_GITLAB_TIMEOUT: &str = "GITLAB_TIMEOUT";
pub const ENV_GITLAB_HTTP_PROXY: &str = "GITLAB_HTTP_PROXY";
pub const ENV_GITLAB_HTTPS_PROXY: &str = "GITLAB_HTTPS_PROXY";
pub const ENV_GITLAB_NO_PROXY: &str = "GITLAB_NO_PROXY";
pub const ENV_GITLAB_CUSTOM_HEADERS: &str = "GITLAB_CUSTOM_HEADERS";
pub const ENV_GITLAB_CLIENT_CERT: &str = "GITLAB_CLIENT_CERT";
pub const ENV_GITLAB_CLIENT_KEY: &str = "GITLAB_CLIENT_KEY";

pub const DEFAULT_GITLAB_TIMEOUT_SECONDS: u64 = 75;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitlabConfig {
    pub base_url: String,
    pub auth: UpstreamAuth,
    pub ssl_verify: bool,
    pub proxy: ProxyConfig,
    pub custom_headers: CustomHeaders,
    pub mtls: Option<ClientTlsIdentityConfig>,
    pub projects_filter: BTreeSet<String>,
    pub timeout_seconds: u64,
}

impl GitlabConfig {
    pub fn from_var_provider<F, E>(get_var: &mut F) -> Result<Option<Self>, ConfigError>
    where
        F: FnMut(&str) -> Result<String, E>,
    {
        let base_url = optional_named_var(get_var, ENV_GITLAB_URL);
        let token = first_named_var(
            get_var,
            [
                ENV_GITLAB_TOKEN,
                ENV_GITLAB_PERSONAL_TOKEN,
                ENV_GITLAB_OAUTH_ACCESS_TOKEN,
            ],
        );

        let Some(base_url) = base_url else {
            if let Some(token) = token {
                return Err(ConfigError::MissingGitlabUrl {
                    credential_variables: vec![token.variable],
                });
            }
            return Ok(None);
        };

        let Some(token) = token else {
            return Err(ConfigError::MissingGitlabToken {
                variable: ENV_GITLAB_TOKEN,
            });
        };

        let parsed_url = parse_base_url(&base_url.value, ENV_GITLAB_URL)?;
        let proxy = ProxyConfig::from_var_provider(
            get_var,
            ENV_GITLAB_HTTP_PROXY,
            ENV_GITLAB_HTTPS_PROXY,
            ENV_GITLAB_NO_PROXY,
        )?;
        let custom_headers = CustomHeaders::from_var_provider(get_var, ENV_GITLAB_CUSTOM_HEADERS)?;
        let mtls = ClientTlsIdentityConfig::from_var_provider(
            get_var,
            ENV_GITLAB_CLIENT_CERT,
            ENV_GITLAB_CLIENT_KEY,
        )?;
        let ssl_verify = parse_ssl_verify(optional_var(get_var, ENV_GITLAB_SSL_VERIFY).as_deref());
        let timeout_seconds = parse_timeout_seconds(
            optional_named_var(get_var, ENV_GITLAB_TIMEOUT),
            DEFAULT_GITLAB_TIMEOUT_SECONDS,
        )?;
        let projects_filter =
            parse_project_filter(optional_var(get_var, ENV_GITLAB_PROJECTS_FILTER));

        Ok(Some(Self {
            base_url: normalize_base_url(parsed_url),
            auth: auth_from_token(token),
            ssl_verify,
            proxy,
            custom_headers,
            mtls,
            projects_filter,
            timeout_seconds,
        }))
    }

    pub fn is_auth_configured(&self) -> bool {
        matches!(
            self.auth,
            UpstreamAuth::HeaderToken { .. }
                | UpstreamAuth::Bearer { .. }
                | UpstreamAuth::OAuthAccessToken { .. }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NamedEnvValue {
    variable: &'static str,
    value: String,
}

fn first_named_var<F, E, const N: usize>(
    get_var: &mut F,
    variables: [&'static str; N],
) -> Option<NamedEnvValue>
where
    F: FnMut(&str) -> Result<String, E>,
{
    variables
        .into_iter()
        .find_map(|variable| optional_named_var(get_var, variable))
}

fn optional_named_var<F, E>(get_var: &mut F, key: &'static str) -> Option<NamedEnvValue>
where
    F: FnMut(&str) -> Result<String, E>,
{
    optional_var(get_var, key).map(|value| NamedEnvValue {
        variable: key,
        value,
    })
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

fn auth_from_token(token: NamedEnvValue) -> UpstreamAuth {
    if token.variable == ENV_GITLAB_OAUTH_ACCESS_TOKEN {
        UpstreamAuth::Bearer { token: token.value }
    } else {
        UpstreamAuth::HeaderToken {
            header_name: HeaderName::from_static("private-token"),
            token: token.value,
        }
    }
}

fn parse_base_url(value: &str, variable: &'static str) -> Result<Url, ConfigError> {
    let url = Url::parse(value).map_err(|_| ConfigError::InvalidGitlabUrl { variable })?;

    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(ConfigError::InvalidGitlabUrl { variable });
    }

    Ok(url)
}

fn normalize_base_url(mut url: Url) -> String {
    url.set_query(None);
    url.set_fragment(None);

    let path = url.path().trim_end_matches('/').to_string();
    if let Some(root_path) = path.strip_suffix("/api/v4") {
        url.set_path(if root_path.is_empty() { "/" } else { root_path });
    }

    url.to_string().trim_end_matches('/').to_string()
}

fn parse_ssl_verify(value: Option<&str>) -> bool {
    !matches!(
        value.map(|value| value.trim().to_ascii_lowercase()),
        Some(value) if matches!(value.as_str(), "false" | "0" | "no" | "off")
    )
}

fn parse_timeout_seconds(
    value: Option<NamedEnvValue>,
    default_timeout_seconds: u64,
) -> Result<u64, ConfigError> {
    let Some(value) = value else {
        return Ok(default_timeout_seconds);
    };

    let seconds: u64 = value
        .value
        .parse()
        .map_err(|_| ConfigError::InvalidGitlabTimeout {
            variable: value.variable,
            value: value.value.clone(),
        })?;

    if seconds == 0 {
        return Err(ConfigError::InvalidGitlabTimeout {
            variable: value.variable,
            value: value.value,
        });
    }

    Ok(seconds)
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

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet},
        path::PathBuf,
    };

    use super::*;

    fn config_from_pairs(pairs: &[(&str, &str)]) -> Result<Option<GitlabConfig>, ConfigError> {
        let vars: BTreeMap<String, String> = pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();

        GitlabConfig::from_var_provider(&mut |key| vars.get(key).cloned().ok_or(()))
    }

    #[test]
    fn returns_none_when_url_and_token_are_absent() {
        assert_eq!(config_from_pairs(&[]).unwrap(), None);
    }

    #[test]
    fn token_without_url_returns_missing_url_without_leaking_token() {
        let error = config_from_pairs(&[(ENV_GITLAB_TOKEN, "secret-token")]).unwrap_err();

        assert_eq!(
            error,
            ConfigError::MissingGitlabUrl {
                credential_variables: vec![ENV_GITLAB_TOKEN],
            }
        );
        assert!(!error.to_string().contains("secret-token"));
    }

    #[test]
    fn url_without_token_returns_missing_token() {
        let error = config_from_pairs(&[(ENV_GITLAB_URL, "https://gitlab.example")]).unwrap_err();

        assert_eq!(
            error,
            ConfigError::MissingGitlabToken {
                variable: ENV_GITLAB_TOKEN,
            }
        );
    }

    #[test]
    fn parses_private_token_config_and_project_filter() {
        let config = config_from_pairs(&[
            (
                ENV_GITLAB_URL,
                "https://gitlab.example/api/v4?token=query-secret",
            ),
            (ENV_GITLAB_TOKEN, "primary-token"),
            (
                ENV_GITLAB_PROJECTS_FILTER,
                "123, group/project , nested/sub/project ",
            ),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(config.base_url, "https://gitlab.example");
        assert!(matches!(
            config.auth,
            UpstreamAuth::HeaderToken { ref header_name, ref token }
                if header_name.as_str() == "private-token" && token == "primary-token"
        ));
        assert_eq!(
            config.projects_filter,
            BTreeSet::from([
                "123".to_string(),
                "group/project".to_string(),
                "nested/sub/project".to_string(),
            ])
        );
        assert_eq!(config.timeout_seconds, DEFAULT_GITLAB_TIMEOUT_SECONDS);
        assert!(config.ssl_verify);
    }

    #[test]
    fn accepts_oauth_access_token_as_bearer() {
        let config = config_from_pairs(&[
            (ENV_GITLAB_URL, "https://gitlab.example"),
            (ENV_GITLAB_OAUTH_ACCESS_TOKEN, "oauth-token"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            config.auth,
            UpstreamAuth::Bearer {
                token: "oauth-token".to_string(),
            }
        );
    }

    #[test]
    fn token_precedence_prefers_gitlab_token_then_personal_then_oauth() {
        let config = config_from_pairs(&[
            (ENV_GITLAB_URL, "https://gitlab.example"),
            (ENV_GITLAB_TOKEN, "primary-token"),
            (ENV_GITLAB_PERSONAL_TOKEN, "personal-token"),
            (ENV_GITLAB_OAUTH_ACCESS_TOKEN, "oauth-token"),
        ])
        .unwrap()
        .unwrap();

        assert!(matches!(
            config.auth,
            UpstreamAuth::HeaderToken { ref token, .. } if token == "primary-token"
        ));

        let config = config_from_pairs(&[
            (ENV_GITLAB_URL, "https://gitlab.example"),
            (ENV_GITLAB_PERSONAL_TOKEN, "personal-token"),
            (ENV_GITLAB_OAUTH_ACCESS_TOKEN, "oauth-token"),
        ])
        .unwrap()
        .unwrap();

        assert!(matches!(
            config.auth,
            UpstreamAuth::HeaderToken { ref token, .. } if token == "personal-token"
        ));
    }

    #[test]
    fn normalizes_relative_gitlab_api_path_to_instance_root() {
        let config = config_from_pairs(&[
            (ENV_GITLAB_URL, "https://gitlab.example/gitlab/api/v4/"),
            (ENV_GITLAB_TOKEN, "token"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(config.base_url, "https://gitlab.example/gitlab");
    }

    #[test]
    fn parses_timeout_ssl_proxy_custom_headers_and_mtls() {
        let config = config_from_pairs(&[
            (ENV_GITLAB_URL, "https://gitlab.example"),
            (ENV_GITLAB_TOKEN, "token"),
            (ENV_GITLAB_SSL_VERIFY, "false"),
            (ENV_GITLAB_TIMEOUT, "90"),
            (ENV_GITLAB_HTTP_PROXY, "http://proxy.example:8080"),
            (ENV_GITLAB_HTTPS_PROXY, "https://secure-proxy.example:8443"),
            (ENV_GITLAB_NO_PROXY, " gitlab.example, localhost "),
            (ENV_GITLAB_CUSTOM_HEADERS, "X-Team=platform"),
            (ENV_GITLAB_CLIENT_CERT, "/tmp/gitlab.crt"),
            (ENV_GITLAB_CLIENT_KEY, "/tmp/gitlab.key"),
        ])
        .unwrap()
        .unwrap();

        assert!(!config.ssl_verify);
        assert_eq!(config.timeout_seconds, 90);
        assert_eq!(
            config.proxy.http_proxy.as_deref(),
            Some("http://proxy.example:8080")
        );
        assert_eq!(
            config.proxy.https_proxy.as_deref(),
            Some("https://secure-proxy.example:8443")
        );
        assert_eq!(
            config.proxy.no_proxy.as_deref(),
            Some("gitlab.example,localhost")
        );
        assert!(format!("{:?}", config.custom_headers).contains("x-team"));
        assert_eq!(
            config.mtls.as_ref().unwrap().cert_path,
            PathBuf::from("/tmp/gitlab.crt")
        );
        assert_eq!(
            config.mtls.as_ref().unwrap().key_path,
            PathBuf::from("/tmp/gitlab.key")
        );
    }

    #[test]
    fn rejects_invalid_url_timeout_and_reserved_private_token_custom_header() {
        assert!(matches!(
            config_from_pairs(&[
                (ENV_GITLAB_URL, "ftp://gitlab.example"),
                (ENV_GITLAB_TOKEN, "t")
            ]),
            Err(ConfigError::InvalidGitlabUrl {
                variable: ENV_GITLAB_URL
            })
        ));
        assert!(matches!(
            config_from_pairs(&[
                (ENV_GITLAB_URL, "https://gitlab.example"),
                (ENV_GITLAB_TOKEN, "t"),
                (ENV_GITLAB_TIMEOUT, "0"),
            ]),
            Err(ConfigError::InvalidGitlabTimeout {
                variable: ENV_GITLAB_TIMEOUT,
                ..
            })
        ));
        assert!(matches!(
            config_from_pairs(&[
                (ENV_GITLAB_URL, "https://gitlab.example"),
                (ENV_GITLAB_TOKEN, "t"),
                (ENV_GITLAB_CUSTOM_HEADERS, "Private-Token=secret"),
            ]),
            Err(ConfigError::ReservedCustomHeader {
                variable: ENV_GITLAB_CUSTOM_HEADERS,
                ..
            })
        ));
    }
}
