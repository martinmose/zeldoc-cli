use std::io::{self, Write};
use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use super::request_parameters::search_parameters::SearchParameters;
use super::request_parameters::time_range::TimeRange;
use super::search_results_text;
use super::search_service::{SearchService, SearchServiceImpl};
use crate::api_request_handler::ApiRequestHandler;
use crate::credentials::credentials_store;
use crate::credentials::data_transfer_objects::profile_name_dto::ProfileNameDTO;

// `zeldoc search`. Its help text is on `Command::Search`.
#[derive(Args)]
pub struct SearchCommand {
    /// What to search for. Several words need no quotes.
    #[arg(required = true, num_args = 1..)]
    query: Vec<String>,

    /// Only use these engines, comma-separated: staan, github, stackoverflow,
    /// mdn, "arch linux wiki", pypi, npm, crates.io
    #[arg(long, short, value_delimiter = ',')]
    engines: Vec<String>,

    /// Search these categories instead of general web search, comma-separated,
    /// e.g. `it` for the developer engines
    #[arg(long, value_delimiter = ',')]
    categories: Vec<String>,

    /// Prefer results from this period (engine-dependent, not guaranteed)
    #[arg(long, value_enum)]
    time_range: Option<TimeRange>,

    /// Prefer results in this language, e.g. `de` (engine-dependent)
    #[arg(long)]
    language: Option<String>,

    /// Show at most this many results
    #[arg(long, short = 'n')]
    limit: Option<usize>,

    /// Print the results as a JSON array instead of text
    #[arg(long)]
    json: bool,
}

impl SearchCommand {
    pub fn run(self, profile: Option<&ProfileNameDTO>) -> Result<ExitCode> {
        let api_key = credentials_store::require_api_key(profile)?;
        let service = SearchServiceImpl::new(ApiRequestHandler::new(api_key.secret)?);
        let mut results = service.search(&self.parameters())?;
        if let Some(limit) = self.limit {
            results.truncate(limit);
        }
        if results.is_empty() {
            eprintln!("No results.");
            return Ok(ExitCode::SUCCESS);
        }

        let mut stdout = io::stdout().lock();
        if self.json {
            writeln!(stdout, "{}", serde_json::to_string_pretty(&results)?)?;
        } else {
            search_results_text::write_results(&mut stdout, &results)?;
        }
        Ok(ExitCode::SUCCESS)
    }

    fn parameters(&self) -> SearchParameters {
        SearchParameters {
            query: self.query.join(" "),
            time_range: self.time_range,
            language: self.language.clone(),
            categories: comma_separated(&self.categories),
            engines: comma_separated(&self.engines),
        }
    }
}

fn comma_separated(values: &[String]) -> Option<String> {
    let values: Vec<&str> = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .collect();
    (!values.is_empty()).then(|| values.join(","))
}

#[cfg(test)]
mod tests {
    use clap::FromArgMatches;

    use super::*;

    fn command(arguments: &[&str]) -> SearchCommand {
        let matches = SearchCommand::augment_args(clap::Command::new("search"))
            .try_get_matches_from(std::iter::once("search").chain(arguments.iter().copied()))
            .unwrap();
        SearchCommand::from_arg_matches(&matches).unwrap()
    }

    #[test]
    fn words_are_joined_into_one_query() {
        let parameters = command(&["podman", "rootless", "ports"]).parameters();
        assert_eq!(parameters.query, "podman rootless ports");
        assert!(parameters.engines.is_none() && parameters.categories.is_none());
    }

    #[test]
    fn repeated_and_comma_separated_engines_are_joined() {
        let parameters = command(&[
            "--engines",
            "github, arch linux wiki",
            "-e",
            "crates.io",
            "--categories",
            "it",
            "query",
        ])
        .parameters();
        assert_eq!(
            parameters.engines.as_deref(),
            Some("github,arch linux wiki,crates.io")
        );
        assert_eq!(parameters.categories.as_deref(), Some("it"));
    }
}
