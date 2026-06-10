use std::{fs, path::PathBuf};

use reqwest::Identity;

use crate::{upstream::error::UpstreamError, error::ConfigError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientTlsIdentityConfig {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

impl ClientTlsIdentityConfig {
    #[cfg(test)]
    pub fn from_var_provider<F, E>(
        get_var: &mut F,
        cert_variable: &'static str,
        key_variable: &'static str,
    ) -> Result<Option<Self>, ConfigError>
    where
        F: FnMut(&str) -> Result<String, E>,
    {
        let cert_path = optional_var(get_var, cert_variable);
        let key_path = optional_var(get_var, key_variable);
        match (cert_path, key_path) {
            (None, None) => Ok(None),
            (Some(cert_path), Some(key_path)) => Ok(Some(Self {
                cert_path: PathBuf::from(cert_path),
                key_path: PathBuf::from(key_path),
            })),
            _ => Err(ConfigError::MissingClientCertKeyPair {
                cert_variable,
                key_variable,
            }),
        }
    }

    pub fn from_var_provider_with_fallback<F, E>(
        get_var: &mut F,
        service_cert_variable: &'static str,
        service_key_variable: &'static str,
        atlassian_cert_variable: &'static str,
        atlassian_key_variable: &'static str,
    ) -> Result<Option<Self>, ConfigError>
    where
        F: FnMut(&str) -> Result<String, E>,
    {
        let service_cert_path = optional_var(get_var, service_cert_variable);
        let service_key_path = optional_var(get_var, service_key_variable);
        if service_cert_path.is_some() || service_key_path.is_some() {
            return cert_key_pair_from_values(
                service_cert_path,
                service_key_path,
                service_cert_variable,
                service_key_variable,
            );
        }

        cert_key_pair_from_values(
            optional_var(get_var, atlassian_cert_variable),
            optional_var(get_var, atlassian_key_variable),
            atlassian_cert_variable,
            atlassian_key_variable,
        )
    }

    pub fn load_identity(&self) -> Result<Identity, UpstreamError> {
        let cert = fs::read(&self.cert_path).map_err(|_| {
            UpstreamError::invalid_input("failed to read mTLS client certificate file")
        })?;
        let key = fs::read(&self.key_path)
            .map_err(|_| UpstreamError::invalid_input("failed to read mTLS client key file"))?;
        let mut pem = cert;
        if !pem.ends_with(b"\n") {
            pem.push(b'\n');
        }
        pem.extend_from_slice(&key);

        Identity::from_pem(&pem).map_err(UpstreamError::transport)
    }
}

fn cert_key_pair_from_values(
    cert_path: Option<String>,
    key_path: Option<String>,
    cert_variable: &'static str,
    key_variable: &'static str,
) -> Result<Option<ClientTlsIdentityConfig>, ConfigError> {
    match (cert_path, key_path) {
        (None, None) => Ok(None),
        (Some(cert_path), Some(key_path)) => Ok(Some(ClientTlsIdentityConfig {
            cert_path: PathBuf::from(cert_path),
            key_path: PathBuf::from(key_path),
        })),
        _ => Err(ConfigError::MissingClientCertKeyPair {
            cert_variable,
            key_variable,
        }),
    }
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::atlassian::compat::{ENV_JIRA_CLIENT_CERT, ENV_JIRA_CLIENT_KEY};

    use super::*;

    fn mtls_from_pairs(
        pairs: &[(&str, &str)],
    ) -> Result<Option<ClientTlsIdentityConfig>, ConfigError> {
        let vars: BTreeMap<String, String> = pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();

        ClientTlsIdentityConfig::from_var_provider(
            &mut |key| vars.get(key).cloned().ok_or(()),
            ENV_JIRA_CLIENT_CERT,
            ENV_JIRA_CLIENT_KEY,
        )
    }

    #[test]
    fn mtls_config_parses_cert_key_pair() {
        let config = mtls_from_pairs(&[
            (ENV_JIRA_CLIENT_CERT, " /tmp/client.crt "),
            (ENV_JIRA_CLIENT_KEY, "/tmp/client.key"),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(config.cert_path, PathBuf::from("/tmp/client.crt"));
        assert_eq!(config.key_path, PathBuf::from("/tmp/client.key"));
    }

    #[test]
    fn mtls_config_rejects_missing_cert_or_key_pair() {
        let missing_key =
            mtls_from_pairs(&[(ENV_JIRA_CLIENT_CERT, "/tmp/client.crt")]).unwrap_err();
        assert_eq!(
            missing_key,
            ConfigError::MissingClientCertKeyPair {
                cert_variable: ENV_JIRA_CLIENT_CERT,
                key_variable: ENV_JIRA_CLIENT_KEY,
            }
        );

        let missing_cert =
            mtls_from_pairs(&[(ENV_JIRA_CLIENT_KEY, "/tmp/client.key")]).unwrap_err();
        assert_eq!(
            missing_cert,
            ConfigError::MissingClientCertKeyPair {
                cert_variable: ENV_JIRA_CLIENT_CERT,
                key_variable: ENV_JIRA_CLIENT_KEY,
            }
        );
    }
}
