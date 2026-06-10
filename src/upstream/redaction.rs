#[cfg(test)]
use std::collections::BTreeMap;

#[cfg(test)]
use reqwest::{Url, header::HeaderMap};

pub const REDACTED: &str = "<redacted>";
const MIN_SECRET_LENGTH: usize = 4;

#[cfg(test)]
pub const SENSITIVE_HEADER_NAMES: &[&str] = &[
    "authorization",
    "cookie",
    "set-cookie",
    "proxy-authorization",
    "private-token",
    "job-token",
    "x-atlassian-jira-personal-token",
    "x-atlassian-confluence-personal-token",
];

pub const SENSITIVE_QUERY_KEYS: &[&str] = &[
    "token",
    "access_token",
    "api_token",
    "personal_token",
    "jwt",
    "client_secret",
    "password",
    "key",
    "signature",
];

pub const SECRET_ENV_SUFFIXES: &[&str] = &["_TOKEN", "_SECRET", "_PASSWORD"];

pub fn redact_text(text: &str) -> String {
    redact_text_with_secrets(text, current_env_secret_values())
}

pub fn redact_text_with_secrets<I, S>(text: &str, secrets: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut redacted = redact_query_fragments(&redact_auth_fragments(text));

    for secret in secrets {
        let secret = secret.as_ref();
        if is_redactable_secret_value(secret) {
            redacted = redacted.replace(secret, REDACTED);
        }
    }

    redacted
}

#[cfg(test)]
pub fn redact_url(value: &str) -> String {
    let Ok(mut url) = Url::parse(value) else {
        return redact_text(value);
    };

    let Some(_) = url.query() else {
        return url.to_string();
    };

    let pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(key, value)| {
            let value = if is_sensitive_query_key(&key) {
                REDACTED.to_string()
            } else {
                value.into_owned()
            };
            (key.into_owned(), value)
        })
        .collect();

    url.set_query(None);
    {
        let mut query_pairs = url.query_pairs_mut();
        for (key, value) in pairs {
            query_pairs.append_pair(&key, &value);
        }
    }

    url.to_string()
}

#[cfg(test)]
pub fn redact_header_map(headers: &HeaderMap) -> BTreeMap<String, String> {
    headers
        .iter()
        .map(|(name, value)| {
            let value = value.to_str().unwrap_or("<non-utf8>");
            (
                name.as_str().to_string(),
                redact_header_value(name.as_str(), value),
            )
        })
        .collect()
}

#[cfg(test)]
pub fn redact_header_value(name: &str, value: &str) -> String {
    let name = name.trim().to_ascii_lowercase();
    if !is_sensitive_header_name(&name) {
        return redact_text(value);
    }

    if name == "authorization" {
        return redact_authorization_value(value);
    }

    REDACTED.to_string()
}

pub fn env_secret_values_from_pairs<I, K, V>(pairs: I) -> Vec<String>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    pairs
        .into_iter()
        .filter_map(|(key, value)| {
            let value = value.as_ref().trim();
            (is_secret_env_key(key.as_ref()) && is_redactable_secret_value(value))
                .then(|| value.to_string())
        })
        .collect()
}

pub fn current_env_secret_values() -> Vec<String> {
    env_secret_values_from_pairs(std::env::vars())
}

#[cfg(test)]
pub fn is_sensitive_header_name(name: &str) -> bool {
    let name = name.trim().to_ascii_lowercase();
    SENSITIVE_HEADER_NAMES.contains(&name.as_str())
}

pub fn is_sensitive_query_key(key: &str) -> bool {
    let key = key.trim().to_ascii_lowercase();
    SENSITIVE_QUERY_KEYS.contains(&key.as_str())
}

pub fn is_secret_env_key(key: &str) -> bool {
    let key = key.trim().to_ascii_uppercase();
    SECRET_ENV_SUFFIXES
        .iter()
        .any(|suffix| key.ends_with(suffix))
}

fn is_redactable_secret_value(value: &str) -> bool {
    value.trim().chars().count() >= MIN_SECRET_LENGTH
}

#[cfg(test)]
fn redact_authorization_value(value: &str) -> String {
    for scheme in ["Bearer", "Token", "Basic"] {
        if value
            .trim_start()
            .get(..scheme.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(scheme))
        {
            let rest = value.trim_start()[scheme.len()..].trim_start();
            if !rest.is_empty() {
                return format!("{scheme} {REDACTED}");
            }
        }
    }

    REDACTED.to_string()
}

fn redact_auth_fragments(text: &str) -> String {
    let redacted = redact_prefixed_credential(text, "Bearer ");
    let redacted = redact_prefixed_credential(&redacted, "Token ");
    redact_prefixed_credential(&redacted, "Basic ")
}

fn redact_query_fragments(text: &str) -> String {
    let mut redacted = text.to_string();
    for key in SENSITIVE_QUERY_KEYS {
        redacted = redact_query_fragment_key(&redacted, key);
    }
    redacted
}

fn redact_query_fragment_key(input: &str, key: &str) -> String {
    let prefix = format!("{key}=");
    let mut output = String::with_capacity(input.len());
    let mut remaining = input;

    while let Some(index) = find_query_prefix(remaining, &prefix) {
        let (before, after_before) = remaining.split_at(index);
        output.push_str(before);
        output.push_str(&after_before[..prefix.len()]);
        output.push_str(REDACTED);

        let value_start = prefix.len();
        let value_end = after_before[value_start..]
            .char_indices()
            .find_map(|(offset, character)| is_query_value_delimiter(character).then_some(offset))
            .map_or(after_before.len(), |offset| value_start + offset);
        remaining = &after_before[value_end..];
    }

    output.push_str(remaining);
    output
}

fn find_query_prefix(haystack: &str, needle: &str) -> Option<usize> {
    let haystack_lower = haystack.to_ascii_lowercase();
    let needle_lower = needle.to_ascii_lowercase();
    let mut search_start = 0;

    while let Some(relative_index) = haystack_lower[search_start..].find(&needle_lower) {
        let index = search_start + relative_index;
        if is_query_prefix_boundary(haystack, index) {
            return Some(index);
        }
        search_start = index + needle.len();
        if search_start >= haystack.len() {
            return None;
        }
    }

    None
}

fn is_query_prefix_boundary(haystack: &str, index: usize) -> bool {
    if index == 0 {
        return true;
    }

    haystack[..index]
        .chars()
        .next_back()
        .is_some_and(|character| {
            character.is_whitespace()
                || matches!(
                    character,
                    '?' | '&' | '"' | '\'' | ',' | ';' | '(' | '[' | '{' | '<'
                )
        })
}

fn is_query_value_delimiter(character: char) -> bool {
    character.is_whitespace()
        || matches!(
            character,
            '&' | '"' | '\'' | ',' | ';' | ')' | ']' | '}' | '<' | '>'
        )
}

fn redact_prefixed_credential(input: &str, prefix: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut remaining = input;

    while let Some(index) = find_credential_prefix(remaining, prefix) {
        let (before, after_before) = remaining.split_at(index);
        output.push_str(before);
        output.push_str(prefix);
        output.push_str(REDACTED);

        let credential_start = prefix.len();
        let credential_end = after_before[credential_start..]
            .char_indices()
            .find_map(|(offset, character)| is_credential_delimiter(character).then_some(offset))
            .map_or(after_before.len(), |offset| credential_start + offset);
        remaining = &after_before[credential_end..];
    }

    output.push_str(remaining);
    output
}

fn find_credential_prefix(haystack: &str, needle: &str) -> Option<usize> {
    let haystack_lower = haystack.to_ascii_lowercase();
    let needle_lower = needle.to_ascii_lowercase();
    let mut search_start = 0;

    while let Some(relative_index) = haystack_lower[search_start..].find(&needle_lower) {
        let index = search_start + relative_index;
        if is_credential_prefix_boundary(haystack, index) {
            return Some(index);
        }
        search_start = index + needle.len();
        if search_start >= haystack.len() {
            return None;
        }
    }

    None
}

fn is_credential_prefix_boundary(haystack: &str, index: usize) -> bool {
    if index == 0 {
        return true;
    }

    haystack[..index]
        .chars()
        .next_back()
        .is_none_or(is_credential_delimiter)
}

fn is_credential_delimiter(character: char) -> bool {
    character.is_whitespace()
        || matches!(
            character,
            '"' | '\'' | ',' | ';' | ')' | ']' | '}' | '<' | '>'
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{AUTHORIZATION, HeaderValue};

    #[test]
    fn redaction_contract_exposes_sensitive_inputs() {
        assert!(SENSITIVE_HEADER_NAMES.contains(&"authorization"));
        assert!(SENSITIVE_HEADER_NAMES.contains(&"private-token"));
        assert!(SENSITIVE_HEADER_NAMES.contains(&"job-token"));
        assert!(SENSITIVE_HEADER_NAMES.contains(&"x-atlassian-jira-personal-token"));
        assert!(SENSITIVE_HEADER_NAMES.contains(&"x-atlassian-confluence-personal-token"));
        assert!(SENSITIVE_QUERY_KEYS.contains(&"access_token"));
        assert!(SENSITIVE_QUERY_KEYS.contains(&"client_secret"));
        assert!(SECRET_ENV_SUFFIXES.contains(&"_TOKEN"));
        assert_eq!(REDACTED, "<redacted>");
    }

    #[test]
    fn redact_text_masks_explicit_and_env_style_secrets() {
        let secrets = env_secret_values_from_pairs([
            ("JIRA_API_TOKEN", "jira-secret-token"),
            ("CONFLUENCE_PERSONAL_TOKEN", "conf-secret-token"),
            ("UNRELATED", "visible-value"),
        ]);
        let output =
            redact_text_with_secrets("jira-secret-token conf-secret-token visible-value", secrets);

        assert_eq!(output, "<redacted> <redacted> visible-value");
    }

    #[test]
    fn redact_text_masks_authorization_fragments_without_known_secret() {
        let output = redact_text_with_secrets(
            "Authorization: Bearer abcdef123 and Token pat-secret and Basic dXNlcjp0b2tlbg==",
            std::iter::empty::<&str>(),
        );

        assert_eq!(
            output,
            "Authorization: Bearer <redacted> and Token <redacted> and Basic <redacted>"
        );
    }

    #[test]
    fn redact_text_masks_query_fragments_without_full_url() {
        let output = redact_text_with_secrets(
            "failed /rest/api/2/issue?token=query-secret&client=abc api_token=api-secret",
            std::iter::empty::<&str>(),
        );

        assert_eq!(
            output,
            "failed /rest/api/2/issue?token=<redacted>&client=abc api_token=<redacted>"
        );
    }

    #[test]
    fn redact_url_masks_sensitive_query_values() {
        let output = redact_url(
            "https://jira.example/rest/api/2/attachment?token=secret&client=abc&password=pw",
        );

        assert_eq!(
            output,
            "https://jira.example/rest/api/2/attachment?token=%3Credacted%3E&client=abc&password=%3Credacted%3E"
        );
    }

    #[test]
    fn redact_header_map_masks_sensitive_headers_and_preserves_auth_scheme() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer secret-token"),
        );
        headers.insert(
            "x-atlassian-jira-personal-token",
            HeaderValue::from_static("jira-secret-token"),
        );
        headers.insert("x-request-id", HeaderValue::from_static("req-1"));

        let output = redact_header_map(&headers);

        assert_eq!(
            output.get("authorization"),
            Some(&"Bearer <redacted>".to_string())
        );
        assert_eq!(
            output.get("x-atlassian-jira-personal-token"),
            Some(&"<redacted>".to_string())
        );
        assert_eq!(output.get("x-request-id"), Some(&"req-1".to_string()));
    }

    #[test]
    fn redaction_negative_matrix_covers_sensitive_inputs() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Token pat-secret"));
        headers.insert(
            "x-atlassian-confluence-personal-token",
            HeaderValue::from_static("conf-pat-secret"),
        );
        headers.insert(
            "x-page-token",
            HeaderValue::from_static("visible-page-token"),
        );
        let header_output = redact_header_map(&headers);

        let secrets = env_secret_values_from_pairs([("ATLASSIAN_CLIENT_SECRET", "env-secret")]);
        let text_output = redact_text_with_secrets(
            "Bearer auth-secret /download?token=query-secret&page_token=visible-page-token env-secret",
            secrets,
        );
        let url_output = redact_url(
            "https://example.atlassian.net/path?client_secret=client-secret&page_token=visible-page-token",
        );

        let combined = format!("{header_output:?} {text_output} {url_output}");

        assert!(combined.contains("Token <redacted>"));
        assert!(combined.contains("Bearer <redacted>"));
        assert!(combined.contains("token=<redacted>"));
        assert!(combined.contains("client_secret=%3Credacted%3E"));
        assert!(combined.contains("visible-page-token"));
        assert!(!combined.contains("pat-secret"));
        assert!(!combined.contains("conf-pat-secret"));
        assert!(!combined.contains("auth-secret"));
        assert!(!combined.contains("query-secret"));
        assert!(!combined.contains("client-secret"));
        assert!(!combined.contains("env-secret"));
    }
}
