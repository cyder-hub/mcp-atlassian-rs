use std::{env, fmt::Display, path::PathBuf};

pub const ENV_FILE_VARIABLE: &str = "ENV_FILE";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DotenvSelection {
    Explicit(String),
    Environment(String),
    Default,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvLoadError {
    path: String,
    message: String,
}

impl EnvLoadError {
    #[cfg(test)]
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Display for EnvLoadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "failed to load env file {}: {}",
            self.path, self.message
        )
    }
}

impl std::error::Error for EnvLoadError {}

pub fn load_dotenv(explicit_env_file: Option<&str>) -> Result<Option<PathBuf>, EnvLoadError> {
    load_dotenv_with(
        explicit_env_file,
        |key| env::var(key),
        DotenvyLoader::default(),
    )
}

pub fn select_dotenv<F, E>(explicit_env_file: Option<&str>, mut get_var: F) -> DotenvSelection
where
    F: FnMut(&str) -> Result<String, E>,
{
    explicit_env_file
        .map(|path| DotenvSelection::Explicit(path.to_string()))
        .or_else(|| {
            get_var(ENV_FILE_VARIABLE)
                .ok()
                .map(DotenvSelection::Environment)
        })
        .unwrap_or(DotenvSelection::Default)
}

pub fn load_dotenv_with<F, E, L>(
    explicit_env_file: Option<&str>,
    get_var: F,
    mut loader: L,
) -> Result<Option<PathBuf>, EnvLoadError>
where
    F: FnMut(&str) -> Result<String, E>,
    L: DotenvLoader,
{
    match select_dotenv(explicit_env_file, get_var) {
        DotenvSelection::Explicit(path) | DotenvSelection::Environment(path) => loader
            .load_path(&path)
            .map(Some)
            .map_err(|message| EnvLoadError { path, message }),
        DotenvSelection::Default => Ok(loader.load_default().ok()),
    }
}

pub trait DotenvLoader {
    fn load_path(&mut self, path: &str) -> Result<PathBuf, String>;
    fn load_default(&mut self) -> Result<PathBuf, String>;
}

#[derive(Default)]
struct DotenvyLoader;

impl DotenvLoader for DotenvyLoader {
    fn load_path(&mut self, path: &str) -> Result<PathBuf, String> {
        dotenvy::from_filename(path).map_err(|error| error.to_string())
    }

    fn load_default(&mut self) -> Result<PathBuf, String> {
        dotenvy::dotenv().map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct FakeLoader {
        loaded_paths: Vec<String>,
        default_loaded: bool,
        fail_path: bool,
        fail_default: bool,
    }

    impl DotenvLoader for FakeLoader {
        fn load_path(&mut self, path: &str) -> Result<PathBuf, String> {
            self.loaded_paths.push(path.to_string());
            if self.fail_path {
                Err("missing file".to_string())
            } else {
                Ok(PathBuf::from(path))
            }
        }

        fn load_default(&mut self) -> Result<PathBuf, String> {
            self.default_loaded = true;
            if self.fail_default {
                Err("default missing".to_string())
            } else {
                Ok(PathBuf::from(".env"))
            }
        }
    }

    fn env_provider<'a>(
        pairs: &'a [(&'a str, &'a str)],
    ) -> impl FnMut(&str) -> Result<String, ()> + 'a {
        move |key| {
            pairs
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).to_string())
                .ok_or(())
        }
    }

    #[test]
    fn cli_env_loader_prefers_explicit_env_file_over_env_file_variable() {
        let selected = select_dotenv(
            Some("explicit.env"),
            env_provider(&[(ENV_FILE_VARIABLE, "from-env.env")]),
        );

        assert_eq!(
            selected,
            DotenvSelection::Explicit("explicit.env".to_string())
        );
    }

    #[test]
    fn cli_env_loader_prefers_env_file_variable_over_default_dotenv() {
        let selected = select_dotenv(None, env_provider(&[(ENV_FILE_VARIABLE, "from-env.env")]));

        assert_eq!(
            selected,
            DotenvSelection::Environment("from-env.env".to_string())
        );
    }

    #[test]
    fn cli_env_loader_uses_default_dotenv_when_no_explicit_source_exists() {
        let loaded = load_dotenv_with(None, env_provider(&[]), FakeLoader::default()).unwrap();

        assert_eq!(loaded, Some(PathBuf::from(".env")));
    }

    #[test]
    fn cli_env_loader_reports_explicit_env_file_failure() {
        let error = load_dotenv_with(
            Some("missing.env"),
            env_provider(&[]),
            FakeLoader {
                fail_path: true,
                ..FakeLoader::default()
            },
        )
        .unwrap_err();

        assert_eq!(error.path(), "missing.env");
    }

    #[test]
    fn cli_env_loader_ignores_missing_default_dotenv() {
        let loaded = load_dotenv_with(
            None,
            env_provider(&[]),
            FakeLoader {
                fail_default: true,
                ..FakeLoader::default()
            },
        )
        .unwrap();

        assert_eq!(loaded, None);
    }
}
