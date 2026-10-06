use std::io::{self, Write};
use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use super::data_transfer_objects::profile_key_details_dto::ProfileKeyDetailsDTO;
use super::key_details_service::{KeyDetailsService, KeyDetailsServiceImpl};
use super::key_details_text;
use crate::api_request_handler::ApiRequestHandler;
use crate::credentials::credentials_store::{self, CredentialsStore};
use crate::credentials::data_transfer_objects::profile_name_dto::ProfileNameDTO;

// `zeldoc auth fields`. Its help text is on `AuthCommand::Fields`.
#[derive(Args)]
pub struct KeyDetailsCommand {
    /// Show the fields of every saved profile's key instead of one key.
    /// ZELDOC_API_KEY and `--profile` are not used
    #[arg(long)]
    all: bool,

    /// Print the key's name and fields as JSON, with each value as the API
    /// sends it (a select field's option key, with its label next to it)
    #[arg(long)]
    json: bool,
}

impl KeyDetailsCommand {
    pub fn run(self, profile: Option<&ProfileNameDTO>) -> Result<ExitCode> {
        if self.all {
            return self.run_for_all_profiles();
        }
        let api_key = credentials_store::require_api_key(profile)?;
        let service = KeyDetailsServiceImpl::new(ApiRequestHandler::new(api_key.secret)?);
        let details = service.details()?;

        let mut stdout = io::stdout().lock();
        if self.json {
            writeln!(stdout, "{}", serde_json::to_string_pretty(&details)?)?;
        } else {
            key_details_text::write_details(&mut stdout, &details)?;
        }
        Ok(ExitCode::SUCCESS)
    }

    fn run_for_all_profiles(&self) -> Result<ExitCode> {
        let file = CredentialsStore::at_default_location()?.load()?;
        if file.profiles.is_empty() {
            eprintln!("No saved API keys. Run `zeldoc auth login`.");
            return Ok(ExitCode::FAILURE);
        }

        let mut profiles = Vec::new();
        for (name, saved) in &file.profiles {
            let service =
                KeyDetailsServiceImpl::new(ApiRequestHandler::new(saved.api_key.clone())?);
            let (details, error) = match service.details() {
                Ok(details) => (Some(details), None),
                Err(error) => (None, Some(error.to_string())),
            };
            profiles.push(ProfileKeyDetailsDTO {
                profile: name.clone(),
                details,
                error,
            });
        }

        let mut stdout = io::stdout().lock();
        if self.json {
            writeln!(stdout, "{}", serde_json::to_string_pretty(&profiles)?)?;
        } else {
            key_details_text::write_profile_details(&mut stdout, &profiles)?;
        }
        let mut failed = false;
        for profile in &profiles {
            if let Some(error) = &profile.error {
                eprintln!("error: profile `{}`: {error}", profile.profile);
                failed = true;
            }
        }
        Ok(if failed {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        })
    }
}
