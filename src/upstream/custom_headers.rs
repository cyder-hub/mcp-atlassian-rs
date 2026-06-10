use std::fmt::{Debug, Formatter};

use reqwest::header::{HeaderName, HeaderValue};

use crate::error::ConfigError;

pub const CUSTOM_HEADER_RESERVED_NAMES: &[&str] = &[
    "authorization",
    "cookie",
    "set-cookie",
    "proxy-authorization",
    "host",
    "content-type",
    "content-length",
    "transfer-encoding",
    "connection",
    "private-token",
    "job-token",
    "x-atlassian-jira-personal-token",
    "x-atlassian-confluence-personal-token",
    "x-atlassian-jira-url",
    "x-atlassian-confluence-url",
    "x-atlassian-cloud-id",
];

#[derive(Clone, Default, PartialEq, Eq)]
pub struct CustomHeaders {
    headers: Vec<(HeaderName, HeaderValue)>,
}

impl CustomHeaders {
    pub fn from_var_provider<F, E>(
        get_var: &mut F,
        variable: &'static str,
    ) -> Result<Self, ConfigError>
    where
        F: FnMut(&str) -> Result<String, E>,
    {
        let Some(value) = optional_var(get_var, variable) else {
            return Ok(Self::default());
        };

        parse_custom_headers(variable, &value)
    }

    pub fn from_var_provider_with_fallback<F, E>(
        get_var: &mut F,
        service_variable: &'static str,
        atlassian_variable: &'static str,
    ) -> Result<Self, ConfigError>
    where
        F: FnMut(&str) -> Result<String, E>,
    {
        let Some((variable, value)) = optional_named_var(get_var, service_variable)
            .or_else(|| optional_named_var(get_var, atlassian_variable))
        else {
            return Ok(Self::default());
        };

        parse_custom_headers(variable, &value)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&HeaderName, &HeaderValue)> {
        self.headers.iter().map(|(name, value)| (name, value))
    }

    #[cfg(test)]
    fn value(&self, name: &str) -> Option<&str> {
        let name = HeaderName::from_bytes(name.as_bytes()).ok()?;
        self.headers
            .iter()
            .rev()
            .find(|(header_name, _)| header_name == name)
            .and_then(|(_, value)| value.to_str().ok())
    }
}

impl Debug for CustomHeaders {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let redacted = self
            .headers
            .iter()
            .map(|(name, _)| format!("{name}: <redacted>"))
            .collect::<Vec<_>>();
        formatter
            .debug_struct("CustomHeaders")
            .field("headers", &redacted)
            .finish()
    }
}

fn parse_custom_headers(variable: &'static str, value: &str) -> Result<CustomHeaders, ConfigError> {
    let mut headers = Vec::new();

    for pair in value
        .split(',')
        .map(str::trim)
        .filter(|pair| !pair.is_empty())
    {
        let Some((name, value)) = pair.split_once('=') else {
            return Err(ConfigError::InvalidCustomHeaderFormat { variable });
        };
        let name = name.trim();
        if name.is_empty() {
            return Err(ConfigError::InvalidCustomHeaderName { variable });
        }
        let normalized_name = name.to_ascii_lowercase();
        if CUSTOM_HEADER_RESERVED_NAMES.contains(&normalized_name.as_str()) {
            return Err(ConfigError::ReservedCustomHeader {
                variable,
                header: normalized_name,
            });
        }
        let header_name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| ConfigError::InvalidCustomHeaderName { variable })?;
        let header_value = HeaderValue::from_str(value.trim()).map_err(|_| {
            ConfigError::InvalidCustomHeaderValue {
                variable,
                header: header_name.as_str().to_string(),
            }
        })?;
        headers.push((header_name, header_value));
    }

    Ok(CustomHeaders { headers })
}

fn optional_var<F, E>(get_var: &mut F, key: &'static str) -> Option<String>
where
    F: FnMut(&str) -> Result<String, E>,
{
    get_var(key).ok().and_then(non_empty_trimmed)
}

fn optional_named_var<F, E>(get_var: &mut F, key: &'static str) -> Option<(&'static str, String)>
where
    F: FnMut(&str) -> Result<String, E>,
{
    optional_var(get_var, key).map(|value| (key, value))
}

fn non_empty_trimmed(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::atlassian::compat::ENV_JIRA_CUSTOM_HEADERS;

    use super::*;

    fn headers_from_pairs(pairs: &[(&str, &str)]) -> Result<CustomHeaders, ConfigError> {
        let vars: BTreeMap<String, String> = pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();

        CustomHeaders::from_var_provider(
            &mut |key| vars.get(key).cloned().ok_or(()),
            ENV_JIRA_CUSTOM_HEADERS,
        )
    }

    #[test]
    fn custom_headers_parse_key_value_pairs_and_allow_empty_value() {
        let headers = headers_from_pairs(&[(
            ENV_JIRA_CUSTOM_HEADERS,
            "X-Team=platform,X-Trace=abc=def,X-Empty=",
        )])
        .unwrap();

        assert_eq!(headers.value("x-team"), Some("platform"));
        assert_eq!(headers.value("x-trace"), Some("abc=def"));
        assert_eq!(headers.value("x-empty"), Some(""));
    }

    #[test]
    fn custom_headers_reject_pairs_without_equals_without_leaking_value() {
        let error =
            headers_from_pairs(&[(ENV_JIRA_CUSTOM_HEADERS, "X-Good=yes,bad-secret")]).unwrap_err();

        assert_eq!(
            error,
            ConfigError::InvalidCustomHeaderFormat {
                variable: ENV_JIRA_CUSTOM_HEADERS,
            }
        );
        assert!(!error.to_string().contains("bad-secret"));
    }

    #[test]
    fn custom_headers_reject_reserved_headers_without_leaking_value() {
        let error =
            headers_from_pairs(&[(ENV_JIRA_CUSTOM_HEADERS, "Authorization=Bearer secret-token")])
                .unwrap_err();

        assert_eq!(
            error,
            ConfigError::ReservedCustomHeader {
                variable: ENV_JIRA_CUSTOM_HEADERS,
                header: "authorization".to_string(),
            }
        );
        assert!(!error.to_string().contains("secret-token"));
    }

    #[test]
    fn custom_headers_reject_content_type_so_request_content_type_wins() {
        let error = headers_from_pairs(&[(ENV_JIRA_CUSTOM_HEADERS, "Content-Type=text/plain")])
            .unwrap_err();

        assert_eq!(
            error,
            ConfigError::ReservedCustomHeader {
                variable: ENV_JIRA_CUSTOM_HEADERS,
                header: "content-type".to_string(),
            }
        );
        assert!(!error.to_string().contains("text/plain"));
    }

    #[test]
    fn custom_headers_reject_invalid_name_and_value_without_leaking_value() {
        let invalid_name =
            headers_from_pairs(&[(ENV_JIRA_CUSTOM_HEADERS, "Bad Header=secret")]).unwrap_err();
        assert_eq!(
            invalid_name,
            ConfigError::InvalidCustomHeaderName {
                variable: ENV_JIRA_CUSTOM_HEADERS,
            }
        );
        assert!(!invalid_name.to_string().contains("secret"));

        let invalid_value =
            headers_from_pairs(&[(ENV_JIRA_CUSTOM_HEADERS, "X-Test=line\nsecret")]).unwrap_err();
        assert_eq!(
            invalid_value,
            ConfigError::InvalidCustomHeaderValue {
                variable: ENV_JIRA_CUSTOM_HEADERS,
                header: "x-test".to_string(),
            }
        );
        assert!(!invalid_value.to_string().contains("secret"));
    }

    #[test]
    fn custom_headers_debug_redacts_values() {
        let headers =
            headers_from_pairs(&[(ENV_JIRA_CUSTOM_HEADERS, "X-Token=secret-token")]).unwrap();
        let debug = format!("{headers:?}");

        assert!(debug.contains("x-token"));
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("secret-token"));
    }
}
