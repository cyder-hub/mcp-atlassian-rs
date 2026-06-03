#![allow(dead_code)]

use std::fmt::{Debug, Formatter};

use reqwest::RequestBuilder;

#[derive(Clone, PartialEq, Eq)]
pub enum AtlassianAuth {
    Basic { username: String, api_token: String },
    Pat { personal_token: String },
}

impl AtlassianAuth {
    pub fn apply(&self, builder: RequestBuilder) -> RequestBuilder {
        match self {
            Self::Basic {
                username,
                api_token,
            } => builder.basic_auth(username, Some(api_token)),
            Self::Pat { personal_token } => builder.bearer_auth(personal_token),
        }
    }
}

impl Debug for AtlassianAuth {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic { username, .. } => formatter
                .debug_struct("AtlassianAuth::Basic")
                .field("username", username)
                .field("api_token", &"<redacted>")
                .finish(),
            Self::Pat { .. } => formatter
                .debug_struct("AtlassianAuth::Pat")
                .field("personal_token", &"<redacted>")
                .finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Client;

    use super::*;

    #[test]
    fn debug_output_redacts_basic_api_token() {
        let auth = AtlassianAuth::Basic {
            username: "user@example.com".to_string(),
            api_token: "test-api-token".to_string(),
        };
        let output = format!("{auth:?}");

        assert!(output.contains("user@example.com"));
        assert!(output.contains("<redacted>"));
        assert!(!output.contains("test-api-token"));
    }

    #[test]
    fn debug_output_redacts_pat() {
        let auth = AtlassianAuth::Pat {
            personal_token: "test-pat-value".to_string(),
        };
        let output = format!("{auth:?}");

        assert!(output.contains("<redacted>"));
        assert!(!output.contains("test-pat-value"));
    }

    #[test]
    fn pat_auth_applies_bearer_header_without_debug_leakage() {
        let expected_header = format!("Bearer {}", "test-pat-value");
        let auth = AtlassianAuth::Pat {
            personal_token: "test-pat-value".to_string(),
        };
        let request = auth
            .apply(Client::new().get("https://jira.example/rest/api/2/myself"))
            .build()
            .unwrap();
        let header = request.headers().get(reqwest::header::AUTHORIZATION);

        assert!(header.is_some());
        assert!(
            header
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value == expected_header)
        );
        assert!(!format!("{auth:?}").contains("test-pat-value"));
    }
}
