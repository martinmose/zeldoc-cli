use std::io;
use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use super::update_outcome_text;
use super::update_service::{UpdateService, UpdateServiceImpl};

// `zeldoc update`. Its help text is on `Command::Update`.
#[derive(Args)]
pub struct UpdateCommand {
    /// Only report whether a newer release exists; install nothing
    #[arg(long)]
    check: bool,
}

impl UpdateCommand {
    pub fn run(self) -> Result<ExitCode> {
        let service = UpdateServiceImpl::new();
        let outcome = if self.check {
            service.check()?
        } else {
            service.update()?
        };
        update_outcome_text::write_outcome(&mut io::stdout().lock(), &outcome)?;
        Ok(ExitCode::SUCCESS)
    }
}
