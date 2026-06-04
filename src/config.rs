use std::collections::BTreeSet;

use crate::tool_registry::{all_toolsets, default_toolsets};
use crate::{error::ConfigError, jira::config::JiraConfig};

pub const DEFAULT_HTTP_HOST: &str = "127.0.0.1";
pub const DEFAULT_HTTP_PORT: u16 = 8000;
pub const DEFAULT_HTTP_PATH: &str = "/mcp";

pub const ENV_READ_ONLY_MODE: &str = "READ_ONLY_MODE";
pub const ENV_ENABLED_TOOLS: &str = "ENABLED_TOOLS";
pub const ENV_TOOLSETS: &str = "TOOLSETS";
pub const ENV_CONFLUENCE_URL: &str = "CONFLUENCE_URL";
pub const ENV_ATLASSIAN_OAUTH_CLOUD_ID: &str = "ATLASSIAN_OAUTH_CLOUD_ID";
pub const ENV_HTTP_HOST: &str = "MCP_HTTP_HOST";
pub const ENV_HTTP_PORT: &str = "MCP_HTTP_PORT";
pub const ENV_HTTP_PATH: &str = "MCP_HTTP_PATH";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub read_only: bool,
    pub enabled_tools: Option<BTreeSet<String>>,
    pub enabled_toolsets: BTreeSet<String>,
    pub jira: Option<JiraConfig>,
    pub confluence_url: Option<String>,
    pub atlassian_oauth_cloud_id: Option<String>,
    pub http: HttpConfig,
}

impl RuntimeConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_var_provider(|key| std::env::var(key), HttpConfigOverrides::default())
    }

    pub fn from_env_with_http_overrides(
        http_overrides: HttpConfigOverrides,
    ) -> Result<Self, ConfigError> {
        Self::from_var_provider(|key| std::env::var(key), http_overrides)
    }

    pub fn from_var_provider<F, E>(
        mut get_var: F,
        http_overrides: HttpConfigOverrides,
    ) -> Result<Self, ConfigError>
    where
        F: FnMut(&str) -> Result<String, E>,
    {
        let read_only = parse_extended_truthy(get_var(ENV_READ_ONLY_MODE).ok().as_deref());
        let enabled_tools = parse_enabled_tools(get_var(ENV_ENABLED_TOOLS).ok().as_deref());
        let enabled_toolsets = parse_toolsets(get_var(ENV_TOOLSETS).ok().as_deref());
        let jira = JiraConfig::from_var_provider(&mut get_var)?;
        let confluence_url = parse_optional_string(get_var(ENV_CONFLUENCE_URL).ok());
        let atlassian_oauth_cloud_id =
            parse_optional_string(get_var(ENV_ATLASSIAN_OAUTH_CLOUD_ID).ok());
        let http = HttpConfig::from_var_provider(&mut get_var, http_overrides)?;

        Ok(Self {
            read_only,
            enabled_tools,
            enabled_toolsets,
            jira,
            confluence_url,
            atlassian_oauth_cloud_id,
            http,
        })
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            read_only: false,
            enabled_tools: None,
            enabled_toolsets: all_toolsets(),
            jira: None,
            confluence_url: None,
            atlassian_oauth_cloud_id: None,
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

fn parse_extended_truthy(value: Option<&str>) -> bool {
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

fn parse_toolsets(value: Option<&str>) -> BTreeSet<String> {
    let tokens = parse_csv_tokens(value);
    if tokens.is_empty() {
        return all_toolsets();
    }

    let all = all_toolsets();
    let defaults = default_toolsets();
    let mut result = BTreeSet::new();

    for token in tokens {
        match token.to_ascii_lowercase().as_str() {
            "all" => return all,
            "default" => result.extend(defaults.iter().cloned()),
            _ if all.contains(&token) => {
                result.insert(token);
            }
            _ => {}
        }
    }

    result
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
        atlassian::auth::AtlassianAuth,
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
            HttpConfigOverrides::default(),
        )
    }

    #[test]
    fn runtime_config_defaults_to_stage_one_control_plane_values() {
        let config = config_from_pairs(&[]).unwrap();

        assert!(!config.read_only);
        assert_eq!(config.enabled_tools, None);
        assert_eq!(config.enabled_toolsets, all_toolsets());
        assert_eq!(config.jira, None);
        assert_eq!(config.confluence_url, None);
        assert_eq!(config.atlassian_oauth_cloud_id, None);
        assert_eq!(config.http, HttpConfig::default());
    }

    #[test]
    fn read_only_mode_uses_extended_truthy_values() {
        for value in ["true", "1", "yes", "y", "on", "TRUE", " On "] {
            let config = config_from_pairs(&[(ENV_READ_ONLY_MODE, value)]).unwrap();
            assert!(config.read_only, "value `{value}` should be truthy");
        }

        for value in ["false", "0", "no", "off", ""] {
            let config = config_from_pairs(&[(ENV_READ_ONLY_MODE, value)]).unwrap();
            assert!(!config.read_only, "value `{value}` should be false");
        }
    }

    #[test]
    fn enabled_tools_are_trimmed_and_empty_values_mean_all() {
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
    }

    #[test]
    fn toolsets_unset_empty_or_all_enable_all_baseline_toolsets() {
        assert_eq!(
            config_from_pairs(&[]).unwrap().enabled_toolsets,
            all_toolsets()
        );
        assert_eq!(
            config_from_pairs(&[(ENV_TOOLSETS, " , ")])
                .unwrap()
                .enabled_toolsets,
            all_toolsets()
        );
        assert_eq!(
            config_from_pairs(&[(ENV_TOOLSETS, "all")])
                .unwrap()
                .enabled_toolsets,
            all_toolsets()
        );
    }

    #[test]
    fn toolsets_default_keyword_enables_python_default_toolsets() {
        let config = config_from_pairs(&[(ENV_TOOLSETS, "default,jira_agile")]).unwrap();
        let mut expected = default_toolsets();
        expected.insert("jira_agile".to_string());

        assert_eq!(config.enabled_toolsets, expected);
    }

    #[test]
    fn toolsets_unknown_only_fails_closed() {
        let config = config_from_pairs(&[(ENV_TOOLSETS, "typo_name")]).unwrap();

        assert!(config.enabled_toolsets.is_empty());
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
        assert_eq!(config.confluence_url, None);
        assert_eq!(
            config.atlassian_oauth_cloud_id.as_deref(),
            Some("cloud-123")
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
    fn http_config_reads_env_defaults_and_normalizes_path() {
        let config = config_from_pairs(&[
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
        let config = config_from_pairs(&[
            (ENV_HTTP_HOST, " "),
            (ENV_HTTP_PORT, " "),
            (ENV_HTTP_PATH, " "),
        ])
        .unwrap();

        assert_eq!(config.http, HttpConfig::default());
    }

    #[test]
    fn http_config_rejects_invalid_env_port() {
        let error = config_from_pairs(&[(ENV_HTTP_PORT, "bad-port")]).unwrap_err();

        assert_eq!(
            error,
            ConfigError::InvalidHttpPort {
                variable: ENV_HTTP_PORT,
                value: "bad-port".to_string(),
            }
        );
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
            HttpConfigOverrides {
                host: Some("127.0.0.2".to_string()),
                port: Some(9001),
                path: Some("cli-mcp".to_string()),
            },
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
