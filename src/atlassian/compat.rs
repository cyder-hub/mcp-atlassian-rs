#![allow(dead_code)]

pub const ENV_ATLASSIAN_OAUTH_ENABLE: &str = "ATLASSIAN_OAUTH_ENABLE";
pub const ENV_ATLASSIAN_OAUTH_CLOUD_ID: &str = "ATLASSIAN_OAUTH_CLOUD_ID";
pub const ENV_ATLASSIAN_OAUTH_ACCESS_TOKEN: &str = "ATLASSIAN_OAUTH_ACCESS_TOKEN";
pub const ENV_JIRA_OAUTH_ACCESS_TOKEN: &str = "JIRA_OAUTH_ACCESS_TOKEN";
pub const ENV_CONFLUENCE_OAUTH_ACCESS_TOKEN: &str = "CONFLUENCE_OAUTH_ACCESS_TOKEN";

pub const ENV_HTTP_PROXY: &str = "HTTP_PROXY";
pub const ENV_HTTPS_PROXY: &str = "HTTPS_PROXY";
pub const ENV_NO_PROXY: &str = "NO_PROXY";

pub const ENV_JIRA_HTTP_PROXY: &str = "JIRA_HTTP_PROXY";
pub const ENV_JIRA_HTTPS_PROXY: &str = "JIRA_HTTPS_PROXY";
pub const ENV_JIRA_NO_PROXY: &str = "JIRA_NO_PROXY";
pub const ENV_JIRA_CUSTOM_HEADERS: &str = "JIRA_CUSTOM_HEADERS";
pub const ENV_JIRA_CLIENT_CERT: &str = "JIRA_CLIENT_CERT";
pub const ENV_JIRA_CLIENT_KEY: &str = "JIRA_CLIENT_KEY";

pub const ENV_CONFLUENCE_HTTP_PROXY: &str = "CONFLUENCE_HTTP_PROXY";
pub const ENV_CONFLUENCE_HTTPS_PROXY: &str = "CONFLUENCE_HTTPS_PROXY";
pub const ENV_CONFLUENCE_NO_PROXY: &str = "CONFLUENCE_NO_PROXY";
pub const ENV_CONFLUENCE_CUSTOM_HEADERS: &str = "CONFLUENCE_CUSTOM_HEADERS";
pub const ENV_CONFLUENCE_CLIENT_CERT: &str = "CONFLUENCE_CLIENT_CERT";
pub const ENV_CONFLUENCE_CLIENT_KEY: &str = "CONFLUENCE_CLIENT_KEY";

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
    "x-atlassian-jira-personal-token",
    "x-atlassian-confluence-personal-token",
    "x-atlassian-jira-url",
    "x-atlassian-confluence-url",
    "x-atlassian-cloud-id",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityStatus {
    Included,
    Unsupported,
    Backlog,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byot_contract_exposes_access_token_env_names() {
        assert_eq!(ENV_ATLASSIAN_OAUTH_ENABLE, "ATLASSIAN_OAUTH_ENABLE");
        assert_eq!(ENV_ATLASSIAN_OAUTH_CLOUD_ID, "ATLASSIAN_OAUTH_CLOUD_ID");
        assert_eq!(
            ENV_ATLASSIAN_OAUTH_ACCESS_TOKEN,
            "ATLASSIAN_OAUTH_ACCESS_TOKEN"
        );
        assert_eq!(ENV_JIRA_OAUTH_ACCESS_TOKEN, "JIRA_OAUTH_ACCESS_TOKEN");
        assert_eq!(
            ENV_CONFLUENCE_OAUTH_ACCESS_TOKEN,
            "CONFLUENCE_OAUTH_ACCESS_TOKEN"
        );
        assert_eq!(CapabilityStatus::Included, CapabilityStatus::Included);
    }

    #[test]
    fn proxy_contract_exposes_service_and_global_env_names() {
        assert_eq!(ENV_HTTP_PROXY, "HTTP_PROXY");
        assert_eq!(ENV_HTTPS_PROXY, "HTTPS_PROXY");
        assert_eq!(ENV_NO_PROXY, "NO_PROXY");
        assert_eq!(ENV_JIRA_HTTP_PROXY, "JIRA_HTTP_PROXY");
        assert_eq!(ENV_JIRA_HTTPS_PROXY, "JIRA_HTTPS_PROXY");
        assert_eq!(ENV_JIRA_NO_PROXY, "JIRA_NO_PROXY");
        assert_eq!(ENV_CONFLUENCE_HTTP_PROXY, "CONFLUENCE_HTTP_PROXY");
        assert_eq!(ENV_CONFLUENCE_HTTPS_PROXY, "CONFLUENCE_HTTPS_PROXY");
        assert_eq!(ENV_CONFLUENCE_NO_PROXY, "CONFLUENCE_NO_PROXY");
    }

    #[test]
    fn custom_headers_contract_exposes_reserved_headers() {
        assert_eq!(ENV_JIRA_CUSTOM_HEADERS, "JIRA_CUSTOM_HEADERS");
        assert_eq!(ENV_CONFLUENCE_CUSTOM_HEADERS, "CONFLUENCE_CUSTOM_HEADERS");
        assert!(CUSTOM_HEADER_RESERVED_NAMES.contains(&"authorization"));
        assert!(CUSTOM_HEADER_RESERVED_NAMES.contains(&"cookie"));
        assert!(CUSTOM_HEADER_RESERVED_NAMES.contains(&"host"));
        assert!(CUSTOM_HEADER_RESERVED_NAMES.contains(&"content-type"));
        assert!(CUSTOM_HEADER_RESERVED_NAMES.contains(&"x-atlassian-cloud-id"));
    }

    #[test]
    fn mtls_contract_exposes_cert_key_env_names() {
        assert_eq!(ENV_JIRA_CLIENT_CERT, "JIRA_CLIENT_CERT");
        assert_eq!(ENV_JIRA_CLIENT_KEY, "JIRA_CLIENT_KEY");
        assert_eq!(ENV_CONFLUENCE_CLIENT_CERT, "CONFLUENCE_CLIENT_CERT");
        assert_eq!(ENV_CONFLUENCE_CLIENT_KEY, "CONFLUENCE_CLIENT_KEY");
    }

    #[test]
    fn sse_unsupported_contract_is_explicit() {
        assert_eq!(CapabilityStatus::Unsupported, CapabilityStatus::Unsupported);
        assert_eq!(CapabilityStatus::Backlog, CapabilityStatus::Backlog);
    }
}
