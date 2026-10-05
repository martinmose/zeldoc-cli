use std::io::{self, Write};

use super::data_transfer_objects::saved_profile_dto::SavedProfileDTO;
use crate::text_table::{self, Align};

const COLUMNS: [(&str, Align); 3] = [
    ("PROFILE", Align::Left),
    ("KEY", Align::Left),
    ("", Align::Left),
];

/// The saved profiles as a table, marking the default and the one pinned for
/// the current folder.
pub fn write_table(out: &mut impl Write, profiles: &[SavedProfileDTO]) -> io::Result<()> {
    let rows: Vec<[String; 3]> = profiles
        .iter()
        .map(|profile| {
            let notes: Vec<&str> = [
                (profile.default, "default"),
                (profile.pinned_here, "pinned here"),
            ]
            .into_iter()
            .filter_map(|(applies, note)| applies.then_some(note))
            .collect();
            [
                profile.profile.to_string(),
                profile.key.clone(),
                notes.join(", "),
            ]
        })
        .collect();
    text_table::write_table(out, &COLUMNS, &rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_marks_the_default_and_the_pinned_profile() {
        let profiles = [
            SavedProfileDTO {
                profile: "acme".parse().unwrap(),
                key: "sk-…a1b2".to_string(),
                default: false,
                pinned_here: true,
            },
            SavedProfileDTO {
                profile: "globex".parse().unwrap(),
                key: "sk-…c3d4".to_string(),
                default: true,
                pinned_here: false,
            },
        ];
        let mut out = Vec::new();
        write_table(&mut out, &profiles).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "\
PROFILE  KEY
acme     sk-…a1b2  pinned here
globex   sk-…c3d4  default
"
        );
    }
}
