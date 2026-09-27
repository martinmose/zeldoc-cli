use std::process::ExitCode;

use anyhow::Result;
use clap::Subcommand;

use crate::auth::auth_command::AuthCommand;
use crate::models::models_command::ModelsCommand;
use crate::search::search_command::SearchCommand;

// Help text lives on the variants: clap shows it for `zeldoc <command> --help`.
#[derive(Subcommand)]
pub enum Command {
    /// Save, check, or remove the API key the CLI uses
    #[command(subcommand)]
    Auth(AuthCommand),
    /// List the models your API key can use
    Models(ModelsCommand),
    /// Search the web through Zeldoc.ai's search endpoint
    ///
    /// Queries go to Zeldoc.ai's own SearXNG instance with your API key.
    /// General searches use Staan, an EU-based provider; `--categories it` and
    /// `--engines` can reach GitHub, Stack Overflow and other services outside
    /// the EU. Keep personal and confidential information out of queries.
    /// Searches count towards your key's rate limit but are not billed.
    Search(SearchCommand),
}

impl Command {
    pub fn run(self) -> Result<ExitCode> {
        match self {
            Self::Auth(command) => command.run(),
            Self::Models(command) => command.run(),
            Self::Search(command) => command.run(),
        }
    }
}
