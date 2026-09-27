use std::io::{self, Write};
use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use super::data_transfer_objects::model_dto::ModelDTO;
use super::models_service::{ModelsService, ModelsServiceImpl};
use super::models_table;
use crate::api_request_handler::ApiRequestHandler;
use crate::credentials::credentials_store;

// `zeldoc models`. Its help text is on `Command::Models`.
#[derive(Args)]
pub struct ModelsCommand {
    /// Only list models of this kind, e.g. chat, responses, embedding,
    /// image_generation or audio_transcription
    #[arg(long)]
    mode: Option<String>,

    /// Print the models as JSON, with prices and capabilities in full
    #[arg(long)]
    json: bool,
}

impl ModelsCommand {
    pub fn run(self) -> Result<ExitCode> {
        let api_key = credentials_store::require_api_key()?;
        let service = ModelsServiceImpl::new(ApiRequestHandler::new(api_key.secret)?);
        let models = filter_by_mode(service.list()?, self.mode.as_deref());
        if models.is_empty() {
            eprintln!("No models match.");
            return Ok(ExitCode::SUCCESS);
        }

        let mut stdout = io::stdout().lock();
        if self.json {
            writeln!(stdout, "{}", serde_json::to_string_pretty(&models)?)?;
        } else {
            models_table::write_table(&mut stdout, &models)?;
        }
        Ok(ExitCode::SUCCESS)
    }
}

fn filter_by_mode(models: Vec<ModelDTO>, mode: Option<&str>) -> Vec<ModelDTO> {
    let Some(mode) = mode else {
        return models;
    };
    models
        .into_iter()
        .filter(|model| {
            model
                .mode
                .as_ref()
                .is_some_and(|model_mode| model_mode.to_string().eq_ignore_ascii_case(mode))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::data_transfer_objects::model_id_dto::ModelIdDTO;
    use crate::models::test_catalog;

    #[test]
    fn mode_filter_ignores_case_and_skips_models_without_a_mode() {
        let ids: Vec<ModelIdDTO> = filter_by_mode(test_catalog::models(), Some("CHAT"))
            .into_iter()
            .map(|model| model.id)
            .collect();
        assert_eq!(ids, [ModelIdDTO::from("zdev".to_string())]);
    }
}
