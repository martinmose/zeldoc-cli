use std::fmt;

use serde::{Deserialize, Serialize};

/// What kind of model it is, in the gateway's words. A kind the CLI does not
/// know yet is kept as `Unknown` instead of failing the whole catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelModeDTO {
    Chat,
    Responses,
    Embedding,
    ImageGeneration,
    AudioTranscription,
    Realtime,
    #[serde(untagged)]
    Unknown(String),
}

impl fmt::Display for ModelModeDTO {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Chat => "chat",
            Self::Responses => "responses",
            Self::Embedding => "embedding",
            Self::ImageGeneration => "image_generation",
            Self::AudioTranscription => "audio_transcription",
            Self::Realtime => "realtime",
            Self::Unknown(mode) => mode,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_modes_round_trip_as_their_gateway_names() {
        for (json, mode) in [
            ("\"chat\"", ModelModeDTO::Chat),
            ("\"image_generation\"", ModelModeDTO::ImageGeneration),
            ("\"audio_transcription\"", ModelModeDTO::AudioTranscription),
        ] {
            assert_eq!(serde_json::from_str::<ModelModeDTO>(json).unwrap(), mode);
            assert_eq!(serde_json::to_string(&mode).unwrap(), json);
            assert_eq!(format!("\"{mode}\""), json);
        }
    }

    #[test]
    fn unknown_modes_are_kept() {
        let mode: ModelModeDTO = serde_json::from_str("\"video_generation\"").unwrap();
        assert_eq!(mode, ModelModeDTO::Unknown("video_generation".to_string()));
        assert_eq!(
            serde_json::to_string(&mode).unwrap(),
            "\"video_generation\""
        );
        assert_eq!(mode.to_string(), "video_generation");
    }
}
