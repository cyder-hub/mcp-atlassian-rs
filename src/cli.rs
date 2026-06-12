use std::{fmt::Display, sync::Arc};

use clap::{Args, Subcommand};
#[cfg(test)]
use clap::{Command, FromArgMatches};

use crate::{
    context::AppContext,
    operations::{
        OperationError, OperationResult,
        output::{CliOutputOptions, render_error, render_success},
    },
};

pub mod confluence;
#[cfg(test)]
pub mod contract;
pub mod gitlab;
pub mod jira;

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct CliArgs {
    #[arg(long, value_name = "path", global = true)]
    pub env_file: Option<String>,
    #[arg(long, global = true)]
    pub json: bool,
    #[arg(long, global = true, requires = "json")]
    pub pretty: bool,
    #[command(subcommand)]
    pub command: ProviderCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum ProviderCommand {
    Jira(jira::JiraArgs),
    Confluence(confluence::ConfluenceArgs),
    Gitlab(gitlab::GitlabArgs),
}

#[derive(Debug)]
pub struct CliRunError {
    message: String,
    exit_code: i32,
}

impl CliRunError {
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }
}

impl Display for CliRunError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for CliRunError {}

pub async fn run(args: CliArgs, context: Arc<AppContext>) -> Result<(), CliRunError> {
    let options = output_options(&args);
    let result = match args.command {
        ProviderCommand::Jira(args) => jira::execute(args, &context).await,
        ProviderCommand::Confluence(args) => confluence::execute(args, &context).await,
        ProviderCommand::Gitlab(args) => gitlab::execute(args, &context).await,
    };

    match result {
        Ok(result) if result.is_error => Err(render_business_error(result, options)),
        Ok(result) => {
            let output = render_success(&result, options).map_err(rendering_error)?;
            if !output.stdout.is_empty() {
                println!("{}", output.stdout);
            }
            if !output.stderr.is_empty() {
                eprintln!("{}", output.stderr);
            }
            Ok(())
        }
        Err(error) => Err(render_operation_error(&error, options)),
    }
}

pub fn render_config_error(args: &CliArgs, message: impl Into<String>) -> CliRunError {
    render_operation_error(&OperationError::config(message), output_options(args))
}

fn output_options(args: &CliArgs) -> CliOutputOptions {
    CliOutputOptions {
        json: args.json,
        pretty: args.pretty,
    }
}

fn render_business_error(result: OperationResult, options: CliOutputOptions) -> CliRunError {
    let message = serde_json::to_string(&result.value)
        .unwrap_or_else(|_| "operation returned a business error".to_string());
    render_operation_error(&OperationError::business(message), options)
}

fn render_operation_error(error: &OperationError, options: CliOutputOptions) -> CliRunError {
    match render_error(error, options) {
        Ok(output) => CliRunError {
            message: output.stderr,
            exit_code: output.exit_code,
        },
        Err(error) => rendering_error(error),
    }
}

fn rendering_error(error: serde_json::Error) -> CliRunError {
    CliRunError {
        message: format!("failed to render CLI output: {error}"),
        exit_code: 5,
    }
}

#[cfg(test)]
pub fn cli_command() -> Command {
    <CliArgs as Args>::augment_args(Command::new("cli").no_binary_name(true))
}

#[cfg(test)]
pub fn parse_cli_args<I, S>(args: I) -> Result<CliArgs, clap::Error>
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString> + Clone,
{
    let mut matches = cli_command().try_get_matches_from(args)?;
    CliArgs::from_arg_matches_mut(&mut matches)
}

#[cfg(test)]
mod tests {
    use clap::error::ErrorKind;

    use super::*;

    #[test]
    fn cli_parse_accepts_json_and_pretty_together() {
        let args = parse_cli_args(["--json", "--pretty", "jira", "project", "list"]).unwrap();

        assert!(args.json);
        assert!(args.pretty);
    }

    #[test]
    fn cli_parse_rejects_pretty_without_json() {
        let error = parse_cli_args(["--pretty", "jira", "project", "list"]).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn cli_parse_accepts_provider_subcommand_help() {
        let error = parse_cli_args(["jira", "issue", "--help"]).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::DisplayHelp);
    }

    #[test]
    fn cli_config_error_uses_json_contract_and_config_exit_code() {
        let args = parse_cli_args(["--json", "jira", "project", "list"]).unwrap();
        let error = render_config_error(&args, "invalid JIRA_TIMEOUT value `abc`");

        assert_eq!(error.exit_code(), 3);
        assert_eq!(
            error.to_string(),
            r#"{"error":{"category":"config","message":"invalid JIRA_TIMEOUT value `abc`"},"success":false}"#
        );
    }
}
