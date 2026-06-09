use std::{fs, path::PathBuf};

use reqwest::Identity;

use crate::{atlassian::error::AtlassianError, error::ConfigError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientTlsIdentityConfig {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

impl ClientTlsIdentityConfig {
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

    pub fn load_identity(&self) -> Result<Identity, AtlassianError> {
        let cert = fs::read(&self.cert_path).map_err(|_| {
            AtlassianError::invalid_input("failed to read mTLS client certificate file")
        })?;
        let key = fs::read(&self.key_path)
            .map_err(|_| AtlassianError::invalid_input("failed to read mTLS client key file"))?;
        let mut pem = cert;
        if !pem.ends_with(b"\n") {
            pem.push(b'\n');
        }
        pem.extend_from_slice(&key);

        Identity::from_pem(&pem).map_err(AtlassianError::transport)
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
