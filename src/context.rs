use std::collections::BTreeSet;

#[cfg(test)]
use crate::atlassian::{custom_headers::CustomHeaders, proxy::ProxyConfig};
use crate::{
    atlassian::request_auth::RequestAuthContext,
    config::RuntimeConfig,
    confluence::config::{
        ConfluenceConfig, ConfluenceDeployment, DEFAULT_CONFLUENCE_TIMEOUT_SECONDS,
    },
    jira::config::{DEFAULT_JIRA_TIMEOUT_SECONDS, JiraConfig, JiraDeployment},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppContext {
    enabled_tools: Option<BTreeSet<String>>,
    disabled_tools: BTreeSet<String>,
    enabled_toolsets: BTreeSet<String>,
    jira: Option<JiraConfig>,
    confluence: Option<ConfluenceConfig>,
    atlassian_oauth_cloud_id: Option<String>,
    atlassian_oauth_enabled: bool,
    allowed_url_domains: Option<Vec<String>>,
    ignore_header_auth: bool,
    service_availability: ServiceAvailability,
}

impl AppContext {
    pub fn from_config(config: &RuntimeConfig) -> Self {
        Self {
            enabled_tools: config.enabled_tools.clone(),
            disabled_tools: config.disabled_tools.clone(),
            enabled_toolsets: config.enabled_toolsets.clone(),
            jira: config.jira.clone(),
            confluence: config.confluence.clone(),
            atlassian_oauth_cloud_id: config.atlassian_oauth_cloud_id.clone(),
            atlassian_oauth_enabled: config.atlassian_oauth_enabled,
            allowed_url_domains: config.allowed_url_domains.clone(),
            ignore_header_auth: config.ignore_header_auth,
            service_availability: ServiceAvailability::from_config(config),
        }
    }

    pub fn with_request_auth(&self, request_auth: &RequestAuthContext) -> Self {
        let mut context = self.clone();

        if let Some(auth) = request_auth.authorization.clone() {
            let oauth_cloud_id = request_auth
                .cloud_id
                .clone()
                .or_else(|| context.atlassian_oauth_cloud_id.clone());
            if let Some(jira) = context.jira.as_mut() {
                *jira = jira.with_auth_override(auth.clone(), oauth_cloud_id.clone());
            }
            if let Some(confluence) = context.confluence.as_mut() {
                *confluence = confluence.with_auth_override(auth, oauth_cloud_id);
            }
        }

        if let Some(jira) = request_auth.jira.as_ref() {
            context.jira = Some(jira_config_from_override(jira, context.jira.as_ref()));
        }
        if let Some(confluence) = request_auth.confluence.as_ref() {
            context.confluence = Some(confluence_config_from_override(
                confluence,
                context.confluence.as_ref(),
            ));
        }
        if let Some(cloud_id) = request_auth.cloud_id.as_ref() {
            context.atlassian_oauth_cloud_id = Some(cloud_id.clone());
        }

        context.service_availability =
            ServiceAvailability::from_service_configs(&context.jira, &context.confluence);
        context
    }

    pub fn enabled_tools(&self) -> Option<&BTreeSet<String>> {
        self.enabled_tools.as_ref()
    }

    pub fn disabled_tools(&self) -> &BTreeSet<String> {
        &self.disabled_tools
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

    pub fn atlassian_oauth_enabled(&self) -> bool {
        self.atlassian_oauth_enabled
    }

    pub fn allowed_url_domains(&self) -> Option<&[String]> {
        self.allowed_url_domains.as_deref()
    }

    pub fn ignore_header_auth(&self) -> bool {
        self.ignore_header_auth
    }

    #[allow(dead_code)]
    pub fn jira_config(&self) -> Option<&JiraConfig> {
        self.jira.as_ref()
    }

    #[allow(dead_code)]
    pub fn confluence_config(&self) -> Option<&ConfluenceConfig> {
        self.confluence.as_ref()
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
        Self::from_service_configs(&config.jira, &config.confluence)
    }

    fn from_service_configs(
        jira: &Option<JiraConfig>,
        confluence: &Option<ConfluenceConfig>,
    ) -> Self {
        Self {
            jira: jira.as_ref().is_some_and(JiraConfig::is_auth_configured),
            confluence: confluence
                .as_ref()
                .is_some_and(ConfluenceConfig::is_auth_configured),
        }
    }
}

fn jira_config_from_override(
    override_auth: &crate::atlassian::request_auth::ServiceAuthOverride,
    existing: Option<&JiraConfig>,
) -> JiraConfig {
    JiraConfig {
        base_url: override_auth.base_url.clone(),
        deployment: jira_deployment_from_base_url(&override_auth.base_url),
        auth: override_auth.auth.clone(),
        oauth_cloud_id: None,
        ssl_verify: existing.is_none_or(|config| config.ssl_verify),
        proxy: existing
            .map(|config| config.proxy.clone())
            .unwrap_or_default(),
        custom_headers: existing
            .map(|config| config.custom_headers.clone())
            .unwrap_or_default(),
        mtls: existing.and_then(|config| config.mtls.clone()),
        projects_filter: existing
            .map(|config| config.projects_filter.clone())
            .unwrap_or_default(),
        timeout_seconds: existing.map_or(DEFAULT_JIRA_TIMEOUT_SECONDS, |config| {
            config.timeout_seconds
        }),
    }
}

fn confluence_config_from_override(
    override_auth: &crate::atlassian::request_auth::ServiceAuthOverride,
    existing: Option<&ConfluenceConfig>,
) -> ConfluenceConfig {
    ConfluenceConfig {
        base_url: override_auth.base_url.clone(),
        deployment: confluence_deployment_from_base_url(&override_auth.base_url),
        auth: override_auth.auth.clone(),
        oauth_cloud_id: None,
        ssl_verify: existing.is_none_or(|config| config.ssl_verify),
        proxy: existing
            .map(|config| config.proxy.clone())
            .unwrap_or_default(),
        custom_headers: existing
            .map(|config| config.custom_headers.clone())
            .unwrap_or_default(),
        mtls: existing.and_then(|config| config.mtls.clone()),
        spaces_filter: existing
            .map(|config| config.spaces_filter.clone())
            .unwrap_or_default(),
        timeout_seconds: existing.map_or(DEFAULT_CONFLUENCE_TIMEOUT_SECONDS, |config| {
            config.timeout_seconds
        }),
    }
}

fn jira_deployment_from_base_url(base_url: &str) -> JiraDeployment {
    if base_url
        .to_ascii_lowercase()
        .split('/')
        .nth(2)
        .is_some_and(|host| host.ends_with(".atlassian.net"))
    {
        JiraDeployment::Cloud
    } else {
        JiraDeployment::ServerDataCenter
    }
}

fn confluence_deployment_from_base_url(base_url: &str) -> ConfluenceDeployment {
    if base_url
        .to_ascii_lowercase()
        .split('/')
        .nth(2)
        .is_some_and(|host| host.ends_with(".atlassian.net"))
    {
        ConfluenceDeployment::Cloud
    } else {
        ConfluenceDeployment::ServerDataCenter
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{
        atlassian::auth::AtlassianAuth,
        config::{HttpConfig, RuntimeConfig},
        confluence::config::{ConfluenceConfig, ConfluenceDeployment},
        jira::{
            config::{JiraConfig, JiraDeployment},
            tools,
        },
        tool_registry::default_toolsets,
    };

    use super::*;

    #[test]
    fn default_context_has_no_service_availability() {
        let context = AppContext::default();

        assert_eq!(context.enabled_tools(), None);
        assert!(context.disabled_tools().is_empty());
        assert_eq!(context.jira_config(), None);
        assert_eq!(context.confluence_config(), None);
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
        let enabled_tools = BTreeSet::from([tools::JIRA_GET_ISSUE_TOOL_NAME.to_string()]);
        let disabled_tools = BTreeSet::from([tools::JIRA_DELETE_ISSUE_TOOL_NAME.to_string()]);
        let enabled_toolsets = default_toolsets();
        let jira = jira_config();
        let confluence = confluence_config();
        let config = RuntimeConfig {
            enabled_tools: Some(enabled_tools.clone()),
            disabled_tools: disabled_tools.clone(),
            enabled_toolsets: enabled_toolsets.clone(),
            jira: Some(jira.clone()),
            confluence: Some(confluence.clone()),
            atlassian_oauth_cloud_id: Some("cloud-123".to_string()),
            atlassian_oauth_enabled: true,
            allowed_url_domains: Some(vec!["atlassian.net".to_string()]),
            ignore_header_auth: true,
            http: HttpConfig::default(),
        };

        let context = AppContext::from_config(&config);

        assert_eq!(context.enabled_tools(), Some(&enabled_tools));
        assert_eq!(context.disabled_tools(), &disabled_tools);
        assert_eq!(context.enabled_toolsets(), &enabled_toolsets);
        assert_eq!(context.jira_config(), Some(&jira));
        assert_eq!(context.confluence_config(), Some(&confluence));
        assert_eq!(context.atlassian_oauth_cloud_id(), Some("cloud-123"));
        assert!(context.atlassian_oauth_enabled());
        assert_eq!(
            context.allowed_url_domains(),
            Some(&["atlassian.net".to_string()][..])
        );
        assert!(context.ignore_header_auth());
        assert_eq!(
            context.service_availability(),
            &ServiceAvailability {
                jira: true,
                confluence: true,
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
            }
        );
    }

    #[test]
    fn request_auth_overrides_existing_service_credentials_without_mutating_global_context() {
        use crate::atlassian::request_auth::parse_request_auth_headers_with_resolver;
        use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

        let context = AppContext::from_config(&RuntimeConfig {
            jira: Some(jira_config()),
            confluence: Some(confluence_config()),
            ..RuntimeConfig::default()
        });
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer request-token"),
        );
        let request_auth = parse_request_auth_headers_with_resolver(&headers, false, None, |_| {
            Ok(vec!["8.8.8.8".parse().unwrap()])
        })
        .unwrap();

        let scoped = context.with_request_auth(&request_auth);

        assert!(matches!(
            scoped.jira_config().unwrap().auth,
            AtlassianAuth::Pat { ref personal_token } if personal_token == "request-token"
        ));
        assert!(matches!(
            scoped.confluence_config().unwrap().auth,
            AtlassianAuth::Pat { ref personal_token } if personal_token == "request-token"
        ));
        assert!(matches!(
            context.jira_config().unwrap().auth,
            AtlassianAuth::Pat { ref personal_token } if personal_token == "test-pat-value"
        ));
    }

    #[test]
    fn request_auth_cloud_oauth_rewrites_service_configs_without_mutating_global_context() {
        use crate::atlassian::request_auth::{
            HEADER_ATLASSIAN_CLOUD_ID, parse_request_auth_headers_with_resolver,
        };
        use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

        let jira = JiraConfig {
            base_url: "https://example.atlassian.net".to_string(),
            deployment: JiraDeployment::Cloud,
            auth: AtlassianAuth::Basic {
                username: "user@example.com".to_string(),
                api_token: "jira-api-token".to_string(),
            },
            oauth_cloud_id: None,
            ssl_verify: true,
            proxy: ProxyConfig::default(),
            custom_headers: CustomHeaders::default(),
            mtls: None,
            projects_filter: BTreeSet::new(),
            timeout_seconds: 75,
        };
        let confluence = ConfluenceConfig {
            base_url: "https://example.atlassian.net/wiki".to_string(),
            deployment: ConfluenceDeployment::Cloud,
            auth: AtlassianAuth::Basic {
                username: "user@example.com".to_string(),
                api_token: "confluence-api-token".to_string(),
            },
            oauth_cloud_id: None,
            ssl_verify: true,
            proxy: ProxyConfig::default(),
            custom_headers: CustomHeaders::default(),
            mtls: None,
            spaces_filter: BTreeSet::new(),
            timeout_seconds: 75,
        };
        let context = AppContext::from_config(&RuntimeConfig {
            jira: Some(jira),
            confluence: Some(confluence),
            ..RuntimeConfig::default()
        });
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer request-access-token"),
        );
        headers.insert(
            HEADER_ATLASSIAN_CLOUD_ID,
            HeaderValue::from_static("cloud-123"),
        );
        let request_auth = parse_request_auth_headers_with_resolver(&headers, false, None, |_| {
            Ok(vec!["8.8.8.8".parse().unwrap()])
        })
        .unwrap();

        let scoped = context.with_request_auth(&request_auth);

        assert_eq!(
            scoped.jira_config().unwrap().auth,
            AtlassianAuth::OAuthAccessToken {
                access_token: "request-access-token".to_string(),
            }
        );
        assert_eq!(
            scoped.jira_config().unwrap().base_url,
            "https://api.atlassian.com/ex/jira/cloud-123"
        );
        assert_eq!(
            scoped.jira_config().unwrap().oauth_cloud_id.as_deref(),
            Some("cloud-123")
        );
        assert_eq!(
            scoped.confluence_config().unwrap().base_url,
            "https://api.atlassian.com/ex/confluence/cloud-123/wiki"
        );
        assert_eq!(
            scoped
                .confluence_config()
                .unwrap()
                .oauth_cloud_id
                .as_deref(),
            Some("cloud-123")
        );
        assert_eq!(
            context.jira_config().unwrap().base_url,
            "https://example.atlassian.net"
        );
        assert_eq!(
            context.confluence_config().unwrap().base_url,
            "https://example.atlassian.net/wiki"
        );
    }

    #[test]
    fn request_auth_service_headers_create_scoped_service_configs() {
        use crate::atlassian::request_auth::{
            HEADER_CONFLUENCE_PERSONAL_TOKEN, HEADER_CONFLUENCE_URL, HEADER_JIRA_PERSONAL_TOKEN,
            HEADER_JIRA_URL, parse_request_auth_headers_with_resolver,
        };
        use reqwest::header::{HeaderMap, HeaderValue};

        let context = AppContext::default();
        let allowed_domains = vec!["atlassian.net".to_string()];
        let mut headers = HeaderMap::new();
        headers.insert(
            HEADER_JIRA_URL,
            HeaderValue::from_static("https://example.atlassian.net"),
        );
        headers.insert(
            HEADER_JIRA_PERSONAL_TOKEN,
            HeaderValue::from_static("jira-request-token"),
        );
        headers.insert(
            HEADER_CONFLUENCE_URL,
            HeaderValue::from_static("https://example.atlassian.net/wiki"),
        );
        headers.insert(
            HEADER_CONFLUENCE_PERSONAL_TOKEN,
            HeaderValue::from_static("conf-request-token"),
        );
        let request_auth = parse_request_auth_headers_with_resolver(
            &headers,
            false,
            Some(&allowed_domains),
            |_| Ok(vec!["8.8.8.8".parse().unwrap()]),
        )
        .unwrap();

        let scoped = context.with_request_auth(&request_auth);

        assert_eq!(
            scoped.jira_config().unwrap().base_url,
            "https://example.atlassian.net"
        );
        assert_eq!(
            scoped.confluence_config().unwrap().base_url,
            "https://example.atlassian.net/wiki"
        );
        assert_eq!(
            scoped.service_availability(),
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
            oauth_cloud_id: None,
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
            auth: AtlassianAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            },
            oauth_cloud_id: None,
            ssl_verify: true,
            proxy: ProxyConfig::default(),
            custom_headers: CustomHeaders::default(),
            mtls: None,
            spaces_filter: BTreeSet::new(),
            timeout_seconds: 75,
        }
    }
}
