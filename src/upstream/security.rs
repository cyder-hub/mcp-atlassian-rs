use std::{
    fmt::{Display, Formatter},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs},
};

use reqwest::Url;

pub const ENV_ALLOWED_URL_DOMAINS: &str = "MCP_ALLOWED_URL_DOMAINS";
pub const MAX_SAME_ORIGIN_REDIRECTS: usize = 3;

pub const BLOCKED_HOSTNAMES: &[&str] = &["localhost", "metadata.google.internal"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlValidationError {
    EmptyUrl,
    InvalidUrl,
    BlockedScheme { scheme: String },
    MissingHost,
    BlockedHostname { hostname: String },
    BlockedIpAddress { address: IpAddr },
    HostNotAllowed { hostname: String },
    DnsResolutionFailed { hostname: String },
    DnsResolvedToBlockedIp { hostname: String, address: IpAddr },
}

impl Display for UrlValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyUrl => write!(formatter, "empty URL"),
            Self::InvalidUrl => write!(formatter, "invalid URL"),
            Self::BlockedScheme { scheme } => {
                write!(formatter, "blocked URL scheme `{scheme}`")
            }
            Self::MissingHost => write!(formatter, "URL is missing a hostname"),
            Self::BlockedHostname { hostname } => {
                write!(formatter, "blocked hostname `{hostname}`")
            }
            Self::BlockedIpAddress { address } => {
                write!(formatter, "blocked non-global IP address `{address}`")
            }
            Self::HostNotAllowed { hostname } => {
                write!(formatter, "hostname `{hostname}` is not in allowed domains")
            }
            Self::DnsResolutionFailed { hostname } => {
                write!(formatter, "DNS resolution failed for `{hostname}`")
            }
            Self::DnsResolvedToBlockedIp { hostname, address } => write!(
                formatter,
                "DNS for `{hostname}` resolved to blocked non-global IP `{address}`"
            ),
        }
    }
}

impl std::error::Error for UrlValidationError {}

pub fn validate_service_base_url_with_resolver<F>(
    value: &str,
    allowed_domains: Option<&[String]>,
    mut resolver: F,
) -> Result<Url, UrlValidationError>
where
    F: FnMut(&str) -> Result<Vec<IpAddr>, UrlValidationError>,
{
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(UrlValidationError::EmptyUrl);
    }

    let mut url = Url::parse(trimmed).map_err(|_| UrlValidationError::InvalidUrl)?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(UrlValidationError::BlockedScheme {
            scheme: url.scheme().to_string(),
        });
    }

    let hostname = url
        .host_str()
        .ok_or(UrlValidationError::MissingHost)?
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim_end_matches('.')
        .to_ascii_lowercase();

    if BLOCKED_HOSTNAMES.contains(&hostname.as_str()) {
        return Err(UrlValidationError::BlockedHostname { hostname });
    }

    if let Ok(address) = hostname.parse::<IpAddr>() {
        if is_non_global_ip(address) {
            return Err(UrlValidationError::BlockedIpAddress { address });
        }
        if allowed_domains.is_some() {
            return Err(UrlValidationError::HostNotAllowed { hostname });
        }
    } else {
        if let Some(allowed_domains) = allowed_domains
            && !hostname_matches_allowlist(&hostname, allowed_domains)
        {
            return Err(UrlValidationError::HostNotAllowed { hostname });
        }

        let addresses = resolver(&hostname)?;
        if addresses.is_empty() {
            return Err(UrlValidationError::DnsResolutionFailed { hostname });
        }
        if let Some(address) = addresses
            .into_iter()
            .find(|address| is_non_global_ip(*address))
        {
            return Err(UrlValidationError::DnsResolvedToBlockedIp { hostname, address });
        }
    }

    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

pub fn hostname_matches_allowlist(hostname: &str, allowed_domains: &[String]) -> bool {
    let hostname = hostname.trim_end_matches('.').to_ascii_lowercase();
    allowed_domains.iter().any(|domain| {
        let domain = normalize_allowed_domain(domain);
        !domain.is_empty() && (hostname == domain || hostname.ends_with(&format!(".{domain}")))
    })
}

pub fn is_non_global_ip(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => is_non_global_ipv4(address),
        IpAddr::V6(address) => is_non_global_ipv6(address),
    }
}

fn normalize_allowed_domain(domain: &str) -> String {
    domain
        .trim()
        .trim_start_matches('.')
        .trim_end_matches('.')
        .to_ascii_lowercase()
}

pub(crate) fn resolve_hostname(hostname: &str) -> Result<Vec<IpAddr>, UrlValidationError> {
    (hostname, 0)
        .to_socket_addrs()
        .map(|addresses| addresses.map(|address| address.ip()).collect())
        .map_err(|_| UrlValidationError::DnsResolutionFailed {
            hostname: hostname.to_string(),
        })
}

fn is_non_global_ipv4(address: Ipv4Addr) -> bool {
    let [a, b, c, d] = address.octets();
    address.is_private()
        || address.is_loopback()
        || address.is_link_local()
        || address.is_multicast()
        || address.is_unspecified()
        || address.is_broadcast()
        || a == 0
        || (a == 100 && (64..=127).contains(&b))
        || (a == 192 && b == 0 && c == 0)
        || (a == 192 && b == 0 && c == 2)
        || (a == 198 && matches!(b, 18 | 19))
        || (a == 198 && b == 51 && c == 100)
        || (a == 203 && b == 0 && c == 113)
        || a >= 240
        || (a == 255 && b == 255 && c == 255 && d == 255)
}

fn is_non_global_ipv6(address: Ipv6Addr) -> bool {
    if let Some(mapped) = address.to_ipv4_mapped() {
        return is_non_global_ipv4(mapped);
    }

    let segments = address.segments();
    address.is_loopback()
        || address.is_multicast()
        || address.is_unspecified()
        || (segments[0] & 0xfe00) == 0xfc00
        || (segments[0] & 0xffc0) == 0xfe80
        || (segments[0] == 0x2001 && segments[1] == 0x0db8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_resolver(_: &str) -> Result<Vec<IpAddr>, UrlValidationError> {
        Ok(vec![])
    }

    #[test]
    fn security_contract_exposes_allowlist_and_redirect_defaults() {
        assert_eq!(ENV_ALLOWED_URL_DOMAINS, "MCP_ALLOWED_URL_DOMAINS");
        assert_eq!(MAX_SAME_ORIGIN_REDIRECTS, 3);
        assert!(BLOCKED_HOSTNAMES.contains(&"localhost"));
        assert!(BLOCKED_HOSTNAMES.contains(&"metadata.google.internal"));
    }

    #[test]
    fn validates_and_normalizes_safe_allowlisted_url_with_dns_check() {
        let allowed_domains = vec!["atlassian.net".to_string()];
        let url = validate_service_base_url_with_resolver(
            "https://example.atlassian.net/wiki?token=secret#fragment",
            Some(&allowed_domains),
            |_| Ok(vec!["8.8.8.8".parse().unwrap()]),
        )
        .unwrap();

        assert_eq!(url.as_str(), "https://example.atlassian.net/wiki");
    }

    #[test]
    fn rejects_unsafe_service_urls_before_normalization() {
        assert!(matches!(
            validate_service_base_url_with_resolver("file:///etc/passwd", None, empty_resolver),
            Err(UrlValidationError::BlockedScheme { .. })
        ));
        assert!(matches!(
            validate_service_base_url_with_resolver("http://localhost:8080", None, empty_resolver),
            Err(UrlValidationError::BlockedHostname { .. })
        ));
        assert!(matches!(
            validate_service_base_url_with_resolver("http://169.254.169.254", None, empty_resolver),
            Err(UrlValidationError::BlockedIpAddress { .. })
        ));
    }

    #[test]
    fn rejects_non_global_ip_ranges() {
        for value in [
            "http://127.0.0.1",
            "http://10.0.0.1",
            "http://172.16.0.1",
            "http://192.168.0.1",
            "http://169.254.169.254",
            "http://198.51.100.10",
            "http://[fc00::1]",
            "http://[fe80::1]",
            "http://[2001:db8::1]",
        ] {
            let error =
                validate_service_base_url_with_resolver(value, None, empty_resolver).unwrap_err();

            assert!(
                matches!(error, UrlValidationError::BlockedIpAddress { .. }),
                "{value} returned {error:?}"
            );
        }
    }

    #[test]
    fn rejects_non_allowlisted_domain() {
        let allowed_domains = vec!["atlassian.net".to_string()];
        let error = validate_service_base_url_with_resolver(
            "https://evil.example/wiki",
            Some(&allowed_domains),
            |_| panic!("non-allowlisted host should not require DNS resolution"),
        )
        .unwrap_err();

        assert!(matches!(error, UrlValidationError::HostNotAllowed { .. }));
    }

    #[test]
    fn allowlist_still_rejects_dns_resolution_to_non_global_ip() {
        let allowed_domains = vec!["atlassian.net".to_string()];
        let error = validate_service_base_url_with_resolver(
            "https://example.atlassian.net/wiki",
            Some(&allowed_domains),
            |_| Ok(vec!["10.0.0.5".parse().unwrap()]),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            UrlValidationError::DnsResolvedToBlockedIp { .. }
        ));
    }

    #[test]
    fn allowlist_rejects_global_ip_literals() {
        let allowed_domains = vec!["atlassian.net".to_string()];
        let error = validate_service_base_url_with_resolver(
            "https://8.8.8.8/wiki",
            Some(&allowed_domains),
            |_| panic!("IP literals should not require DNS resolution"),
        )
        .unwrap_err();

        assert!(matches!(error, UrlValidationError::HostNotAllowed { .. }));
    }

    #[test]
    fn allowlist_matches_exact_hosts_and_subdomains_only() {
        let allowed_domains = vec!["atlassian.net".to_string()];

        assert!(hostname_matches_allowlist(
            "example.atlassian.net",
            &allowed_domains
        ));
        assert!(hostname_matches_allowlist(
            "atlassian.net",
            &allowed_domains
        ));
        assert!(!hostname_matches_allowlist(
            "evil-atlassian.net",
            &allowed_domains
        ));
        assert!(!hostname_matches_allowlist(
            "atlassian.net.evil.example",
            &allowed_domains
        ));
    }

    #[test]
    fn allowlist_does_not_bypass_blocked_ip_or_hostname() {
        let allowed_domains = vec![
            "localhost".to_string(),
            "169.254.169.254".to_string(),
            "atlassian.net".to_string(),
        ];

        assert!(matches!(
            validate_service_base_url_with_resolver(
                "http://localhost",
                Some(&allowed_domains),
                empty_resolver
            ),
            Err(UrlValidationError::BlockedHostname { .. })
        ));
        assert!(matches!(
            validate_service_base_url_with_resolver(
                "http://169.254.169.254",
                Some(&allowed_domains),
                empty_resolver
            ),
            Err(UrlValidationError::BlockedIpAddress { .. })
        ));
    }

    #[test]
    fn rejects_dns_resolution_to_non_global_ip() {
        let error = validate_service_base_url_with_resolver("https://jira.example", None, |_| {
            Ok(vec!["10.0.0.5".parse().unwrap()])
        })
        .unwrap_err();

        assert!(matches!(
            error,
            UrlValidationError::DnsResolvedToBlockedIp { .. }
        ));
    }

    #[test]
    fn allows_dns_resolution_to_global_ip() {
        let url = validate_service_base_url_with_resolver("https://jira.example", None, |_| {
            Ok(vec!["8.8.8.8".parse().unwrap()])
        })
        .unwrap();

        assert_eq!(url.as_str(), "https://jira.example/");
    }
}
