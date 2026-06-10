use reqwest::Url;

use crate::{
    atlassian::{
        auth::UpstreamAuth,
        compat::{
            ENV_ATLASSIAN_API_TOKEN, ENV_ATLASSIAN_CLIENT_CERT, ENV_ATLASSIAN_CLIENT_KEY,
            ENV_ATLASSIAN_CUSTOM_HEADERS, ENV_ATLASSIAN_OAUTH_ACCESS_TOKEN,
            ENV_ATLASSIAN_OAUTH_CLOUD_ID, ENV_ATLASSIAN_PASSWORD, ENV_ATLASSIAN_PERSONAL_TOKEN,
            ENV_ATLASSIAN_SSL_VERIFY, ENV_ATLASSIAN_TIMEOUT, ENV_ATLASSIAN_USERNAME,
        },
        custom_headers::CustomHeaders,
        mtls::ClientTlsIdentityConfig,
        proxy::ProxyConfig,
    },
    error::ConfigError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedUpstreamServiceConfig<D> {
    pub base_url: String,
    pub deployment: D,
    pub auth: UpstreamAuth,
    pub oauth_cloud_id: Option<String>,
    pub ssl_verify: bool,
    pub proxy: ProxyConfig,
    pub custom_headers: CustomHeaders,
    pub mtls: Option<ClientTlsIdentityConfig>,
    pub timeout_seconds: u64,
}

pub struct UpstreamServiceConfigSpec<D> {
    pub url_variable: &'static str,
    pub username_variable: &'static str,
    pub api_token_variable: &'static str,
    pub password_variable: &'static str,
    pub personal_token_variable: &'static str,
    pub oauth_access_token_variable: &'static str,
    pub ssl_verify_variable: &'static str,
    pub timeout_variable: &'static str,
    pub http_proxy_variable: &'static str,
    pub https_proxy_variable: &'static str,
    pub no_proxy_variable: &'static str,
    pub custom_headers_variable: &'static str,
    pub client_cert_variable: &'static str,
    pub client_key_variable: &'static str,
    pub default_timeout_seconds: u64,
    pub cloud_deployment: D,
    pub deployment_from_url: fn(&Url) -> D,
    pub cloud_oauth_api_base_url: fn(&str) -> String,
    pub missing_url_error: fn(Vec<&'static str>) -> ConfigError,
    pub invalid_url_error: fn(&'static str) -> ConfigError,
    pub missing_cloud_credentials_error: fn(Vec<&'static str>) -> ConfigError,
    pub missing_personal_token_error: fn(&'static str) -> ConfigError,
    pub missing_oauth_cloud_id_error: fn(Vec<&'static str>, &'static str) -> ConfigError,
    pub invalid_timeout_error: fn(&'static str, String) -> ConfigError,
}

pub fn parse_upstream_service_config<F, E, D>(
    get_var: &mut F,
    spec: &UpstreamServiceConfigSpec<D>,
) -> Result<Option<ParsedUpstreamServiceConfig<D>>, ConfigError>
where
    F: FnMut(&str) -> Result<String, E>,
    D: Copy + Eq,
{
    let base_url = optional_named_var(get_var, spec.url_variable);
    let username =
        optional_service_or_atlassian_var(get_var, spec.username_variable, ENV_ATLASSIAN_USERNAME);
    let api_token = optional_service_or_atlassian_var(
        get_var,
        spec.api_token_variable,
        ENV_ATLASSIAN_API_TOKEN,
    );
    let password =
        optional_service_or_atlassian_var(get_var, spec.password_variable, ENV_ATLASSIAN_PASSWORD);
    let personal_token = optional_service_or_atlassian_var(
        get_var,
        spec.personal_token_variable,
        ENV_ATLASSIAN_PERSONAL_TOKEN,
    );
    let service_oauth_access_token = optional_named_var(get_var, spec.oauth_access_token_variable);
    let shared_oauth_access_token = optional_named_var(get_var, ENV_ATLASSIAN_OAUTH_ACCESS_TOKEN);
    let oauth_access_token = service_oauth_access_token
        .clone()
        .or_else(|| shared_oauth_access_token.clone());
    let oauth_access_token_variables = present_variables([
        service_oauth_access_token.as_ref(),
        shared_oauth_access_token.as_ref(),
    ]);
    let oauth_cloud_id = optional_named_var(get_var, ENV_ATLASSIAN_OAUTH_CLOUD_ID);

    let Some(base_url) = base_url else {
        let credential_variables = present_variables([
            service_specific_variable(username.as_ref(), spec.username_variable),
            service_specific_variable(api_token.as_ref(), spec.api_token_variable),
            service_specific_variable(password.as_ref(), spec.password_variable),
            service_specific_variable(personal_token.as_ref(), spec.personal_token_variable),
            service_oauth_access_token.as_ref(),
        ]);

        if credential_variables.is_empty() {
            return Ok(None);
        }

        return Err((spec.missing_url_error)(credential_variables));
    };

    let parsed_url = parse_base_url(&base_url.value, spec.url_variable, spec.invalid_url_error)?;
    let deployment = (spec.deployment_from_url)(&parsed_url);
    let (auth, oauth_cloud_id) = if deployment == spec.cloud_deployment {
        if let Some(access_token) = oauth_access_token {
            let Some(cloud_id) = oauth_cloud_id else {
                return Err((spec.missing_oauth_cloud_id_error)(
                    oauth_access_token_variables,
                    ENV_ATLASSIAN_OAUTH_CLOUD_ID,
                ));
            };

            (
                UpstreamAuth::OAuthAccessToken {
                    access_token: access_token.value,
                },
                Some(cloud_id.value),
            )
        } else {
            let missing_variables = missing_service_variables([
                (spec.username_variable, username.as_ref()),
                (spec.api_token_variable, api_token.as_ref()),
            ]);

            if !missing_variables.is_empty() {
                return Err((spec.missing_cloud_credentials_error)(missing_variables));
            }

            (
                UpstreamAuth::Basic {
                    username: username.expect("missing variables were checked").value,
                    api_token: api_token.expect("missing variables were checked").value,
                },
                None,
            )
        }
    } else if let Some(personal_token) = personal_token {
        (
            UpstreamAuth::Pat {
                personal_token: personal_token.value,
            },
            None,
        )
    } else if let Some(access_token) = oauth_access_token {
        (
            UpstreamAuth::OAuthAccessToken {
                access_token: access_token.value,
            },
            None,
        )
    } else if let Some(auth) = server_basic_auth(username, password, api_token, spec) {
        (auth, None)
    } else {
        return Err((spec.missing_personal_token_error)(
            spec.personal_token_variable,
        ));
    };

    let base_url = normalize_effective_base_url(
        parsed_url,
        deployment,
        spec.cloud_deployment,
        &auth,
        &oauth_cloud_id,
        spec.cloud_oauth_api_base_url,
    );
    let proxy = ProxyConfig::from_var_provider(
        get_var,
        spec.http_proxy_variable,
        spec.https_proxy_variable,
        spec.no_proxy_variable,
    )?;
    let custom_headers = CustomHeaders::from_var_provider_with_fallback(
        get_var,
        spec.custom_headers_variable,
        ENV_ATLASSIAN_CUSTOM_HEADERS,
    )?;
    let mtls = ClientTlsIdentityConfig::from_var_provider_with_fallback(
        get_var,
        spec.client_cert_variable,
        spec.client_key_variable,
        ENV_ATLASSIAN_CLIENT_CERT,
        ENV_ATLASSIAN_CLIENT_KEY,
    )?;
    let ssl_verify = parse_ssl_verify(
        optional_service_or_atlassian_var(
            get_var,
            spec.ssl_verify_variable,
            ENV_ATLASSIAN_SSL_VERIFY,
        )
        .as_ref()
        .map(|value| value.value.as_str()),
    );
    let timeout_seconds = parse_timeout_seconds(
        optional_service_or_atlassian_var(get_var, spec.timeout_variable, ENV_ATLASSIAN_TIMEOUT),
        spec.default_timeout_seconds,
        spec.invalid_timeout_error,
    )?;

    Ok(Some(ParsedUpstreamServiceConfig {
        base_url,
        deployment,
        auth,
        oauth_cloud_id,
        ssl_verify,
        proxy,
        custom_headers,
        mtls,
        timeout_seconds,
    }))
}

fn server_basic_auth<D>(
    username: Option<NamedEnvValue>,
    password: Option<NamedEnvValue>,
    api_token: Option<NamedEnvValue>,
    spec: &UpstreamServiceConfigSpec<D>,
) -> Option<UpstreamAuth> {
    let username = username?;
    let password_is_service_specific = password
        .as_ref()
        .is_some_and(|value| value.variable == spec.password_variable);
    let api_token_is_service_specific = api_token
        .as_ref()
        .is_some_and(|value| value.variable == spec.api_token_variable);
    let secret = if password_is_service_specific || !api_token_is_service_specific {
        password.or(api_token)
    } else {
        api_token.or(password)
    }?;

    Some(UpstreamAuth::Basic {
        username: username.value,
        api_token: secret.value,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NamedEnvValue {
    variable: &'static str,
    value: String,
}

fn optional_service_or_atlassian_var<F, E>(
    get_var: &mut F,
    service_variable: &'static str,
    atlassian_variable: &'static str,
) -> Option<NamedEnvValue>
where
    F: FnMut(&str) -> Result<String, E>,
{
    optional_named_var(get_var, service_variable)
        .or_else(|| optional_named_var(get_var, atlassian_variable))
}

fn optional_named_var<F, E>(get_var: &mut F, key: &'static str) -> Option<NamedEnvValue>
where
    F: FnMut(&str) -> Result<String, E>,
{
    get_var(key)
        .ok()
        .and_then(non_empty_trimmed)
        .map(|value| NamedEnvValue {
            variable: key,
            value,
        })
}

fn non_empty_trimmed(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn present_variables<const N: usize>(variables: [Option<&NamedEnvValue>; N]) -> Vec<&'static str> {
    variables
        .into_iter()
        .filter_map(|value| value.map(|value| value.variable))
        .collect()
}

fn service_specific_variable<'a>(
    value: Option<&'a NamedEnvValue>,
    service_variable: &'static str,
) -> Option<&'a NamedEnvValue> {
    value.filter(|value| value.variable == service_variable)
}

fn missing_service_variables<const N: usize>(
    variables: [(&'static str, Option<&NamedEnvValue>); N],
) -> Vec<&'static str> {
    variables
        .into_iter()
        .filter_map(|(name, value)| if value.is_none() { Some(name) } else { None })
        .collect()
}

fn parse_base_url(
    value: &str,
    variable: &'static str,
    invalid_url_error: fn(&'static str) -> ConfigError,
) -> Result<Url, ConfigError> {
    let url = Url::parse(value).map_err(|_| invalid_url_error(variable))?;

    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(invalid_url_error(variable));
    }

    Ok(url)
}

fn normalize_effective_base_url<D>(
    url: Url,
    deployment: D,
    cloud_deployment: D,
    auth: &UpstreamAuth,
    oauth_cloud_id: &Option<String>,
    cloud_oauth_api_base_url: fn(&str) -> String,
) -> String
where
    D: Eq,
{
    if deployment == cloud_deployment
        && matches!(auth, UpstreamAuth::OAuthAccessToken { .. })
        && let Some(cloud_id) = oauth_cloud_id
    {
        return cloud_oauth_api_base_url(cloud_id);
    }

    normalize_base_url(url)
}

fn normalize_base_url(mut url: Url) -> String {
    url.set_query(None);
    url.set_fragment(None);
    url.to_string().trim_end_matches('/').to_string()
}

fn parse_ssl_verify(value: Option<&str>) -> bool {
    !matches!(
        value.map(|value| value.trim().to_ascii_lowercase()),
        Some(value) if matches!(value.as_str(), "false" | "0" | "no" | "off")
    )
}

fn parse_timeout_seconds(
    value: Option<NamedEnvValue>,
    default_timeout_seconds: u64,
    invalid_timeout_error: fn(&'static str, String) -> ConfigError,
) -> Result<u64, ConfigError> {
    let Some(value) = value else {
        return Ok(default_timeout_seconds);
    };

    let seconds: u64 = value
        .value
        .parse()
        .map_err(|_| invalid_timeout_error(value.variable, value.value.clone()))?;

    if seconds == 0 {
        return Err(invalid_timeout_error(value.variable, value.value));
    }

    Ok(seconds)
}
