use std::env;
use std::io::{self, IsTerminal, Read, Write};
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use clap::Subcommand;

use super::data_transfer_objects::saved_profile_dto::SavedProfileDTO;
use super::profiles_table;
use crate::api_request_error::ApiRequestError;
use crate::api_request_handler::ApiRequestHandler;
use crate::constants::{api, environment};
use crate::credentials::api_key_source::ApiKeySource;
use crate::credentials::credentials_error::CredentialsError;
use crate::credentials::credentials_store::{self, CredentialsStore};
use crate::credentials::data_transfer_objects::api_key_secret_dto::ApiKeySecretDTO;
use crate::credentials::data_transfer_objects::profile_dto::ProfileDTO;
use crate::credentials::data_transfer_objects::profile_name_dto::ProfileNameDTO;
use crate::credentials::profile_pin::ProfilePin;
use crate::credentials::profile_selection::ProfileSelection;
use crate::key_details::key_details_command::KeyDetailsCommand;
use crate::models::models_service::{ModelsService, ModelsServiceImpl};

const TOKEN_EXAMPLES: &str = "\
Examples:
  direnv .envrc in a project: export ZELDOC_API_KEY=\"$(zeldoc auth token)\"
  bash, zsh (Linux, macOS):   export ZELDOC_API_KEY=\"$(zeldoc auth token)\"
  fish:                       set -gx ZELDOC_API_KEY (zeldoc auth token)
  PowerShell (Windows):       $env:ZELDOC_API_KEY = zeldoc auth token";

// `zeldoc auth`. Its help text is on `Command::Auth`.
#[derive(Subcommand)]
pub enum AuthCommand {
    /// Save an API key for the CLI to use, under a profile name
    ///
    /// The key is saved as the profile given with `--profile`; without it, as
    /// the profile the folder's `.zeldoc-profile` file names, or as
    /// `default`. Logging in again with the same profile replaces its key.
    /// The first key saved becomes the default.
    ///
    /// Prompts for the key without showing it, or reads it from standard
    /// input when input is piped (`zeldoc auth login < key.txt`). The key is
    /// checked against Zeldoc.ai before it is saved, and it is stored in a
    /// file only your user can read. Never pass the key as an argument: it
    /// would end up in your shell history.
    Login,
    /// Delete a saved API key
    ///
    /// Deletes the profile the CLI would use here: the one given with
    /// `--profile`, the one the folder is pinned to, or the default.
    Logout,
    /// List the saved profiles, marking the default and the one pinned here
    List {
        /// Print the profiles as JSON, with the keys masked
        #[arg(long)]
        json: bool,
    },
    /// Make a saved profile the default, used where nothing else picks one
    ///
    /// A `.zeldoc-profile` file, `--profile` and ZELDOC_PROFILE still win
    /// over the default.
    Use {
        /// The profile to make the default
        name: ProfileNameDTO,
    },
    /// Pin the current folder, and the folders below it, to a profile
    ///
    /// Writes a `.zeldoc-profile` file with the profile name. In that folder
    /// the CLI then uses the key saved under that name, and `zeldoc auth
    /// token` prints it, so a direnv `.envrc` can hand it to other tools. The
    /// file holds no key and can be committed: everyone saves their own key
    /// under that name with `zeldoc auth login` in the folder.
    Pin {
        /// The profile to pin the folder to
        name: ProfileNameDTO,
    },
    /// Show which API key the CLI uses, why, and whether Zeldoc.ai accepts it
    ///
    /// Exits with status 1 when there is no key or the key is rejected.
    Status,
    /// Show the key fields your organization set on the API key
    ///
    /// Organizations can label their keys with fields such as a team, a
    /// project or whether the key is private. Shows the key's name and each
    /// of the organization's fields with this key's value, `-` where it has
    /// none. Uses the key picked by `--profile`, the folder's
    /// `.zeldoc-profile` file, ZELDOC_API_KEY or the default profile. The
    /// fields are set in the dashboard at app.zeldoc.ai; the CLI only reads
    /// them.
    Fields(KeyDetailsCommand),
    /// Print the API key the CLI uses, for tools that read ZELDOC_API_KEY
    ///
    /// Prints the key the CLI would use here: picked by `--profile`, the
    /// folder's `.zeldoc-profile` file, ZELDOC_API_KEY or the default
    /// profile. It does not create a new key and makes no network call. A
    /// program cannot set variables in the shell that started it, so use it
    /// to set ZELDOC_API_KEY yourself.
    #[command(after_long_help = TOKEN_EXAMPLES)]
    Token,
}

impl AuthCommand {
    pub fn run(self, profile: Option<&ProfileNameDTO>) -> Result<ExitCode> {
        match self {
            Self::Login => login(profile),
            Self::Logout => logout(profile),
            Self::List { json } => list(json),
            Self::Use { name } => use_profile(&name),
            Self::Pin { name } => pin(&name),
            Self::Status => status(profile),
            Self::Fields(command) => command.run(profile),
            Self::Token => token(profile),
        }
    }
}

fn login(requested: Option<&ProfileNameDTO>) -> Result<ExitCode> {
    let pin = credentials_store::find_pin()?;
    let name = match (requested, &pin) {
        (Some(name), _) => name.clone(),
        (None, Some(pin)) => pin.name.clone(),
        (None, None) => ProfileNameDTO::default_name(),
    };

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
    let mut file = store.load()?;
    file.profiles.insert(
        name.clone(),
        ProfileDTO {
            api_key: api_key.clone(),
        },
    );
    file.active_profile.get_or_insert_with(|| name.clone());
    store.save(&file)?;
    println!(
        "Saved {} as profile `{name}` in {}. It can use {model_count} models.",
        api_key.masked(),
        store.path().display()
    );

    let pinned_here = pin.as_ref().is_some_and(|pin| pin.name == name);
    if file.active_profile.as_ref() == Some(&name) {
        println!("`{name}` is the default profile.");
    } else if !pinned_here {
        println!(
            "Use it with `--profile {name}`, make it the default with `zeldoc auth use {name}`, \
             or pin a project folder to it with `zeldoc auth pin {name}`."
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn logout(requested: Option<&ProfileNameDTO>) -> Result<ExitCode> {
    let store = CredentialsStore::at_default_location()?;
    let mut file = store.load()?;
    let pin = credentials_store::find_pin()?;
    let Some(name) = requested
        .cloned()
        .or_else(|| pin.map(|pin| pin.name))
        .or_else(|| file.active_profile.clone())
    else {
        println!("No saved API key.");
        return Ok(ExitCode::SUCCESS);
    };
    if file.profiles.remove(&name).is_none() {
        return Err(CredentialsError::ProfileNotSaved { name }.into());
    }
    if file.active_profile.as_ref() == Some(&name) {
        file.active_profile = None;
    }
    store.save(&file)?;
    let file = store.load()?;
    println!("Deleted profile `{name}`.");
    match (&file.active_profile, file.profiles.len()) {
        (_, 0) => {}
        (Some(active), _) => println!("The default profile is `{active}`."),
        (None, _) => println!("No profile is the default now; pick one with `zeldoc auth use`."),
    }
    Ok(ExitCode::SUCCESS)
}

fn list(json: bool) -> Result<ExitCode> {
    let file = CredentialsStore::at_default_location()?.load()?;
    let pin = credentials_store::find_pin()?;
    let profiles: Vec<SavedProfileDTO> = file
        .profiles
        .iter()
        .map(|(name, profile)| SavedProfileDTO {
            profile: name.clone(),
            key: profile.api_key.masked(),
            default: file.active_profile.as_ref() == Some(name),
            pinned_here: pin.as_ref().is_some_and(|pin| &pin.name == name),
        })
        .collect();

    let mut stdout = io::stdout().lock();
    if json {
        writeln!(stdout, "{}", serde_json::to_string_pretty(&profiles)?)?;
        return Ok(ExitCode::SUCCESS);
    }
    if profiles.is_empty() {
        eprintln!("No saved API keys. Run `zeldoc auth login`.");
    } else {
        profiles_table::write_table(&mut stdout, &profiles)?;
    }
    if let Some(pin) = pin
        && !file.profiles.contains_key(&pin.name)
    {
        eprintln!(
            "{} picks profile `{}`, which is not saved. Run `zeldoc auth login` here to save it.",
            pin.path.display(),
            pin.name
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn use_profile(name: &ProfileNameDTO) -> Result<ExitCode> {
    let store = CredentialsStore::at_default_location()?;
    let mut file = store.load()?;
    if !file.profiles.contains_key(name) {
        return Err(CredentialsError::ProfileNotSaved { name: name.clone() }.into());
    }
    file.active_profile = Some(name.clone());
    store.save(&file)?;
    println!("`{name}` is the default profile.");
    if let Some(pin) = credentials_store::find_pin()?
        && &pin.name != name
    {
        println!(
            "This folder still uses `{}`, as {} says.",
            pin.name,
            pin.path.display()
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn pin(name: &ProfileNameDTO) -> Result<ExitCode> {
    let directory = env::current_dir().context("could not find the current folder")?;
    let pin = ProfilePin::write(&directory, name)?;
    println!(
        "Wrote {}: the CLI uses profile `{name}` in this folder and below it.",
        pin.path.display()
    );
    let file = CredentialsStore::at_default_location()?.load()?;
    if !file.profiles.contains_key(name) {
        println!("No key is saved as `{name}` yet. Run `zeldoc auth login` here to save one.");
    }
    Ok(ExitCode::SUCCESS)
}

fn status(requested: Option<&ProfileNameDTO>) -> Result<ExitCode> {
    let Some(api_key) = credentials_store::resolve_api_key(requested)? else {
        println!(
            "Not logged in. Run `zeldoc auth login`, or set {}.",
            environment::API_KEY
        );
        return Ok(ExitCode::FAILURE);
    };

    let origin = match &api_key.source {
        ApiKeySource::Environment => format!("from {}", environment::API_KEY),
        ApiKeySource::Profile { name, selection } => match selection {
            ProfileSelection::Requested => format!("profile `{name}`, asked for"),
            ProfileSelection::Pin(pin) => {
                format!("profile `{name}`, pinned by {}", pin.path.display())
            }
            ProfileSelection::Active => format!("profile `{name}`, the default"),
        },
    };
    println!("API key: {} ({origin})", api_key.secret.masked());
    match &api_key.source {
        ApiKeySource::Environment => {
            let file = CredentialsStore::at_default_location()?.load()?;
            if let Some(active) = file.active_profile {
                println!(
                    "The default profile `{active}` is not used while {} is set.",
                    environment::API_KEY
                );
            }
        }
        ApiKeySource::Profile { .. } => {
            if credentials_store::api_key_from_environment().is_some() {
                println!(
                    "{} is set but not used: a profile was picked.",
                    environment::API_KEY
                );
            }
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

fn token(requested: Option<&ProfileNameDTO>) -> Result<ExitCode> {
    let api_key = credentials_store::require_api_key(requested)?;
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
