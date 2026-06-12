use std::collections::BTreeSet;

#[cfg(test)]
use crate::upstream::{custom_headers::CustomHeaders, proxy::ProxyConfig};
use crate::{
    config::RuntimeConfig, confluence::config::ConfluenceConfig, gitlab::config::GitlabConfig,
    jira::config::JiraConfig,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppContext {
    mcp_enabled_tools: Option<BTreeSet<String>>,
    mcp_disabled_tools: BTreeSet<String>,
    mcp_enabled_toolsets: BTreeSet<String>,
    jira: Option<JiraConfig>,
    confluence: Option<ConfluenceConfig>,
    gitlab: Option<GitlabConfig>,
    service_availability: ServiceAvailability,
}

impl AppContext {
    pub fn from_config(config: &RuntimeConfig) -> Self {
        Self {
            mcp_enabled_tools: config.mcp_enabled_tools.clone(),
            mcp_disabled_tools: config.mcp_disabled_tools.clone(),
            mcp_enabled_toolsets: config.mcp_enabled_toolsets.clone(),
            jira: config.jira.clone(),
            confluence: config.confluence.clone(),
            gitlab: config.gitlab.clone(),
            service_availability: ServiceAvailability::from_config(config),
        }
    }

    pub fn mcp_enabled_tools(&self) -> Option<&BTreeSet<String>> {
        self.mcp_enabled_tools.as_ref()
    }

    pub fn mcp_disabled_tools(&self) -> &BTreeSet<String> {
        &self.mcp_disabled_tools
    }

    pub fn mcp_enabled_toolsets(&self) -> &BTreeSet<String> {
        &self.mcp_enabled_toolsets
    }

    pub fn service_availability(&self) -> &ServiceAvailability {
        &self.service_availability
    }

    #[allow(dead_code)]
    pub fn jira_config(&self) -> Option<&JiraConfig> {
        self.jira.as_ref()
    }

    #[allow(dead_code)]
    pub fn confluence_config(&self) -> Option<&ConfluenceConfig> {
        self.confluence.as_ref()
    }

    #[allow(dead_code)]
    pub fn gitlab_config(&self) -> Option<&GitlabConfig> {
        self.gitlab.as_ref()
    }
}

impl Default for AppContext {
    fn default() -> Self {
        Self::from_config(&RuntimeConfig::default())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceAvailability {
    pub jira: bool,
    pub confluence: bool,
    pub gitlab: bool,
}

impl ServiceAvailability {
    pub fn from_config(config: &RuntimeConfig) -> Self {
        Self::from_service_configs(&config.jira, &config.confluence, &config.gitlab)
    }

    fn from_service_configs(
        jira: &Option<JiraConfig>,
        confluence: &Option<ConfluenceConfig>,
        gitlab: &Option<GitlabConfig>,
    ) -> Self {
        Self {
            jira: jira.as_ref().is_some_and(JiraConfig::is_auth_configured),
            confluence: confluence
                .as_ref()
                .is_some_and(ConfluenceConfig::is_auth_configured),
            gitlab: gitlab
                .as_ref()
                .is_some_and(GitlabConfig::is_auth_configured),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{
        config::{HttpConfig, RuntimeConfig},
        confluence::config::{ConfluenceConfig, ConfluenceDeployment},
        jira::{
            config::{JiraConfig, JiraDeployment},
            tools,
        },
        tool_registry::default_toolsets,
        upstream::auth::UpstreamAuth,
    };

    use super::*;

    #[test]
    fn default_context_has_no_service_availability() {
        let context = AppContext::default();

        assert_eq!(context.mcp_enabled_tools(), None);
        assert!(context.mcp_disabled_tools().is_empty());
        assert_eq!(context.jira_config(), None);
        assert_eq!(context.confluence_config(), None);
        assert_eq!(context.gitlab_config(), None);
        assert_eq!(
            context.service_availability(),
            &ServiceAvailability {
                jira: false,
                confluence: false,
                gitlab: false,
            }
        );
    }

    #[test]
    fn context_preserves_control_plane_config() {
        let enabled_tools = BTreeSet::from([tools::JIRA_GET_ISSUE_TOOL_NAME.to_string()]);
        let disabled_tools = BTreeSet::from([tools::JIRA_DELETE_ISSUE_TOOL_NAME.to_string()]);
        let enabled_toolsets = default_toolsets();
        let jira = jira_config();
        let confluence = confluence_config();
        let config = RuntimeConfig {
            mcp_enabled_tools: Some(enabled_tools.clone()),
            mcp_disabled_tools: disabled_tools.clone(),
            mcp_enabled_toolsets: enabled_toolsets.clone(),
            jira: Some(jira.clone()),
            confluence: Some(confluence.clone()),
            gitlab: None,
            http: HttpConfig::default(),
        };

        let context = AppContext::from_config(&config);

        assert_eq!(context.mcp_enabled_tools(), Some(&enabled_tools));
        assert_eq!(context.mcp_disabled_tools(), &disabled_tools);
        assert_eq!(context.mcp_enabled_toolsets(), &enabled_toolsets);
        assert_eq!(context.jira_config(), Some(&jira));
        assert_eq!(context.confluence_config(), Some(&confluence));
        assert_eq!(context.gitlab_config(), None);
        assert_eq!(
            context.service_availability(),
            &ServiceAvailability {
                jira: true,
                confluence: true,
                gitlab: false,
            }
        );
    }

    #[test]
    fn confluence_availability_requires_typed_config_with_auth() {
        let config = RuntimeConfig {
            confluence: Some(confluence_config()),
            ..RuntimeConfig::default()
        };

        assert_eq!(
            ServiceAvailability::from_config(&config),
            ServiceAvailability {
                jira: false,
                confluence: true,
                gitlab: false,
            }
        );
    }

    fn jira_config() -> JiraConfig {
        JiraConfig {
            base_url: "https://jira.example".to_string(),
            deployment: JiraDeployment::ServerDataCenter,
            auth: UpstreamAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            },
            ssl_verify: true,
            proxy: ProxyConfig::default(),
            custom_headers: CustomHeaders::default(),
            mtls: None,
            projects_filter: BTreeSet::new(),
            timeout_seconds: 75,
        }
    }

    fn confluence_config() -> ConfluenceConfig {
        ConfluenceConfig {
            base_url: "https://confluence.example".to_string(),
            deployment: ConfluenceDeployment::ServerDataCenter,
            auth: UpstreamAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            },
            ssl_verify: true,
            proxy: ProxyConfig::default(),
            custom_headers: CustomHeaders::default(),
            mtls: None,
            spaces_filter: BTreeSet::new(),
            timeout_seconds: 75,
        }
    }
}
