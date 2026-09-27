use clap::Parser;

use crate::command::Command;

/// Command-line client for Zeldoc.ai, the EU-sovereign LLM API.
///
/// Authenticates with the API key in ZELDOC_API_KEY, or with the key saved by
/// `zeldoc auth login` when that variable is unset.
#[derive(Parser)]
#[command(name = "zeldoc", version, about, long_about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }
}
