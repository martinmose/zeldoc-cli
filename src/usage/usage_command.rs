use std::io::{self, Write};
use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use super::data_transfer_objects::profile_usage_dto::ProfileUsageDTO;
use super::request_parameters::usage_period::UsagePeriod;
use super::usage_report_text;
use super::usage_service::{UsageService, UsageServiceImpl};
use crate::api_request_handler::ApiRequestHandler;
use crate::credentials::credentials_store::{self, CredentialsStore};
use crate::credentials::data_transfer_objects::profile_name_dto::ProfileNameDTO;

// `zeldoc usage`. Its help text is on `Command::Usage`.
#[derive(Args)]
pub struct UsageCommand {
    /// The period to report, in UTC
    #[arg(long, short, value_enum, default_value_t)]
    period: UsagePeriod,

    /// Report every saved profile instead of one key: one row per profile,
    /// with a total. ZELDOC_API_KEY and `--profile` are not used
    #[arg(long)]
    all: bool,

    /// Print the report as JSON, with exact costs and cache writes
    #[arg(long)]
    json: bool,
}

impl UsageCommand {
    pub fn run(self, profile: Option<&ProfileNameDTO>) -> Result<ExitCode> {
        if self.all {
            return self.run_for_all_profiles();
        }
        let api_key = credentials_store::require_api_key(profile)?;
        let service = UsageServiceImpl::new(ApiRequestHandler::new(api_key.secret)?);
        let report = service.report(self.period)?;

        let mut stdout = io::stdout().lock();
        if self.json {
            writeln!(stdout, "{}", serde_json::to_string_pretty(&report)?)?;
            return Ok(ExitCode::SUCCESS);
        }
        if report.models.is_empty() {
            eprintln!("No usage in this period.");
        }
        usage_report_text::write_report(&mut stdout, self.period, &report)?;
        Ok(ExitCode::SUCCESS)
    }

    fn run_for_all_profiles(&self) -> Result<ExitCode> {
        let file = CredentialsStore::at_default_location()?.load()?;
        if file.profiles.is_empty() {
            eprintln!("No saved API keys. Run `zeldoc auth login`.");
            return Ok(ExitCode::FAILURE);
        }

        let mut usages = Vec::new();
        for (name, saved) in &file.profiles {
            let service = UsageServiceImpl::new(ApiRequestHandler::new(saved.api_key.clone())?);
            let (report, error) = match service.report(self.period) {
                Ok(report) => (Some(report), None),
                Err(error) => (None, Some(error.to_string())),
            };
            usages.push(ProfileUsageDTO {
                profile: name.clone(),
                report,
                error,
            });
        }

        let mut stdout = io::stdout().lock();
        if self.json {
            writeln!(stdout, "{}", serde_json::to_string_pretty(&usages)?)?;
        } else {
            usage_report_text::write_profile_usages(&mut stdout, self.period, &usages)?;
        }
        let failed: Vec<&ProfileUsageDTO> = usages
            .iter()
            .filter(|usage| usage.error.is_some())
            .collect();
        for usage in &failed {
            eprintln!(
                "error: profile `{}`: {}",
                usage.profile,
                usage.error.as_deref().unwrap_or_default()
            );
        }
        Ok(if failed.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        })
    }
}
