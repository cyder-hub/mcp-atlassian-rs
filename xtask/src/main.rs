mod acceptance;
mod redaction;
mod smoke;

use clap::{Args, Parser, Subcommand};

type XtaskResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Parser)]
#[command(
    name = "xtask",
    bin_name = "cargo xtask",
    about = "Development-only automation for mcp-atlassian-rs",
    subcommand_required = true,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: XtaskCommand,
}

#[derive(Debug, Subcommand)]
enum XtaskCommand {
    /// Run local smoke checks against Rust mock Atlassian services.
    Smoke(SmokeCli),
    /// Run real acceptance checks against configured test Atlassian services.
    Acceptance(acceptance::AcceptanceCommand),
}

#[derive(Debug, Args, PartialEq, Eq)]
struct SmokeCli {
    #[command(subcommand)]
    target: Option<SmokeTarget>,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
enum SmokeTarget {
    /// Run Jira smoke checks.
    Jira(smoke::SmokeArgs),
    /// Run Confluence smoke checks.
    Confluence(smoke::SmokeArgs),
}

impl SmokeTarget {
    fn into_command(self) -> smoke::SmokeCommand {
        match self {
            Self::Jira(args) => args.into_command(smoke::SmokeService::Jira),
            Self::Confluence(args) => args.into_command(smoke::SmokeService::Confluence),
        }
    }
}

#[tokio::main]
async fn main() -> XtaskResult<()> {
    let cli = Cli::parse();

    match cli.command {
        XtaskCommand::Smoke(SmokeCli { target }) => {
            let Some(target) = target else {
                let exit_code = smoke::run_all().await?;
                if exit_code != 0 {
                    std::process::exit(exit_code);
                }
                return Ok(());
            };
            let command = target.into_command();
            let exit_code = smoke::run(command).await?;
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
            Ok(())
        }
        XtaskCommand::Acceptance(command) => {
            let exit_code = acceptance::run(command).await?;
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_smoke_command() {
        let cli = Cli::try_parse_from(["xtask", "smoke", "jira", "restricted"]).unwrap();
        let XtaskCommand::Smoke(SmokeCli {
            target: Some(SmokeTarget::Jira(args)),
        }) = cli.command
        else {
            panic!("expected jira smoke command");
        };
        assert_eq!(args.mode, smoke::SmokeMode::Restricted);
    }

    #[test]
    fn parses_smoke_all_command() {
        let cli = Cli::try_parse_from(["xtask", "smoke"]).unwrap();
        assert!(matches!(
            cli.command,
            XtaskCommand::Smoke(SmokeCli { target: None })
        ));
    }

    #[test]
    fn parses_acceptance_command() {
        let cli = Cli::try_parse_from(["xtask", "acceptance", "jira", "--preflight"]).unwrap();
        let XtaskCommand::Acceptance(command) = cli.command else {
            panic!("expected acceptance command");
        };
        assert_eq!(command.mode, acceptance::AcceptanceMode::Jira);
        assert!(command.preflight);
        assert_eq!(command.run, None);
    }

    #[test]
    fn rejects_unknown_command() {
        assert!(Cli::try_parse_from(["xtask", "unknown"]).is_err());
    }
}
