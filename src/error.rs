use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    InvalidHttpPort {
        variable: &'static str,
        value: String,
    },
    InvalidAllowedUrlDomain {
        variable: &'static str,
        value: String,
    },
    InvalidProxyUrl {
        variable: &'static str,
    },
    InvalidCustomHeaderFormat {
        variable: &'static str,
    },
    InvalidCustomHeaderName {
        variable: &'static str,
    },
    InvalidCustomHeaderValue {
        variable: &'static str,
        header: String,
    },
    ReservedCustomHeader {
        variable: &'static str,
        header: String,
    },
    MissingClientCertKeyPair {
        cert_variable: &'static str,
        key_variable: &'static str,
    },
    MissingJiraUrl {
        credential_variables: Vec<&'static str>,
    },
    InvalidJiraUrl {
        variable: &'static str,
    },
    MissingJiraCloudCredentials {
        missing_variables: Vec<&'static str>,
    },
    MissingJiraPersonalToken {
        variable: &'static str,
    },
    MissingJiraOAuthCloudId {
        access_token_variables: Vec<&'static str>,
        cloud_id_variable: &'static str,
    },
    InvalidJiraTimeout {
        variable: &'static str,
        value: String,
    },
    MissingConfluenceUrl {
        credential_variables: Vec<&'static str>,
    },
    InvalidConfluenceUrl {
        variable: &'static str,
    },
    MissingConfluenceCloudCredentials {
        missing_variables: Vec<&'static str>,
    },
    MissingConfluencePersonalToken {
        variable: &'static str,
    },
    MissingConfluenceOAuthCloudId {
        access_token_variables: Vec<&'static str>,
        cloud_id_variable: &'static str,
    },
    InvalidConfluenceTimeout {
        variable: &'static str,
        value: String,
    },
}

impl Display for ConfigError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHttpPort { variable, value } => {
                write!(formatter, "invalid {variable} value `{value}`")
            }
            Self::InvalidAllowedUrlDomain { variable, value } => {
                write!(formatter, "invalid {variable} domain `{value}`")
            }
            Self::InvalidProxyUrl { variable } => {
                write!(
                    formatter,
                    "invalid {variable} proxy URL; expected http or https URL"
                )
            }
            Self::InvalidCustomHeaderFormat { variable } => {
                write!(
                    formatter,
                    "invalid {variable} custom header; expected comma-separated key=value pairs"
                )
            }
            Self::InvalidCustomHeaderName { variable } => {
                write!(formatter, "invalid {variable} custom header name")
            }
            Self::InvalidCustomHeaderValue { variable, header } => {
                write!(
                    formatter,
                    "invalid {variable} custom header value for `{header}`"
                )
            }
            Self::ReservedCustomHeader { variable, header } => {
                write!(
                    formatter,
                    "reserved header `{header}` is not allowed in {variable}"
                )
            }
            Self::MissingClientCertKeyPair {
                cert_variable,
                key_variable,
            } => {
                write!(
                    formatter,
                    "mTLS client certificate and key must be configured together: {cert_variable}, {key_variable}"
                )
            }
            Self::MissingJiraUrl {
                credential_variables,
            } => {
                write!(
                    formatter,
                    "missing JIRA_URL while Jira credential variables are set: {}",
                    credential_variables.join(", ")
                )
            }
            Self::InvalidJiraUrl { variable } => {
                write!(formatter, "invalid {variable} value")
            }
            Self::MissingJiraCloudCredentials { missing_variables } => {
                write!(
                    formatter,
                    "missing Jira Cloud credential variables: {}",
                    missing_variables.join(", ")
                )
            }
            Self::MissingJiraPersonalToken { variable } => {
                write!(
                    formatter,
                    "missing Jira Server/Data Center credential variable: {variable}"
                )
            }
            Self::MissingJiraOAuthCloudId {
                access_token_variables,
                cloud_id_variable,
            } => write!(
                formatter,
                "missing {cloud_id_variable} while Jira OAuth/BYOT access token variables are set: {}",
                access_token_variables.join(", ")
            ),
            Self::InvalidJiraTimeout { variable, value } => {
                write!(formatter, "invalid {variable} value `{value}`")
            }
            Self::MissingConfluenceUrl {
                credential_variables,
            } => {
                write!(
                    formatter,
                    "missing CONFLUENCE_URL while Confluence credential variables are set: {}",
                    credential_variables.join(", ")
                )
            }
            Self::InvalidConfluenceUrl { variable } => {
                write!(formatter, "invalid {variable} value")
            }
            Self::MissingConfluenceCloudCredentials { missing_variables } => {
                write!(
                    formatter,
                    "missing Confluence Cloud credential variables: {}",
                    missing_variables.join(", ")
                )
            }
            Self::MissingConfluencePersonalToken { variable } => {
                write!(
                    formatter,
                    "missing Confluence Server/Data Center credential variable: {variable}"
                )
            }
            Self::MissingConfluenceOAuthCloudId {
                access_token_variables,
                cloud_id_variable,
            } => write!(
                formatter,
                "missing {cloud_id_variable} while Confluence OAuth/BYOT access token variables are set: {}",
                access_token_variables.join(", ")
            ),
            Self::InvalidConfluenceTimeout { variable, value } => {
                write!(formatter, "invalid {variable} value `{value}`")
            }
        }
    }
}

impl std::error::Error for ConfigError {}
