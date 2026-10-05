use super::data_transfer_objects::profile_name_dto::ProfileNameDTO;
use super::profile_selection::ProfileSelection;

/// Where the API key the CLI uses came from.
pub enum ApiKeySource {
    /// The ZELDOC_API_KEY environment variable.
    Environment,
    /// A profile saved by `zeldoc auth login`, and what picked it.
    Profile {
        name: ProfileNameDTO,
        selection: ProfileSelection,
    },
}
