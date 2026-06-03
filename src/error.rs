use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    InvalidHttpPort {
        variable: &'static str,
        value: String,
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
    InvalidJiraTimeout {
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
            Self::InvalidJiraTimeout { variable, value } => {
                write!(formatter, "invalid {variable} value `{value}`")
            }
        }
    }
}

impl std::error::Error for ConfigError {}
