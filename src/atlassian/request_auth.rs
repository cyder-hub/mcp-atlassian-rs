use std::{
    collections::hash_map::DefaultHasher,
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
    net::IpAddr,
};

use reqwest::{Url, header::HeaderMap};

use crate::atlassian::{
    auth::AtlassianAuth,
    security::{UrlValidationError, resolve_hostname, validate_service_base_url_with_resolver},
};

pub const ENV_IGNORE_HEADER_AUTH: &str = "IGNORE_HEADER_AUTH";

pub const HEADER_AUTHORIZATION: &str = "authorization";
pub const HEADER_ATLASSIAN_CLOUD_ID: &str = "x-atlassian-cloud-id";
pub const HEADER_JIRA_URL: &str = "x-atlassian-jira-url";
pub const HEADER_JIRA_PERSONAL_TOKEN: &str = "x-atlassian-jira-personal-token";
pub const HEADER_CONFLUENCE_URL: &str = "x-atlassian-confluence-url";
pub const HEADER_CONFLUENCE_PERSONAL_TOKEN: &str = "x-atlassian-confluence-personal-token";

pub const AUTH_SCHEME_BASIC: &str = "Basic";
pub const AUTH_SCHEME_TOKEN: &str = "Token";
pub const AUTH_SCHEME_BEARER: &str = "Bearer";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestAuthContext {
    pub authorization: Option<AtlassianAuth>,
    pub jira: Option<ServiceAuthOverride>,
    pub confluence: Option<ServiceAuthOverride>,
    pub cloud_id: Option<String>,
    pub fingerprint: RequestAuthFingerprint,
}

impl RequestAuthContext {
    pub fn has_overrides(&self) -> bool {
        self.authorization.is_some()
            || self.jira.is_some()
            || self.confluence.is_some()
            || self.cloud_id.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceAuthOverride {
    pub base_url: String,
    pub auth: AtlassianAuth,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestAuthFingerprint(String);

impl RequestAuthFingerprint {
    #[cfg(test)]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestAuthError {
    EmptyAuthorizationHeader,
    UnsupportedAuthorizationScheme {
        scheme: Option<String>,
    },
    InvalidHeaderValue {
        header: &'static str,
    },
    EmptyCredential {
        scheme: &'static str,
    },
    InvalidBasicEncoding,
    InvalidBasicFormat,
    MissingServiceHeaderPair {
        service: &'static str,
    },
    InvalidServiceUrl {
        service: &'static str,
        source: UrlValidationError,
    },
}

impl Display for RequestAuthError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyAuthorizationHeader => write!(formatter, "empty Authorization header"),
            Self::UnsupportedAuthorizationScheme { scheme } => {
                if let Some(scheme) = scheme {
                    write!(formatter, "unsupported Authorization scheme `{scheme}`")
                } else {
                    write!(formatter, "unsupported Authorization scheme")
                }
            }
            Self::InvalidHeaderValue { header } => {
                write!(formatter, "invalid {header} header value")
            }
            Self::EmptyCredential { scheme } => {
                write!(formatter, "empty {scheme} credential")
            }
            Self::InvalidBasicEncoding => write!(formatter, "invalid Basic auth encoding"),
            Self::InvalidBasicFormat => {
                write!(
                    formatter,
                    "invalid Basic auth format; expected email:api_token"
                )
            }
            Self::MissingServiceHeaderPair { service } => {
                write!(formatter, "missing {service} URL/token header pair")
            }
            Self::InvalidServiceUrl { service, source } => {
                write!(formatter, "invalid {service} URL: {source}")
            }
        }
    }
}

impl std::error::Error for RequestAuthError {}

pub fn parse_request_auth_headers_with_oauth_bearer(
    headers: &HeaderMap,
    ignore_header_auth: bool,
    allowed_domains: Option<&[String]>,
    oauth_bearer_enabled: bool,
) -> Result<RequestAuthContext, RequestAuthError> {
    parse_request_auth_headers_with_resolver_and_oauth_bearer(
        headers,
        ignore_header_auth,
        allowed_domains,
        oauth_bearer_enabled,
        resolve_hostname,
    )
}

#[cfg(test)]
pub fn parse_request_auth_headers_with_resolver<F>(
    headers: &HeaderMap,
    ignore_header_auth: bool,
    allowed_domains: Option<&[String]>,
    mut resolver: F,
) -> Result<RequestAuthContext, RequestAuthError>
where
    F: FnMut(&str) -> Result<Vec<IpAddr>, UrlValidationError>,
{
    parse_request_auth_headers_with_resolver_and_oauth_bearer(
        headers,
        ignore_header_auth,
        allowed_domains,
        false,
        &mut resolver,
    )
}

pub fn parse_request_auth_headers_with_resolver_and_oauth_bearer<F>(
    headers: &HeaderMap,
    ignore_header_auth: bool,
    allowed_domains: Option<&[String]>,
    oauth_bearer_enabled: bool,
    mut resolver: F,
) -> Result<RequestAuthContext, RequestAuthError>
where
    F: FnMut(&str) -> Result<Vec<IpAddr>, UrlValidationError>,
{
    if ignore_header_auth {
        return Ok(RequestAuthContext {
            authorization: None,
            jira: None,
            confluence: None,
            cloud_id: None,
            fingerprint: RequestAuthFingerprint("ignored-header-auth".to_string()),
        });
    }

    let parsed_authorization = optional_header(headers, HEADER_AUTHORIZATION)?
        .map(parse_authorization_header)
        .transpose()?;
    let cloud_id = optional_header(headers, HEADER_ATLASSIAN_CLOUD_ID)?.map(ToString::to_string);
    let authorization = parsed_authorization
        .map(|authorization| authorization.into_auth(oauth_bearer_enabled || cloud_id.is_some()));
    let jira = parse_service_headers(
        headers,
        "Jira",
        HEADER_JIRA_URL,
        HEADER_JIRA_PERSONAL_TOKEN,
        allowed_domains,
        &mut resolver,
    )?;
    let confluence = parse_service_headers(
        headers,
        "Confluence",
        HEADER_CONFLUENCE_URL,
        HEADER_CONFLUENCE_PERSONAL_TOKEN,
        allowed_domains,
        &mut resolver,
    )?;

    let fingerprint = build_fingerprint(
        authorization.as_ref(),
        jira.as_ref(),
        confluence.as_ref(),
        cloud_id.as_deref(),
    );

    Ok(RequestAuthContext {
        authorization,
        jira,
        confluence,
        cloud_id,
        fingerprint,
    })
}

fn parse_service_headers<F>(
    headers: &HeaderMap,
    service: &'static str,
    url_header: &'static str,
    token_header: &'static str,
    allowed_domains: Option<&[String]>,
    resolver: &mut F,
) -> Result<Option<ServiceAuthOverride>, RequestAuthError>
where
    F: FnMut(&str) -> Result<Vec<IpAddr>, UrlValidationError>,
{
    let base_url = optional_header(headers, url_header)?;
    let personal_token = optional_header(headers, token_header)?;

    match (base_url, personal_token) {
        (None, None) => Ok(None),
        (Some(_), None) | (None, Some(_)) => {
            Err(RequestAuthError::MissingServiceHeaderPair { service })
        }
        (Some(base_url), Some(personal_token)) => {
            let url = validate_service_base_url_with_resolver(base_url, allowed_domains, resolver)
                .map_err(|source| RequestAuthError::InvalidServiceUrl { service, source })?;
            Ok(Some(ServiceAuthOverride {
                base_url: url.to_string().trim_end_matches('/').to_string(),
                auth: AtlassianAuth::Pat {
                    personal_token: personal_token.to_string(),
                },
            }))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedAuthorization {
    scheme: ParsedAuthorizationScheme,
    auth: AtlassianAuth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParsedAuthorizationScheme {
    Basic,
    Token,
    Bearer,
}

impl ParsedAuthorization {
    fn into_auth(self, oauth_bearer: bool) -> AtlassianAuth {
        match (self.scheme, self.auth, oauth_bearer) {
            (ParsedAuthorizationScheme::Bearer, AtlassianAuth::Pat { personal_token }, true) => {
                AtlassianAuth::OAuthAccessToken {
                    access_token: personal_token,
                }
            }
            (_, auth, _) => auth,
        }
    }
}

fn parse_authorization_header(value: &str) -> Result<ParsedAuthorization, RequestAuthError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(RequestAuthError::EmptyAuthorizationHeader);
    }

    let Some((scheme, credential)) = trimmed.split_once(char::is_whitespace) else {
        return Err(RequestAuthError::UnsupportedAuthorizationScheme { scheme: None });
    };
    let credential = credential.trim();
    if credential.is_empty() {
        return Err(RequestAuthError::EmptyCredential {
            scheme: canonical_scheme(scheme),
        });
    }

    if scheme.eq_ignore_ascii_case(AUTH_SCHEME_BASIC) {
        let decoded = decode_basic_credentials(credential)?;
        let Some((username, api_token)) = decoded.split_once(':') else {
            return Err(RequestAuthError::InvalidBasicFormat);
        };
        if username.is_empty() || api_token.is_empty() {
            return Err(RequestAuthError::InvalidBasicFormat);
        }
        return Ok(ParsedAuthorization {
            scheme: ParsedAuthorizationScheme::Basic,
            auth: AtlassianAuth::Basic {
                username: username.to_string(),
                api_token: api_token.to_string(),
            },
        });
    }

    if scheme.eq_ignore_ascii_case(AUTH_SCHEME_TOKEN) {
        return Ok(ParsedAuthorization {
            scheme: ParsedAuthorizationScheme::Token,
            auth: AtlassianAuth::Pat {
                personal_token: credential.to_string(),
            },
        });
    }

    if scheme.eq_ignore_ascii_case(AUTH_SCHEME_BEARER) {
        return Ok(ParsedAuthorization {
            scheme: ParsedAuthorizationScheme::Bearer,
            auth: AtlassianAuth::Pat {
                personal_token: credential.to_string(),
            },
        });
    }

    Err(RequestAuthError::UnsupportedAuthorizationScheme {
        scheme: safe_authorization_scheme(scheme),
    })
}

fn optional_header<'a>(
    headers: &'a HeaderMap,
    name: &'static str,
) -> Result<Option<&'a str>, RequestAuthError> {
    let Some(value) = headers.get(name) else {
        return Ok(None);
    };
    let value = value
        .to_str()
        .map_err(|_| RequestAuthError::InvalidHeaderValue { header: name })?
        .trim();

    if value.is_empty() && name == HEADER_AUTHORIZATION {
        return Err(RequestAuthError::EmptyAuthorizationHeader);
    }

    Ok((!value.is_empty()).then_some(value))
}

fn canonical_scheme(scheme: &str) -> &'static str {
    if scheme.eq_ignore_ascii_case(AUTH_SCHEME_BASIC) {
        AUTH_SCHEME_BASIC
    } else if scheme.eq_ignore_ascii_case(AUTH_SCHEME_TOKEN) {
        AUTH_SCHEME_TOKEN
    } else if scheme.eq_ignore_ascii_case(AUTH_SCHEME_BEARER) {
        AUTH_SCHEME_BEARER
    } else {
        "Authorization"
    }
}

fn safe_authorization_scheme(scheme: &str) -> Option<String> {
    let scheme = scheme.trim();
    (!scheme.is_empty()
        && scheme.len() <= 32
        && scheme
            .chars()
            .all(|character| character.is_ascii_alphanumeric()))
    .then(|| scheme.to_string())
}

fn build_fingerprint(
    authorization: Option<&AtlassianAuth>,
    jira: Option<&ServiceAuthOverride>,
    confluence: Option<&ServiceAuthOverride>,
    cloud_id: Option<&str>,
) -> RequestAuthFingerprint {
    let mut parts = Vec::new();
    parts.push(match authorization {
        Some(AtlassianAuth::Basic {
            username,
            api_token,
        }) => format!(
            "authorization=basic:{}",
            secret_hash(&format!("{username}:{api_token}"))
        ),
        Some(AtlassianAuth::Pat { personal_token }) => {
            format!("authorization=pat:{}", secret_hash(personal_token))
        }
        Some(AtlassianAuth::OAuthAccessToken { access_token }) => {
            format!(
                "authorization=oauth_access_token:{}",
                secret_hash(access_token)
            )
        }
        None => "authorization=global".to_string(),
    });
    parts.push(service_fingerprint("jira", jira));
    parts.push(service_fingerprint("confluence", confluence));
    parts.push(format!(
        "cloud_id={}",
        if cloud_id.is_some() {
            "present"
        } else {
            "absent"
        }
    ));

    RequestAuthFingerprint(parts.join("|"))
}

fn service_fingerprint(service: &str, auth: Option<&ServiceAuthOverride>) -> String {
    auth.map_or_else(
        || format!("{service}=global"),
        |auth| match &auth.auth {
            AtlassianAuth::Basic {
                username,
                api_token,
            } => format!(
                "{service}=basic:{}:{}",
                service_host(&auth.base_url),
                secret_hash(&format!("{username}:{api_token}"))
            ),
            AtlassianAuth::Pat { personal_token } => {
                format!(
                    "{service}=pat:{}:{}",
                    service_host(&auth.base_url),
                    secret_hash(personal_token)
                )
            }
            AtlassianAuth::OAuthAccessToken { access_token } => {
                format!(
                    "{service}=oauth_access_token:{}:{}",
                    service_host(&auth.base_url),
                    secret_hash(access_token)
                )
            }
        },
    )
}

fn secret_hash(value: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn service_host(base_url: &str) -> String {
    Url::parse(base_url)
        .ok()
        .and_then(|url| url.host_str().map(ToString::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

fn decode_basic_credentials(value: &str) -> Result<String, RequestAuthError> {
    let bytes = decode_base64(value).ok_or(RequestAuthError::InvalidBasicEncoding)?;
    String::from_utf8(bytes).map_err(|_| RequestAuthError::InvalidBasicEncoding)
}

fn decode_base64(value: &str) -> Option<Vec<u8>> {
    let mut output = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0u8;
    let mut padding = false;

    for byte in value.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
        if byte == b'=' {
            padding = true;
            continue;
        }
        if padding {
            return None;
        }
        let value = base64_value(byte)? as u32;
        buffer = (buffer << 6) | value;
        bits += 6;
        while bits >= 8 {
            bits -= 8;
            output.push(((buffer >> bits) & 0xff) as u8);
        }
    }

    Some(output)
}

fn base64_value(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atlassian::redaction::REDACTED;
    use reqwest::header::{AUTHORIZATION, HeaderValue};

    #[test]
    fn request_auth_contract_exposes_headers() {
        assert_eq!(ENV_IGNORE_HEADER_AUTH, "IGNORE_HEADER_AUTH");
        assert_eq!(HEADER_AUTHORIZATION, "authorization");
        assert_eq!(HEADER_ATLASSIAN_CLOUD_ID, "x-atlassian-cloud-id");
        assert_eq!(HEADER_JIRA_URL, "x-atlassian-jira-url");
        assert_eq!(
            HEADER_JIRA_PERSONAL_TOKEN,
            "x-atlassian-jira-personal-token"
        );
        assert_eq!(HEADER_CONFLUENCE_URL, "x-atlassian-confluence-url");
        assert_eq!(
            HEADER_CONFLUENCE_PERSONAL_TOKEN,
            "x-atlassian-confluence-personal-token"
        );
        assert_eq!(AUTH_SCHEME_BASIC, "Basic");
        assert_eq!(AUTH_SCHEME_TOKEN, "Token");
        assert_eq!(AUTH_SCHEME_BEARER, "Bearer");
    }

    #[test]
    fn parses_basic_authorization_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Basic dXNlckBleGFtcGxlLmNvbTphcGktdG9rZW4="),
        );

        let context =
            parse_request_auth_headers_with_resolver(&headers, false, None, empty_resolver)
                .unwrap();

        assert_eq!(
            context.authorization,
            Some(AtlassianAuth::Basic {
                username: "user@example.com".to_string(),
                api_token: "api-token".to_string(),
            })
        );
        assert!(!context.fingerprint.as_str().contains("api-token"));
    }

    #[test]
    fn parses_bearer_as_oauth_access_token_when_cloud_id_present() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer bearer-token"),
        );
        headers.insert(
            HEADER_JIRA_URL,
            HeaderValue::from_static("https://example.atlassian.net?token=query-secret"),
        );
        headers.insert(
            HEADER_JIRA_PERSONAL_TOKEN,
            HeaderValue::from_static("jira-pat-token"),
        );
        headers.insert(
            HEADER_ATLASSIAN_CLOUD_ID,
            HeaderValue::from_static("cloud-123"),
        );
        let allowed_domains = vec!["atlassian.net".to_string()];

        let context = parse_request_auth_headers_with_resolver(
            &headers,
            false,
            Some(&allowed_domains),
            |_| Ok(vec!["8.8.8.8".parse().unwrap()]),
        )
        .unwrap();

        assert_eq!(
            context.authorization,
            Some(AtlassianAuth::OAuthAccessToken {
                access_token: "bearer-token".to_string(),
            })
        );
        assert_eq!(
            context.jira.as_ref().unwrap().base_url,
            "https://example.atlassian.net"
        );
        assert_eq!(context.cloud_id.as_deref(), Some("cloud-123"));
        assert!(context.has_overrides());
        assert!(!context.fingerprint.as_str().contains("bearer-token"));
        assert!(!context.fingerprint.as_str().contains("jira-pat-token"));
    }

    #[test]
    fn parses_bearer_as_oauth_access_token_when_oauth_bearer_enabled() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer access-token"),
        );

        let context = parse_request_auth_headers_with_resolver_and_oauth_bearer(
            &headers,
            false,
            None,
            true,
            empty_resolver,
        )
        .unwrap();

        assert_eq!(
            context.authorization,
            Some(AtlassianAuth::OAuthAccessToken {
                access_token: "access-token".to_string(),
            })
        );
        assert_eq!(context.cloud_id, None);
        assert!(!context.fingerprint.as_str().contains("access-token"));
    }

    #[test]
    fn parses_bearer_as_pat_without_byot_signal() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer pat-token"));

        let context =
            parse_request_auth_headers_with_resolver(&headers, false, None, empty_resolver)
                .unwrap();

        assert_eq!(
            context.authorization,
            Some(AtlassianAuth::Pat {
                personal_token: "pat-token".to_string(),
            })
        );
        assert_eq!(context.cloud_id, None);
        assert!(!context.fingerprint.as_str().contains("pat-token"));
    }

    #[test]
    fn rejects_missing_service_header_pair_and_unsafe_urls() {
        let mut missing_pair = HeaderMap::new();
        missing_pair.insert(
            HEADER_CONFLUENCE_URL,
            HeaderValue::from_static("https://example.atlassian.net/wiki"),
        );
        let error =
            parse_request_auth_headers_with_resolver(&missing_pair, false, None, empty_resolver)
                .unwrap_err();
        assert!(matches!(
            error,
            RequestAuthError::MissingServiceHeaderPair {
                service: "Confluence"
            }
        ));

        let mut unsafe_url = HeaderMap::new();
        unsafe_url.insert(
            HEADER_JIRA_URL,
            HeaderValue::from_static("http://127.0.0.1"),
        );
        unsafe_url.insert(
            HEADER_JIRA_PERSONAL_TOKEN,
            HeaderValue::from_static("jira-pat-token"),
        );
        let error =
            parse_request_auth_headers_with_resolver(&unsafe_url, false, None, empty_resolver)
                .unwrap_err();
        assert!(matches!(
            error,
            RequestAuthError::InvalidServiceUrl {
                service: "Jira",
                ..
            }
        ));
        assert!(!error.to_string().contains("jira-pat-token"));
    }

    #[test]
    fn ignore_header_auth_returns_global_fingerprint_without_parsing_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Unsupported secret"),
        );

        let context =
            parse_request_auth_headers_with_resolver(&headers, true, None, empty_resolver).unwrap();

        assert_eq!(context.authorization, None);
        assert_eq!(context.fingerprint.as_str(), "ignored-header-auth");
    }

    #[test]
    fn rejects_empty_and_unsupported_authorization() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Digest secret"));
        let error = parse_request_auth_headers_with_resolver(&headers, false, None, empty_resolver)
            .unwrap_err();

        assert!(matches!(
            error,
            RequestAuthError::UnsupportedAuthorizationScheme { .. }
        ));
        assert!(!error.to_string().contains("secret"));
        assert_eq!(REDACTED, "<redacted>");
    }

    #[test]
    fn rejects_present_empty_invalid_or_unstructured_authorization_without_echoing_value() {
        let mut empty = HeaderMap::new();
        empty.insert(AUTHORIZATION, HeaderValue::from_static("   "));
        let error = parse_request_auth_headers_with_resolver(&empty, false, None, empty_resolver)
            .unwrap_err();
        assert!(matches!(error, RequestAuthError::EmptyAuthorizationHeader));

        let mut invalid = HeaderMap::new();
        invalid.insert(
            AUTHORIZATION,
            HeaderValue::from_bytes(b"Bearer \xff").unwrap(),
        );
        let error = parse_request_auth_headers_with_resolver(&invalid, false, None, empty_resolver)
            .unwrap_err();
        assert!(matches!(
            error,
            RequestAuthError::InvalidHeaderValue {
                header: HEADER_AUTHORIZATION
            }
        ));

        let mut raw_secret = HeaderMap::new();
        raw_secret.insert(
            AUTHORIZATION,
            HeaderValue::from_static("raw-secret-token-value"),
        );
        let error =
            parse_request_auth_headers_with_resolver(&raw_secret, false, None, empty_resolver)
                .unwrap_err();
        let message = error.to_string();
        assert!(matches!(
            error,
            RequestAuthError::UnsupportedAuthorizationScheme { scheme: None }
        ));
        assert!(!message.contains("raw-secret-token-value"));
    }

    fn empty_resolver(_: &str) -> Result<Vec<IpAddr>, UrlValidationError> {
        Ok(vec!["8.8.8.8".parse().unwrap()])
    }
}
