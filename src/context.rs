use std::collections::BTreeSet;

use crate::{config::RuntimeConfig, jira::config::JiraConfig};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppContext {
    read_only: bool,
    enabled_tools: Option<BTreeSet<String>>,
    enabled_toolsets: BTreeSet<String>,
    jira: Option<JiraConfig>,
    atlassian_oauth_cloud_id: Option<String>,
    service_availability: ServiceAvailability,
}

impl AppContext {
    pub fn from_config(config: &RuntimeConfig) -> Self {
        Self {
            read_only: config.read_only,
            enabled_tools: config.enabled_tools.clone(),
            enabled_toolsets: config.enabled_toolsets.clone(),
            jira: config.jira.clone(),
            atlassian_oauth_cloud_id: config.atlassian_oauth_cloud_id.clone(),
            service_availability: ServiceAvailability::from_config(config),
        }
    }

    pub fn read_only(&self) -> bool {
        self.read_only
    }

    pub fn enabled_tools(&self) -> Option<&BTreeSet<String>> {
        self.enabled_tools.as_ref()
    }

    pub fn enabled_toolsets(&self) -> &BTreeSet<String> {
        &self.enabled_toolsets
    }

    pub fn service_availability(&self) -> &ServiceAvailability {
        &self.service_availability
    }

    pub fn atlassian_oauth_cloud_id(&self) -> Option<&str> {
        self.atlassian_oauth_cloud_id.as_deref()
    }

    #[allow(dead_code)]
    pub fn jira_config(&self) -> Option<&JiraConfig> {
        self.jira.as_ref()
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
}

impl ServiceAvailability {
    pub fn from_config(config: &RuntimeConfig) -> Self {
        Self {
            jira: config
                .jira
                .as_ref()
                .is_some_and(JiraConfig::is_auth_configured),
            confluence: config.confluence_url.is_some(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{
        atlassian::auth::AtlassianAuth,
        config::{HttpConfig, RuntimeConfig},
        jira::config::{JiraConfig, JiraDeployment},
        tool_registry::default_toolsets,
    };

    use super::*;

    #[test]
    fn default_context_has_no_service_availability() {
        let context = AppContext::default();

        assert!(!context.read_only());
        assert_eq!(context.enabled_tools(), None);
        assert_eq!(context.jira_config(), None);
        assert_eq!(
            context.service_availability(),
            &ServiceAvailability {
                jira: false,
                confluence: false,
            }
        );
    }

    #[test]
    fn context_preserves_control_plane_config() {
        let enabled_tools = BTreeSet::from(["migration_status".to_string()]);
        let enabled_toolsets = default_toolsets();
        let jira = jira_config();
        let config = RuntimeConfig {
            read_only: true,
            enabled_tools: Some(enabled_tools.clone()),
            enabled_toolsets: enabled_toolsets.clone(),
            jira: Some(jira.clone()),
            confluence_url: Some("https://confluence.example".to_string()),
            atlassian_oauth_cloud_id: Some("cloud-123".to_string()),
            http: HttpConfig::default(),
        };

        let context = AppContext::from_config(&config);

        assert!(context.read_only());
        assert_eq!(context.enabled_tools(), Some(&enabled_tools));
        assert_eq!(context.enabled_toolsets(), &enabled_toolsets);
        assert_eq!(context.jira_config(), Some(&jira));
        assert_eq!(context.atlassian_oauth_cloud_id(), Some("cloud-123"));
        assert_eq!(
            context.service_availability(),
            &ServiceAvailability {
                jira: true,
                confluence: true,
            }
        );
    }

    fn jira_config() -> JiraConfig {
        JiraConfig {
            base_url: "https://jira.example".to_string(),
            deployment: JiraDeployment::ServerDataCenter,
            auth: AtlassianAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            },
            ssl_verify: true,
            projects_filter: BTreeSet::new(),
            timeout_seconds: 75,
        }
    }
}
