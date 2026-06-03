use std::collections::BTreeSet;

use reqwest::Url;

use crate::{atlassian::auth::AtlassianAuth, error::ConfigError};

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
    pub ssl_verify: bool,
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

        let Some(base_url) = base_url else {
            let credential_variables = present_variables([
                (ENV_JIRA_USERNAME, username.as_ref()),
                (ENV_JIRA_API_TOKEN, api_token.as_ref()),
                (ENV_JIRA_PERSONAL_TOKEN, personal_token.as_ref()),
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
        let auth = match deployment {
            JiraDeployment::Cloud => {
                let missing_variables = missing_variables([
                    (ENV_JIRA_USERNAME, username.as_ref()),
                    (ENV_JIRA_API_TOKEN, api_token.as_ref()),
                ]);

                if !missing_variables.is_empty() {
                    return Err(ConfigError::MissingJiraCloudCredentials { missing_variables });
                }

                AtlassianAuth::Basic {
                    username: username.expect("missing variables were checked"),
                    api_token: api_token.expect("missing variables were checked"),
                }
            }
            JiraDeployment::ServerDataCenter => {
                let Some(personal_token) = personal_token else {
                    return Err(ConfigError::MissingJiraPersonalToken {
                        variable: ENV_JIRA_PERSONAL_TOKEN,
                    });
                };

                AtlassianAuth::Pat { personal_token }
            }
        };

        Ok(Some(Self {
            base_url: normalize_base_url(parsed_url),
            deployment,
            auth,
            ssl_verify: parse_ssl_verify(optional_var(get_var, ENV_JIRA_SSL_VERIFY).as_deref()),
            projects_filter: parse_project_filter(optional_var(get_var, ENV_JIRA_PROJECTS_FILTER)),
            timeout_seconds: parse_timeout_seconds(optional_var(get_var, ENV_JIRA_TIMEOUT))?,
        }))
    }

    pub fn is_auth_configured(&self) -> bool {
        match (&self.deployment, &self.auth) {
            (JiraDeployment::Cloud, AtlassianAuth::Basic { .. }) => true,
            (JiraDeployment::ServerDataCenter, AtlassianAuth::Pat { .. }) => true,
            _ => false,
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
        assert!(config.ssl_verify);
        assert!(config.projects_filter.is_empty());
        assert_eq!(config.timeout_seconds, DEFAULT_JIRA_TIMEOUT_SECONDS);
        assert!(config.is_auth_configured());
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
