use std::io::{self, IsTerminal, Read, Write};
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use clap::Subcommand;

use crate::api_request_error::ApiRequestError;
use crate::api_request_handler::ApiRequestHandler;
use crate::constants::{api, environment};
use crate::credentials::api_key_source::ApiKeySource;
use crate::credentials::credentials_error::CredentialsError;
use crate::credentials::credentials_store::{self, CredentialsStore};
use crate::credentials::data_transfer_objects::api_key_secret_dto::ApiKeySecretDTO;
use crate::models::models_service::{ModelsService, ModelsServiceImpl};

const TOKEN_EXAMPLES: &str = "\
Examples:
  bash, zsh (Linux, macOS):  export ZELDOC_API_KEY=\"$(zeldoc auth token)\"
  fish:                      set -gx ZELDOC_API_KEY (zeldoc auth token)
  PowerShell (Windows):      $env:ZELDOC_API_KEY = zeldoc auth token";

// `zeldoc auth`. Its help text is on `Command::Auth`.
#[derive(Subcommand)]
pub enum AuthCommand {
    /// Save an API key for the CLI to use
    ///
    /// Prompts for the key without showing it, or reads it from standard
    /// input when input is piped (`zeldoc auth login < key.txt`). The key is
    /// checked against Zeldoc.ai before it is saved, and it is stored in a
    /// file only your user can read. Never pass the key as an argument: it
    /// would end up in your shell history.
    Login,
    /// Delete the saved API key
    Logout,
    /// Show which API key the CLI uses and whether Zeldoc.ai accepts it
    ///
    /// Exits with status 1 when there is no key or the key is rejected.
    Status,
    /// Print the API key the CLI uses, for tools that read ZELDOC_API_KEY
    ///
    /// Prints the key from ZELDOC_API_KEY, or the key saved by `zeldoc auth
    /// login`. It does not create a new key and makes no network call. A
    /// program cannot set variables in the shell that started it, so use it
    /// to set ZELDOC_API_KEY yourself; put the line in your shell profile to
    /// set it in every new shell.
    #[command(after_long_help = TOKEN_EXAMPLES)]
    Token,
}

impl AuthCommand {
    pub fn run(self) -> Result<ExitCode> {
        match self {
            Self::Login => login(),
            Self::Logout => logout(),
            Self::Status => status(),
            Self::Token => token(),
        }
    }
}

fn login() -> Result<ExitCode> {
    let api_key = read_api_key()?;
    if api_key.is_empty() {
        bail!(
            "no API key given. Need a key? See {}",
            api::API_KEY_DOCS_URL
        );
    }

    let model_count = match count_models(&api_key) {
        Err(error) if error.is_auth_failure() => {
            bail!("Zeldoc.ai rejected this key, so it was not saved")
        }
        result => result.context("could not check the key with Zeldoc.ai, so it was not saved")?,
    };

    let store = CredentialsStore::at_default_location()?;
    store.save(&api_key)?;
    println!(
        "Saved {} to {}. It can use {model_count} models.",
        api_key.masked(),
        store.path().display()
    );
    if credentials_store::api_key_from_environment().is_some() {
        println!(
            "Note: {} is set in this shell and takes precedence over the saved key.",
            environment::API_KEY
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn logout() -> Result<ExitCode> {
    let store = CredentialsStore::at_default_location()?;
    if store.remove()? {
        println!("Deleted the saved API key from {}.", store.path().display());
    } else {
        println!("No saved API key.");
    }
    if credentials_store::api_key_from_environment().is_some() {
        println!(
            "Note: {} is still set in this shell, so the CLI keeps using it.",
            environment::API_KEY
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn status() -> Result<ExitCode> {
    let Some(api_key) = credentials_store::resolve_api_key()? else {
        println!(
            "Not logged in. Run `zeldoc auth login`, or set {}.",
            environment::API_KEY
        );
        return Ok(ExitCode::FAILURE);
    };

    let origin = match &api_key.source {
        ApiKeySource::Environment => format!("from {}", environment::API_KEY),
        ApiKeySource::File(path) => format!("saved in {}", path.display()),
    };
    println!("API key: {} ({origin})", api_key.secret.masked());
    if let ApiKeySource::Environment = api_key.source {
        let store = CredentialsStore::at_default_location()?;
        if store.load()?.is_some() {
            println!(
                "The key saved in {} is ignored while {} is set.",
                store.path().display(),
                environment::API_KEY
            );
        }
    }

    match count_models(&api_key.secret) {
        Ok(model_count) => {
            println!("Zeldoc.ai accepts the key. It can use {model_count} models.");
            Ok(ExitCode::SUCCESS)
        }
        Err(error) if error.is_auth_failure() => {
            println!(
                "Zeldoc.ai rejects the key. Create a new one: {}",
                api::API_KEY_DOCS_URL
            );
            Ok(ExitCode::FAILURE)
        }
        Err(error) => {
            Err(anyhow::Error::new(error).context("could not check the key with Zeldoc.ai"))
        }
    }
}

fn token() -> Result<ExitCode> {
    let api_key = credentials_store::resolve_api_key()?.ok_or(CredentialsError::NoApiKey)?;
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{}", api_key.secret.expose())?;
    Ok(ExitCode::SUCCESS)
}

/// Read the key from a hidden prompt, or from standard input when it is piped.
fn read_api_key() -> Result<ApiKeySecretDTO> {
    let api_key = if io::stdin().is_terminal() {
        eprintln!("Need a key? See {}", api::API_KEY_DOCS_URL);
        rpassword::prompt_password("Paste your Zeldoc.ai API key: ")
            .context("could not read the API key")?
    } else {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .context("could not read the API key from standard input")?;
        input
    };
    Ok(ApiKeySecretDTO::new(api_key.trim().to_string()))
}

fn count_models(api_key: &ApiKeySecretDTO) -> Result<usize, ApiRequestError> {
    let service = ModelsServiceImpl::new(ApiRequestHandler::new(api_key.clone())?);
    Ok(service.list()?.len())
}
