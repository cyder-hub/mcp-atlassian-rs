use std::{collections::BTreeSet, net::IpAddr};

use crate::atlassian::security::BLOCKED_HOSTNAMES;
use crate::tool_registry::{
    DEFAULT_TOOL_PROFILE, all_toolsets, default_toolsets, toolsets_for_profile,
};
use crate::{
    atlassian::compat::ENV_ATLASSIAN_OAUTH_ENABLE, confluence::config::ConfluenceConfig,
    error::ConfigError, jira::config::JiraConfig,
};

#[cfg(test)]
pub use crate::confluence::config::ENV_CONFLUENCE_URL;

pub const DEFAULT_HTTP_HOST: &str = "127.0.0.1";
pub const DEFAULT_HTTP_PORT: u16 = 8000;
pub const DEFAULT_HTTP_PATH: &str = "/mcp";

pub const ENV_TOOL_PROFILE: &str = "TOOL_PROFILE";
pub const ENV_ENABLED_TOOLS: &str = "ENABLED_TOOLS";
pub const ENV_DISABLED_TOOLS: &str = "DISABLED_TOOLS";
pub const ENV_TOOLSETS: &str = "TOOLSETS";
pub const ENV_ATLASSIAN_OAUTH_CLOUD_ID: &str = "ATLASSIAN_OAUTH_CLOUD_ID";
pub const ENV_HTTP_HOST: &str = "MCP_HTTP_HOST";
pub const ENV_HTTP_PORT: &str = "MCP_HTTP_PORT";
pub const ENV_HTTP_PATH: &str = "MCP_HTTP_PATH";
pub use crate::atlassian::request_auth::ENV_IGNORE_HEADER_AUTH;
pub use crate::atlassian::security::ENV_ALLOWED_URL_DOMAINS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub enabled_tools: Option<BTreeSet<String>>,
    pub disabled_tools: BTreeSet<String>,
    pub enabled_toolsets: BTreeSet<String>,
    pub jira: Option<JiraConfig>,
    pub confluence: Option<ConfluenceConfig>,
    pub atlassian_oauth_cloud_id: Option<String>,
    pub atlassian_oauth_enabled: bool,
    pub allowed_url_domains: Option<Vec<String>>,
    pub ignore_header_auth: bool,
    pub http: HttpConfig,
}

impl RuntimeConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_var_provider(|key| std::env::var(key), HttpConfigSource::Default)
    }

    pub fn from_env_with_http_overrides(
        http_overrides: HttpConfigOverrides,
    ) -> Result<Self, ConfigError> {
        Self::from_var_provider(
            |key| std::env::var(key),
            HttpConfigSource::Env(http_overrides),
        )
    }

    pub fn from_var_provider<F, E>(
        mut get_var: F,
        http_source: HttpConfigSource,
    ) -> Result<Self, ConfigError>
    where
        F: FnMut(&str) -> Result<String, E>,
    {
        let tool_profile = parse_tool_profile(get_var(ENV_TOOL_PROFILE).ok().as_deref())?;
        let enabled_tools = parse_enabled_tools(get_var(ENV_ENABLED_TOOLS).ok().as_deref());
        let disabled_tools = parse_csv_names(get_var(ENV_DISABLED_TOOLS).ok().as_deref());
        let enabled_toolsets =
            parse_toolsets(&tool_profile, get_var(ENV_TOOLSETS).ok().as_deref())?;
        let jira = JiraConfig::from_var_provider(&mut get_var)?;
        let confluence = ConfluenceConfig::from_var_provider(&mut get_var)?;
        let atlassian_oauth_cloud_id =
            parse_optional_string(get_var(ENV_ATLASSIAN_OAUTH_CLOUD_ID).ok());
        let atlassian_oauth_enabled =
            parse_extended_truthy(get_var(ENV_ATLASSIAN_OAUTH_ENABLE).ok().as_deref());
        let allowed_url_domains = parse_allowed_url_domains(get_var(ENV_ALLOWED_URL_DOMAINS).ok())?;
        let ignore_header_auth =
            parse_extended_truthy(get_var(ENV_IGNORE_HEADER_AUTH).ok().as_deref());
        let http = match http_source {
            HttpConfigSource::Default => HttpConfig::default(),
            HttpConfigSource::Env(overrides) => {
                HttpConfig::from_var_provider(&mut get_var, overrides)?
            }
        };

        Ok(Self {
            enabled_tools,
            disabled_tools,
            enabled_toolsets,
            jira,
            confluence,
            atlassian_oauth_cloud_id,
            atlassian_oauth_enabled,
            allowed_url_domains,
            ignore_header_auth,
            http,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpConfigSource {
    Default,
    Env(HttpConfigOverrides),
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            enabled_tools: None,
            disabled_tools: BTreeSet::new(),
            enabled_toolsets: default_toolsets(),
            jira: None,
            confluence: None,
            atlassian_oauth_cloud_id: None,
            atlassian_oauth_enabled: false,
            allowed_url_domains: None,
            ignore_header_auth: false,
            http: HttpConfig::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpConfig {
    pub host: String,
    pub port: u16,
    pub path: String,
}

impl HttpConfig {
    pub fn from_var_provider<F, E>(
        mut get_var: F,
        overrides: HttpConfigOverrides,
    ) -> Result<Self, ConfigError>
    where
        F: FnMut(&str) -> Result<String, E>,
    {
        let host = overrides
            .host
            .or_else(|| get_var(ENV_HTTP_HOST).ok())
            .and_then(non_empty_trimmed)
            .unwrap_or_else(|| DEFAULT_HTTP_HOST.to_string());

        let port = match overrides.port {
            Some(port) => port,
            None => parse_optional_http_port(get_var(ENV_HTTP_PORT).ok())?,
        };

        let path = normalize_http_path(overrides.path.or_else(|| get_var(ENV_HTTP_PATH).ok()));

        Ok(Self { host, port, path })
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_HTTP_HOST.to_string(),
            port: DEFAULT_HTTP_PORT,
            path: DEFAULT_HTTP_PATH.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HttpConfigOverrides {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub path: Option<String>,
}

pub(crate) fn parse_extended_truthy(value: Option<&str>) -> bool {
    matches!(
        value.map(|value| value.trim().to_ascii_lowercase()),
        Some(value)
            if matches!(
                value.as_str(),
                "true" | "1" | "yes" | "y" | "on"
            )
    )
}

fn parse_enabled_tools(value: Option<&str>) -> Option<BTreeSet<String>> {
    let tools = parse_csv_names(value);
    if tools.is_empty() { None } else { Some(tools) }
}

fn parse_tool_profile(value: Option<&str>) -> Result<String, ConfigError> {
    let profile = value
        .and_then(|value| {
            let value = value.trim();
            if value.is_empty() {
                None
            } else {
                Some(value.to_ascii_lowercase())
            }
        })
        .unwrap_or_else(|| DEFAULT_TOOL_PROFILE.to_string());

    if toolsets_for_profile(&profile).is_none() {
        return Err(ConfigError::InvalidToolProfile {
            variable: ENV_TOOL_PROFILE,
            value: profile,
        });
    }

    Ok(profile)
}

fn parse_toolsets(profile: &str, value: Option<&str>) -> Result<BTreeSet<String>, ConfigError> {
    let mut result = profile_toolsets(profile);
    let tokens = parse_csv_tokens(value);
    let all = all_toolsets();

    for token in tokens {
        let normalized_token = token.to_ascii_lowercase();
        match normalized_token.as_str() {
            "all" => return Ok(all),
            _ if all.contains(&normalized_token) => {
                result.insert(normalized_token);
            }
            _ => {
                return Err(ConfigError::InvalidToolset {
                    variable: ENV_TOOLSETS,
                    value: token,
                });
            }
        }
    }

    Ok(result)
}

fn profile_toolsets(profile: &str) -> BTreeSet<String> {
    toolsets_for_profile(profile)
        .expect("tool profile was validated before resolving toolsets")
        .iter()
        .map(|toolset| (*toolset).to_string())
        .collect()
}

fn parse_csv_names(value: Option<&str>) -> BTreeSet<String> {
    parse_csv_tokens(value).into_iter().collect()
}

fn parse_csv_tokens(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn parse_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(non_empty_trimmed)
}

fn non_empty_trimmed(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn parse_optional_http_port(value: Option<String>) -> Result<u16, ConfigError> {
    let Some(value) = value.and_then(non_empty_trimmed) else {
        return Ok(DEFAULT_HTTP_PORT);
    };

    value.parse().map_err(|_| ConfigError::InvalidHttpPort {
        variable: ENV_HTTP_PORT,
        value,
    })
}

fn parse_allowed_url_domains(value: Option<String>) -> Result<Option<Vec<String>>, ConfigError> {
    let Some(value) = value.and_then(non_empty_trimmed) else {
        return Ok(None);
    };

    let mut domains = parse_csv_tokens(Some(&value));
    if domains.is_empty() {
        return Ok(None);
    }

    let mut normalized = BTreeSet::new();
    for domain in domains.drain(..) {
        let domain = normalize_allowed_domain_token(&domain)?;
        normalized.insert(domain);
    }

    if normalized.is_empty() {
        Ok(None)
    } else {
        Ok(Some(normalized.into_iter().collect()))
    }
}

fn normalize_allowed_domain_token(value: &str) -> Result<String, ConfigError> {
    let normalized = value
        .trim()
        .trim_start_matches('.')
        .trim_end_matches('.')
        .to_ascii_lowercase();

    if normalized.is_empty()
        || normalized.contains("://")
        || normalized.contains('/')
        || normalized.contains(':')
        || normalized.parse::<IpAddr>().is_ok()
        || BLOCKED_HOSTNAMES.contains(&normalized.as_str())
        || !normalized.split('.').all(is_valid_domain_label)
    {
        return Err(ConfigError::InvalidAllowedUrlDomain {
            variable: ENV_ALLOWED_URL_DOMAINS,
            value: value.to_string(),
        });
    }

    Ok(normalized)
}

fn is_valid_domain_label(label: &str) -> bool {
    !label.is_empty()
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
}

fn normalize_http_path(value: Option<String>) -> String {
    let Some(value) = value.and_then(non_empty_trimmed) else {
        return DEFAULT_HTTP_PATH.to_string();
    };

    if value.starts_with('/') {
        value
    } else {
        format!("/{value}")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        atlassian::{
            auth::AtlassianAuth,
            compat::{
                ENV_ATLASSIAN_API_TOKEN, ENV_ATLASSIAN_PERSONAL_TOKEN, ENV_ATLASSIAN_USERNAME,
            },
        },
        confluence::config::{
            ConfluenceDeployment, ENV_CONFLUENCE_API_TOKEN, ENV_CONFLUENCE_PERSONAL_TOKEN,
            ENV_CONFLUENCE_URL, ENV_CONFLUENCE_USERNAME,
        },
        jira::config::{ENV_JIRA_PERSONAL_TOKEN, ENV_JIRA_URL, JiraDeployment},
    };

    use super::*;

    fn config_from_pairs(pairs: &[(&str, &str)]) -> Result<RuntimeConfig, ConfigError> {
        let vars: BTreeMap<String, String> = pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();

        RuntimeConfig::from_var_provider(
            |key| vars.get(key).cloned().ok_or(()),
            HttpConfigSource::Default,
        )
    }

    fn streamhttp_config_from_pairs(pairs: &[(&str, &str)]) -> Result<RuntimeConfig, ConfigError> {
        let vars: BTreeMap<String, String> = pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();

        RuntimeConfig::from_var_provider(
            |key| vars.get(key).cloned().ok_or(()),
            HttpConfigSource::Env(HttpConfigOverrides::default()),
        )
    }

    #[test]
    fn runtime_config_defaults_to_control_plane_values() {
        let config = config_from_pairs(&[]).unwrap();

        assert_eq!(config.enabled_tools, None);
        assert!(config.disabled_tools.is_empty());
        assert_eq!(config.enabled_toolsets, default_toolsets());
        assert_eq!(config.jira, None);
        assert_eq!(config.confluence, None);
        assert_eq!(config.atlassian_oauth_cloud_id, None);
        assert!(!config.atlassian_oauth_enabled);
        assert_eq!(config.allowed_url_domains, None);
        assert!(!config.ignore_header_auth);
        assert_eq!(config.http, HttpConfig::default());
    }

    #[test]
    fn enabled_and_disabled_tools_are_trimmed_and_empty_values_are_ignored() {
        let config =
            config_from_pairs(&[(ENV_ENABLED_TOOLS, " jira_search, , get_issue ")]).unwrap();
        let tools = config.enabled_tools.unwrap();

        assert!(tools.contains("jira_search"));
        assert!(tools.contains("get_issue"));
        assert_eq!(tools.len(), 2);

        assert_eq!(
            config_from_pairs(&[(ENV_ENABLED_TOOLS, " , ")])
                .unwrap()
                .enabled_tools,
            None
        );

        let disabled = config_from_pairs(&[(ENV_DISABLED_TOOLS, " jira_delete_issue, , typo ")])
            .unwrap()
            .disabled_tools;
        assert!(disabled.contains("jira_delete_issue"));
        assert!(disabled.contains("typo"));
        assert_eq!(disabled.len(), 2);
    }

    #[test]
    fn tool_profile_defaults_to_basic_and_can_select_higher_profiles() {
        assert_eq!(
            config_from_pairs(&[]).unwrap().enabled_toolsets,
            default_toolsets()
        );
        assert_eq!(
            config_from_pairs(&[(ENV_TOOL_PROFILE, "developer")])
                .unwrap()
                .enabled_toolsets
                .contains("jira_sprint_planning"),
            true
        );
        assert_eq!(
            config_from_pairs(&[(ENV_TOOL_PROFILE, "manager")])
                .unwrap()
                .enabled_toolsets
                .contains("jira_issue_delete"),
            true
        );
        assert_eq!(
            config_from_pairs(&[(ENV_TOOL_PROFILE, "full")])
                .unwrap()
                .enabled_toolsets,
            all_toolsets()
        );
    }

    #[test]
    fn unknown_tool_profile_is_rejected() {
        let error = config_from_pairs(&[(ENV_TOOL_PROFILE, "admin")]).unwrap_err();

        assert_eq!(
            error,
            ConfigError::InvalidToolProfile {
                variable: ENV_TOOL_PROFILE,
                value: "admin".to_string(),
            }
        );
    }

    #[test]
    fn toolsets_are_additive_to_profile_and_all_explicitly_enables_everything() {
        let config = config_from_pairs(&[(ENV_TOOLSETS, "jira_sprint_manage")]).unwrap();
        let mut expected = default_toolsets();
        expected.insert("jira_sprint_manage".to_string());

        assert_eq!(config.enabled_toolsets, expected);

        assert_eq!(
            config_from_pairs(&[(ENV_TOOLSETS, "all")])
                .unwrap()
                .enabled_toolsets,
            all_toolsets()
        );
    }

    #[test]
    fn custom_profile_starts_empty_and_unknown_toolsets_are_rejected() {
        let config = config_from_pairs(&[(ENV_TOOL_PROFILE, "custom")]).unwrap();
        assert!(config.enabled_toolsets.is_empty());

        let error = config_from_pairs(&[(ENV_TOOL_PROFILE, "custom"), (ENV_TOOLSETS, "typo_name")])
            .unwrap_err();

        assert_eq!(
            error,
            ConfigError::InvalidToolset {
                variable: ENV_TOOLSETS,
                value: "typo_name".to_string(),
            }
        );
    }

    #[test]
    fn service_configs_are_trimmed_and_empty_values_are_ignored() {
        let config = config_from_pairs(&[
            (ENV_JIRA_URL, " https://jira.example "),
            (ENV_JIRA_PERSONAL_TOKEN, "test-pat-value"),
            (ENV_CONFLUENCE_URL, " "),
            (ENV_ATLASSIAN_OAUTH_CLOUD_ID, " cloud-123 "),
        ])
        .unwrap();
        let jira = config.jira.unwrap();

        assert_eq!(jira.base_url, "https://jira.example");
        assert_eq!(jira.deployment, JiraDeployment::ServerDataCenter);
        assert_eq!(
            jira.auth,
            AtlassianAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            }
        );
        assert_eq!(config.confluence, None);
        assert_eq!(
            config.atlassian_oauth_cloud_id.as_deref(),
            Some("cloud-123")
        );
        assert!(!config.atlassian_oauth_enabled);
    }

    #[test]
    fn shared_pat_configures_only_services_with_urls() {
        let config = config_from_pairs(&[
            (ENV_JIRA_URL, "https://jira.example"),
            (ENV_ATLASSIAN_PERSONAL_TOKEN, "shared-pat-value"),
        ])
        .unwrap();
        let jira = config.jira.unwrap();

        assert_eq!(
            jira.auth,
            AtlassianAuth::Pat {
                personal_token: "shared-pat-value".to_string(),
            }
        );
        assert_eq!(config.confluence, None);
    }

    #[test]
    fn shared_basic_auth_configures_only_services_with_urls() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, "https://example.atlassian.net/wiki"),
            (ENV_ATLASSIAN_USERNAME, "user@example.com"),
            (ENV_ATLASSIAN_API_TOKEN, "shared-api-token"),
        ])
        .unwrap();
        let confluence = config.confluence.unwrap();

        assert_eq!(config.jira, None);
        assert_eq!(
            confluence.auth,
            AtlassianAuth::Basic {
                username: "user@example.com".to_string(),
                api_token: "shared-api-token".to_string(),
            }
        );
    }

    #[test]
    fn runtime_config_reads_typed_confluence_config() {
        let config = config_from_pairs(&[
            (ENV_CONFLUENCE_URL, " https://example.atlassian.net/wiki/ "),
            (ENV_CONFLUENCE_USERNAME, "user@example.com"),
            (ENV_CONFLUENCE_API_TOKEN, "test-api-token"),
        ])
        .unwrap();
        let confluence = config.confluence.unwrap();

        assert_eq!(confluence.base_url, "https://example.atlassian.net/wiki");
        assert_eq!(confluence.deployment, ConfluenceDeployment::Cloud);
        assert_eq!(
            confluence.auth,
            AtlassianAuth::Basic {
                username: "user@example.com".to_string(),
                api_token: "test-api-token".to_string(),
            }
        );
    }

    #[test]
    fn runtime_config_rejects_confluence_credentials_without_url() {
        let error =
            config_from_pairs(&[(ENV_CONFLUENCE_PERSONAL_TOKEN, "test-pat-value")]).unwrap_err();

        assert_eq!(
            error,
            ConfigError::MissingConfluenceUrl {
                credential_variables: vec![ENV_CONFLUENCE_PERSONAL_TOKEN],
            }
        );
    }

    #[test]
    fn atlassian_oauth_cloud_id_is_trimmed_and_optional() {
        assert_eq!(
            config_from_pairs(&[(ENV_ATLASSIAN_OAUTH_CLOUD_ID, " cloud-123 ")])
                .unwrap()
                .atlassian_oauth_cloud_id
                .as_deref(),
            Some("cloud-123")
        );
        assert_eq!(
            config_from_pairs(&[(ENV_ATLASSIAN_OAUTH_CLOUD_ID, " ")])
                .unwrap()
                .atlassian_oauth_cloud_id,
            None
        );
    }

    #[test]
    fn atlassian_oauth_enable_uses_extended_truthy_values() {
        for value in ["true", "1", "yes", "y", "on", "TRUE", " On "] {
            let config = config_from_pairs(&[(ENV_ATLASSIAN_OAUTH_ENABLE, value)]).unwrap();
            assert!(
                config.atlassian_oauth_enabled,
                "value `{value}` should enable OAuth/BYOT request mode"
            );
        }

        for value in ["false", "0", "no", "off", ""] {
            let config = config_from_pairs(&[(ENV_ATLASSIAN_OAUTH_ENABLE, value)]).unwrap();
            assert!(
                !config.atlassian_oauth_enabled,
                "value `{value}` should leave OAuth/BYOT request mode disabled"
            );
        }
    }

    #[test]
    fn allowed_url_domains_are_trimmed_normalized_and_deduplicated() {
        let config = config_from_pairs(&[(
            ENV_ALLOWED_URL_DOMAINS,
            " Example.Atlassian.Net, .atlassian.net. ,example.atlassian.net ",
        )])
        .unwrap();

        assert_eq!(
            config.allowed_url_domains,
            Some(vec![
                "atlassian.net".to_string(),
                "example.atlassian.net".to_string()
            ])
        );
    }

    #[test]
    fn allowed_url_domains_empty_values_are_unset() {
        let config = config_from_pairs(&[(ENV_ALLOWED_URL_DOMAINS, " , ")]).unwrap();

        assert_eq!(config.allowed_url_domains, None);
    }

    #[test]
    fn allowed_url_domains_reject_unsafe_or_malformed_values() {
        for value in [
            "https://jira.example",
            "127.0.0.1",
            "localhost",
            "metadata.google.internal",
            "bad/domain",
            "-bad.example",
            "bad-.example",
            "bad..example",
        ] {
            let error = config_from_pairs(&[(ENV_ALLOWED_URL_DOMAINS, value)]).unwrap_err();

            assert_eq!(
                error,
                ConfigError::InvalidAllowedUrlDomain {
                    variable: ENV_ALLOWED_URL_DOMAINS,
                    value: value.to_string(),
                }
            );
        }
    }

    #[test]
    fn ignore_header_auth_uses_extended_truthy_values() {
        for value in ["true", "1", "yes", "y", "on", "TRUE", " On "] {
            let config = config_from_pairs(&[(ENV_IGNORE_HEADER_AUTH, value)]).unwrap();
            assert!(
                config.ignore_header_auth,
                "value `{value}` should be truthy"
            );
        }

        for value in ["false", "0", "no", "off", ""] {
            let config = config_from_pairs(&[(ENV_IGNORE_HEADER_AUTH, value)]).unwrap();
            assert!(
                !config.ignore_header_auth,
                "value `{value}` should be false"
            );
        }
    }

    #[test]
    fn http_config_reads_env_defaults_and_normalizes_path() {
        let config = streamhttp_config_from_pairs(&[
            (ENV_HTTP_HOST, " 0.0.0.0 "),
            (ENV_HTTP_PORT, "9000"),
            (ENV_HTTP_PATH, "mcp-alt"),
        ])
        .unwrap();

        assert_eq!(
            config.http,
            HttpConfig {
                host: "0.0.0.0".to_string(),
                port: 9000,
                path: "/mcp-alt".to_string(),
            }
        );
    }

    #[test]
    fn http_config_empty_values_fall_back_to_defaults() {
        let config = streamhttp_config_from_pairs(&[
            (ENV_HTTP_HOST, " "),
            (ENV_HTTP_PORT, " "),
            (ENV_HTTP_PATH, " "),
        ])
        .unwrap();

        assert_eq!(config.http, HttpConfig::default());
    }

    #[test]
    fn http_config_rejects_invalid_env_port() {
        let error = streamhttp_config_from_pairs(&[(ENV_HTTP_PORT, "bad-port")]).unwrap_err();

        assert_eq!(
            error,
            ConfigError::InvalidHttpPort {
                variable: ENV_HTTP_PORT,
                value: "bad-port".to_string(),
            }
        );
    }

    #[test]
    fn default_runtime_config_ignores_http_environment() {
        let config = config_from_pairs(&[
            (ENV_HTTP_HOST, "0.0.0.0"),
            (ENV_HTTP_PORT, "bad-port"),
            (ENV_HTTP_PATH, "env-mcp"),
        ])
        .unwrap();

        assert_eq!(config.http, HttpConfig::default());
    }

    #[test]
    fn http_overrides_take_precedence_over_environment() {
        let vars = BTreeMap::from([
            (ENV_HTTP_HOST.to_string(), "0.0.0.0".to_string()),
            (ENV_HTTP_PORT.to_string(), "bad-port".to_string()),
            (ENV_HTTP_PATH.to_string(), "env-mcp".to_string()),
        ]);
        let config = RuntimeConfig::from_var_provider(
            |key| vars.get(key).cloned().ok_or(()),
            HttpConfigSource::Env(HttpConfigOverrides {
                host: Some("127.0.0.2".to_string()),
                port: Some(9001),
                path: Some("cli-mcp".to_string()),
            }),
        )
        .unwrap();

        assert_eq!(
            config.http,
            HttpConfig {
                host: "127.0.0.2".to_string(),
                port: 9001,
                path: "/cli-mcp".to_string(),
            }
        );
    }
}
