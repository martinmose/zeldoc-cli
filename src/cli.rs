use clap::Parser;

use crate::command::Command;
use crate::constants::environment;
use crate::credentials::data_transfer_objects::profile_name_dto::ProfileNameDTO;

/// Command-line client for Zeldoc.ai, the EU-sovereign LLM API.
///
/// Uses an API key saved with `zeldoc auth login`, under a profile name such
/// as a customer's. The key is picked in this order: `--profile` or
/// ZELDOC_PROFILE; a `.zeldoc-profile` file in the current folder or a folder
/// above it; ZELDOC_API_KEY; the default profile (`zeldoc auth use`). Run
/// `zeldoc auth status` to see which key is used and why.
#[derive(Parser)]
#[command(name = "zeldoc", version, about, long_about)]
pub struct Cli {
    /// Use the API key saved under this profile name
    #[arg(long, short = 'P', global = true, env = environment::PROFILE)]
    pub profile: Option<ProfileNameDTO>,

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
