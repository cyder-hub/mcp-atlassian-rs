use std::fmt::{Debug, Formatter};

use reqwest::Url;

use crate::{
    atlassian::compat::{ENV_HTTP_PROXY, ENV_HTTPS_PROXY, ENV_NO_PROXY},
    error::ConfigError,
};

#[derive(Clone, Default, PartialEq, Eq)]
pub struct ProxyConfig {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub no_proxy: Option<String>,
}

impl ProxyConfig {
    pub fn from_var_provider<F, E>(
        get_var: &mut F,
        service_http_proxy: &'static str,
        service_https_proxy: &'static str,
        service_no_proxy: &'static str,
    ) -> Result<Self, ConfigError>
    where
        F: FnMut(&str) -> Result<String, E>,
    {
        let http_proxy = first_proxy_var(get_var, service_http_proxy, None, ENV_HTTP_PROXY)
            .map(|(variable, value)| validate_proxy_url(variable, value))
            .transpose()?;
        let https_proxy = first_proxy_var(get_var, service_https_proxy, None, ENV_HTTPS_PROXY)
            .map(|(variable, value)| validate_proxy_url(variable, value))
            .transpose()?;
        let no_proxy = first_proxy_var(get_var, service_no_proxy, None, ENV_NO_PROXY).and_then(
            |(_, value)| {
                let normalized = normalize_no_proxy(&value);
                (!normalized.is_empty()).then_some(normalized)
            },
        );

        Ok(Self {
            http_proxy,
            https_proxy,
            no_proxy,
        })
    }

    pub fn from_var_provider_with_fallback<F, E>(
        get_var: &mut F,
        service_http_proxy: &'static str,
        service_https_proxy: &'static str,
        service_no_proxy: &'static str,
        fallback_http_proxy: &'static str,
        fallback_https_proxy: &'static str,
        fallback_no_proxy: &'static str,
    ) -> Result<Self, ConfigError>
    where
        F: FnMut(&str) -> Result<String, E>,
    {
        let http_proxy = first_proxy_var(
            get_var,
            service_http_proxy,
            Some(fallback_http_proxy),
            ENV_HTTP_PROXY,
        )
        .map(|(variable, value)| validate_proxy_url(variable, value))
        .transpose()?;
        let https_proxy = first_proxy_var(
            get_var,
            service_https_proxy,
            Some(fallback_https_proxy),
            ENV_HTTPS_PROXY,
        )
        .map(|(variable, value)| validate_proxy_url(variable, value))
        .transpose()?;
        let no_proxy = first_proxy_var(
            get_var,
            service_no_proxy,
            Some(fallback_no_proxy),
            ENV_NO_PROXY,
        )
        .and_then(|(_, value)| {
            let normalized = normalize_no_proxy(&value);
            (!normalized.is_empty()).then_some(normalized)
        });

        Ok(Self {
            http_proxy,
            https_proxy,
            no_proxy,
        })
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.http_proxy.is_none() && self.https_proxy.is_none() && self.no_proxy.is_none()
    }
}

impl Debug for ProxyConfig {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProxyConfig")
            .field(
                "http_proxy",
                &self.http_proxy.as_deref().map(redact_proxy_url),
            )
            .field(
                "https_proxy",
                &self.https_proxy.as_deref().map(redact_proxy_url),
            )
            .field("no_proxy", &self.no_proxy)
            .finish()
    }
}

fn first_proxy_var<F, E>(
    get_var: &mut F,
    service_variable: &'static str,
    fallback_variable: Option<&'static str>,
    global_variable: &'static str,
) -> Option<(&'static str, String)>
where
    F: FnMut(&str) -> Result<String, E>,
{
    optional_var(get_var, service_variable)
        .map(|value| (service_variable, value))
        .or_else(|| {
            fallback_variable
                .and_then(|variable| optional_var(get_var, variable).map(|value| (variable, value)))
        })
        .or_else(|| optional_var(get_var, global_variable).map(|value| (global_variable, value)))
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

fn validate_proxy_url(variable: &'static str, value: String) -> Result<String, ConfigError> {
    let url = Url::parse(&value).map_err(|_| ConfigError::InvalidProxyUrl { variable })?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(ConfigError::InvalidProxyUrl { variable });
    }

    Ok(value)
}

fn normalize_no_proxy(value: &str) -> String {
    value
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>()
        .join(",")
}

fn redact_proxy_url(value: &str) -> String {
    let Ok(mut url) = Url::parse(value) else {
        return "<invalid proxy URL>".to_string();
    };

    let mut redacted = format!("{}://", url.scheme());
    if !url.username().is_empty() || url.password().is_some() {
        redacted.push_str("<redacted>@");
    }
    if let Some(host) = url.host_str() {
        redacted.push_str(host);
    }
    if let Some(port) = url.port() {
        redacted.push(':');
        redacted.push_str(&port.to_string());
    }
    let path = url.path();
    if !path.is_empty() && path != "/" {
        redacted.push_str(path);
    }
    url.set_query(None);
    url.set_fragment(None);
    redacted
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::atlassian::compat::{
        ENV_ATLASSIAN_HTTP_PROXY, ENV_ATLASSIAN_HTTPS_PROXY, ENV_ATLASSIAN_NO_PROXY,
        ENV_JIRA_HTTP_PROXY, ENV_JIRA_HTTPS_PROXY, ENV_JIRA_NO_PROXY,
    };

    use super::*;

    fn proxy_from_pairs(pairs: &[(&str, &str)]) -> Result<ProxyConfig, ConfigError> {
        let vars: BTreeMap<String, String> = pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();

        ProxyConfig::from_var_provider_with_fallback(
            &mut |key| vars.get(key).cloned().ok_or(()),
            ENV_JIRA_HTTP_PROXY,
            ENV_JIRA_HTTPS_PROXY,
            ENV_JIRA_NO_PROXY,
            ENV_ATLASSIAN_HTTP_PROXY,
            ENV_ATLASSIAN_HTTPS_PROXY,
            ENV_ATLASSIAN_NO_PROXY,
        )
    }

    #[test]
    fn proxy_config_uses_service_specific_values_before_global() {
        let proxy = proxy_from_pairs(&[
            (ENV_HTTP_PROXY, "http://global-proxy.example:8080"),
            (
                ENV_ATLASSIAN_HTTP_PROXY,
                "http://atlassian-proxy.example:8080",
            ),
            (ENV_JIRA_HTTP_PROXY, "http://jira-proxy.example:8080"),
            (ENV_HTTPS_PROXY, "http://global-secure-proxy.example:8080"),
            (
                ENV_ATLASSIAN_HTTPS_PROXY,
                "http://atlassian-secure-proxy.example:8080",
            ),
            (
                ENV_JIRA_HTTPS_PROXY,
                "https://jira-secure-proxy.example:8443",
            ),
            (ENV_NO_PROXY, "global.example"),
            (ENV_ATLASSIAN_NO_PROXY, "atlassian.example"),
            (ENV_JIRA_NO_PROXY, " example.atlassian.net,localhost "),
        ])
        .unwrap();

        assert_eq!(
            proxy.http_proxy.as_deref(),
            Some("http://jira-proxy.example:8080")
        );
        assert_eq!(
            proxy.https_proxy.as_deref(),
            Some("https://jira-secure-proxy.example:8443")
        );
        assert_eq!(
            proxy.no_proxy.as_deref(),
            Some("example.atlassian.net,localhost")
        );
    }

    #[test]
    fn proxy_config_uses_atlassian_fallback_before_standard_proxy_values() {
        let proxy = proxy_from_pairs(&[
            (ENV_HTTP_PROXY, "http://global-proxy.example:8080"),
            (
                ENV_ATLASSIAN_HTTP_PROXY,
                "http://atlassian-proxy.example:8080",
            ),
            (ENV_HTTPS_PROXY, "http://global-secure-proxy.example:8080"),
            (
                ENV_ATLASSIAN_HTTPS_PROXY,
                "http://atlassian-secure-proxy.example:8080",
            ),
            (ENV_NO_PROXY, "global.example"),
            (ENV_ATLASSIAN_NO_PROXY, " atlassian.example,localhost "),
        ])
        .unwrap();

        assert_eq!(
            proxy.http_proxy.as_deref(),
            Some("http://atlassian-proxy.example:8080")
        );
        assert_eq!(
            proxy.https_proxy.as_deref(),
            Some("http://atlassian-secure-proxy.example:8080")
        );
        assert_eq!(
            proxy.no_proxy.as_deref(),
            Some("atlassian.example,localhost")
        );
    }

    #[test]
    fn proxy_config_uses_global_fallback_values() {
        let proxy = proxy_from_pairs(&[
            (ENV_HTTP_PROXY, "http://global-proxy.example:8080"),
            (ENV_HTTPS_PROXY, "http://global-secure-proxy.example:8080"),
            (ENV_NO_PROXY, "example.atlassian.net"),
        ])
        .unwrap();

        assert_eq!(
            proxy.http_proxy.as_deref(),
            Some("http://global-proxy.example:8080")
        );
        assert_eq!(
            proxy.https_proxy.as_deref(),
            Some("http://global-secure-proxy.example:8080")
        );
        assert_eq!(proxy.no_proxy.as_deref(), Some("example.atlassian.net"));
    }

    #[test]
    fn invalid_proxy_url_is_rejected_without_leaking_credentials() {
        let error = proxy_from_pairs(&[(ENV_JIRA_HTTP_PROXY, "ftp://user:secret@proxy.example")])
            .unwrap_err();

        assert_eq!(
            error,
            ConfigError::InvalidProxyUrl {
                variable: ENV_JIRA_HTTP_PROXY,
            }
        );
        assert!(!error.to_string().contains("secret"));
        assert!(!error.to_string().contains("proxy.example"));
    }

    #[test]
    fn proxy_debug_redacts_credentials() {
        let proxy = ProxyConfig {
            http_proxy: Some("http://user:secret@proxy.example:8080".to_string()),
            https_proxy: None,
            no_proxy: Some("example.atlassian.net".to_string()),
        };
        let debug = format!("{proxy:?}");

        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("user:secret"));
        assert!(!debug.contains("secret"));
    }
}
