use std::collections::BTreeSet;

use crate::tool_registry::{
    DEFAULT_TOOL_PROFILE, all_toolsets, default_toolsets, toolsets_for_profile,
};
use crate::{
    confluence::config::ConfluenceConfig, error::ConfigError, gitlab::config::GitlabConfig,
    jira::config::JiraConfig,
};

pub const DEFAULT_HTTP_HOST: &str = "127.0.0.1";
pub const DEFAULT_HTTP_PORT: u16 = 8000;
pub const DEFAULT_HTTP_PATH: &str = "/mcp";

pub const ENV_MCP_TOOL_PROFILE: &str = "MCP_TOOL_PROFILE";
pub const ENV_MCP_ENABLED_TOOLS: &str = "MCP_ENABLED_TOOLS";
pub const ENV_MCP_DISABLED_TOOLS: &str = "MCP_DISABLED_TOOLS";
pub const ENV_MCP_TOOLSETS: &str = "MCP_TOOLSETS";
pub const ENV_HTTP_HOST: &str = "MCP_HTTP_HOST";
pub const ENV_HTTP_PORT: &str = "MCP_HTTP_PORT";
pub const ENV_HTTP_PATH: &str = "MCP_HTTP_PATH";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub mcp_enabled_tools: Option<BTreeSet<String>>,
    pub mcp_disabled_tools: BTreeSet<String>,
    pub mcp_enabled_toolsets: BTreeSet<String>,
    pub jira: Option<JiraConfig>,
    pub confluence: Option<ConfluenceConfig>,
    pub gitlab: Option<GitlabConfig>,
    pub http: HttpConfig,
}

impl RuntimeConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_var_provider(|key| std::env::var(key), HttpConfigSource::Default)
    }

    pub fn from_env_for_cli() -> Result<Self, ConfigError> {
        Self::from_var_provider_for_cli(|key| std::env::var(key))
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
        let mcp_tool_profile =
            parse_mcp_tool_profile(get_var(ENV_MCP_TOOL_PROFILE).ok().as_deref())?;
        let mcp_enabled_tools = parse_enabled_tools(get_var(ENV_MCP_ENABLED_TOOLS).ok().as_deref());
        let mcp_disabled_tools = parse_csv_names(get_var(ENV_MCP_DISABLED_TOOLS).ok().as_deref());
        let mcp_enabled_toolsets =
            parse_mcp_toolsets(&mcp_tool_profile, get_var(ENV_MCP_TOOLSETS).ok().as_deref())?;
        let jira = JiraConfig::from_var_provider(&mut get_var)?;
        let confluence = ConfluenceConfig::from_var_provider(&mut get_var)?;
        let gitlab = GitlabConfig::from_var_provider(&mut get_var)?;
        let http = match http_source {
            HttpConfigSource::Default => HttpConfig::default(),
            HttpConfigSource::Env(overrides) => {
                HttpConfig::from_var_provider(&mut get_var, overrides)?
            }
        };

        Ok(Self {
            mcp_enabled_tools,
            mcp_disabled_tools,
            mcp_enabled_toolsets,
            jira,
            confluence,
            gitlab,
            http,
        })
    }

    pub fn from_var_provider_for_cli<F, E>(mut get_var: F) -> Result<Self, ConfigError>
    where
        F: FnMut(&str) -> Result<String, E>,
    {
        let jira = JiraConfig::from_var_provider(&mut get_var)?;
        let confluence = ConfluenceConfig::from_var_provider(&mut get_var)?;
        let gitlab = GitlabConfig::from_var_provider(&mut get_var)?;

        Ok(Self {
            mcp_enabled_tools: None,
            mcp_disabled_tools: BTreeSet::new(),
            mcp_enabled_toolsets: default_toolsets(),
            jira,
            confluence,
            gitlab,
            http: HttpConfig::default(),
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
            mcp_enabled_tools: None,
            mcp_disabled_tools: BTreeSet::new(),
            mcp_enabled_toolsets: default_toolsets(),
            jira: None,
            confluence: None,
            gitlab: None,
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

fn parse_mcp_tool_profile(value: Option<&str>) -> Result<String, ConfigError> {
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
            variable: ENV_MCP_TOOL_PROFILE,
            value: profile,
        });
    }

    Ok(profile)
}

fn parse_mcp_toolsets(profile: &str, value: Option<&str>) -> Result<BTreeSet<String>, ConfigError> {
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
                    variable: ENV_MCP_TOOLSETS,
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
        atlassian::compat::{
            ENV_ATLASSIAN_API_TOKEN, ENV_ATLASSIAN_PERSONAL_TOKEN, ENV_ATLASSIAN_USERNAME,
        },
        confluence::config::{
            ConfluenceDeployment, ENV_CONFLUENCE_API_TOKEN, ENV_CONFLUENCE_PERSONAL_TOKEN,
            ENV_CONFLUENCE_URL, ENV_CONFLUENCE_USERNAME,
        },
        jira::config::{ENV_JIRA_PERSONAL_TOKEN, ENV_JIRA_URL, JiraDeployment},
        upstream::auth::UpstreamAuth,
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

    fn cli_config_from_pairs(pairs: &[(&str, &str)]) -> Result<RuntimeConfig, ConfigError> {
        let vars: BTreeMap<String, String> = pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();

        RuntimeConfig::from_var_provider_for_cli(|key| vars.get(key).cloned().ok_or(()))
    }

    #[test]
    fn runtime_config_defaults_to_control_plane_values() {
        let config = config_from_pairs(&[]).unwrap();

        assert_eq!(config.mcp_enabled_tools, None);
        assert!(config.mcp_disabled_tools.is_empty());
        assert_eq!(config.mcp_enabled_toolsets, default_toolsets());
        assert_eq!(config.jira, None);
        assert_eq!(config.confluence, None);
        assert_eq!(config.http, HttpConfig::default());
    }

    #[test]
    fn mcp_enabled_and_disabled_tools_are_trimmed_and_empty_values_are_ignored() {
        let config =
            config_from_pairs(&[(ENV_MCP_ENABLED_TOOLS, " jira_search_issues, , get_issue ")])
                .unwrap();
        let tools = config.mcp_enabled_tools.unwrap();

        assert!(tools.contains("jira_search_issues"));
        assert!(tools.contains("get_issue"));
        assert_eq!(tools.len(), 2);

        assert_eq!(
            config_from_pairs(&[(ENV_MCP_ENABLED_TOOLS, " , ")])
                .unwrap()
                .mcp_enabled_tools,
            None
        );

        let disabled =
            config_from_pairs(&[(ENV_MCP_DISABLED_TOOLS, " jira_delete_issue, , typo ")])
                .unwrap()
                .mcp_disabled_tools;
        assert!(disabled.contains("jira_delete_issue"));
        assert!(disabled.contains("typo"));
        assert_eq!(disabled.len(), 2);
    }

    #[test]
    fn legacy_unprefixed_tool_controls_are_ignored() {
        let config = config_from_pairs(&[
            ("TOOL_PROFILE", "full"),
            ("TOOLSETS", "all"),
            ("ENABLED_TOOLS", "jira_delete_issue"),
            ("DISABLED_TOOLS", "jira_get_issue"),
        ])
        .unwrap();

        assert_eq!(config.mcp_enabled_tools, None);
        assert!(config.mcp_disabled_tools.is_empty());
        assert_eq!(config.mcp_enabled_toolsets, default_toolsets());
    }

    #[test]
    fn cli_config_ignores_mcp_tool_controls() {
        let config = cli_config_from_pairs(&[
            (ENV_MCP_TOOL_PROFILE, "not-a-profile"),
            (ENV_MCP_TOOLSETS, "not-a-toolset"),
            (ENV_MCP_ENABLED_TOOLS, "jira_delete_issue"),
            (ENV_MCP_DISABLED_TOOLS, "jira_get_issue"),
        ])
        .unwrap();

        assert_eq!(config.mcp_enabled_tools, None);
        assert!(config.mcp_disabled_tools.is_empty());
        assert_eq!(config.mcp_enabled_toolsets, default_toolsets());
    }

    #[test]
    fn mcp_tool_profile_defaults_to_basic_and_can_select_higher_profiles() {
        assert_eq!(
            config_from_pairs(&[]).unwrap().mcp_enabled_toolsets,
            default_toolsets()
        );
        assert_eq!(
            config_from_pairs(&[(ENV_MCP_TOOL_PROFILE, "developer")])
                .unwrap()
                .mcp_enabled_toolsets
                .contains("jira_sprint_membership_write"),
            true
        );
        assert_eq!(
            config_from_pairs(&[(ENV_MCP_TOOL_PROFILE, "manager")])
                .unwrap()
                .mcp_enabled_toolsets
                .contains("jira_issues_delete"),
            true
        );
        assert_eq!(
            config_from_pairs(&[(ENV_MCP_TOOL_PROFILE, "full")])
                .unwrap()
                .mcp_enabled_toolsets,
            all_toolsets()
        );
    }

    #[test]
    fn unknown_mcp_tool_profile_is_rejected() {
        let error = config_from_pairs(&[(ENV_MCP_TOOL_PROFILE, "admin")]).unwrap_err();

        assert_eq!(
            error,
            ConfigError::InvalidToolProfile {
                variable: ENV_MCP_TOOL_PROFILE,
                value: "admin".to_string(),
            }
        );
    }

    #[test]
    fn mcp_toolsets_are_additive_to_profile_and_all_explicitly_enables_everything() {
        let config = config_from_pairs(&[(ENV_MCP_TOOLSETS, "jira_sprints_write")]).unwrap();
        let mut expected = default_toolsets();
        expected.insert("jira_sprints_write".to_string());

        assert_eq!(config.mcp_enabled_toolsets, expected);

        assert_eq!(
            config_from_pairs(&[(ENV_MCP_TOOLSETS, "all")])
                .unwrap()
                .mcp_enabled_toolsets,
            all_toolsets()
        );
    }

    #[test]
    fn custom_mcp_profile_starts_empty_and_unknown_toolsets_are_rejected() {
        let config = config_from_pairs(&[(ENV_MCP_TOOL_PROFILE, "custom")]).unwrap();
        assert!(config.mcp_enabled_toolsets.is_empty());

        let error = config_from_pairs(&[
            (ENV_MCP_TOOL_PROFILE, "custom"),
            (ENV_MCP_TOOLSETS, "typo_name"),
        ])
        .unwrap_err();

        assert_eq!(
            error,
            ConfigError::InvalidToolset {
                variable: ENV_MCP_TOOLSETS,
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
        ])
        .unwrap();
        let jira = config.jira.unwrap();

        assert_eq!(jira.base_url, "https://jira.example");
        assert_eq!(jira.deployment, JiraDeployment::ServerDataCenter);
        assert_eq!(
            jira.auth,
            UpstreamAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            }
        );
        assert_eq!(config.confluence, None);
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
            UpstreamAuth::Pat {
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
            UpstreamAuth::Basic {
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
            UpstreamAuth::Basic {
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
