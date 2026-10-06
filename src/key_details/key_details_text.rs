use std::io::{self, Write};

use super::data_transfer_objects::key_details_dto::KeyDetailsDTO;
use super::data_transfer_objects::key_field_dto::KeyFieldDTO;
use super::data_transfer_objects::key_field_value_dto::KeyFieldValueDTO;
use super::data_transfer_objects::profile_key_details_dto::ProfileKeyDetailsDTO;
use crate::terminal_text::{MAX_CELL_CHARACTERS, printable};
use crate::text_table::{Align, write_table};

/// The key's name and a table of its fields. Every text from the server goes
/// through `printable`: other people type it in.
pub fn write_details(out: &mut impl Write, details: &KeyDetailsDTO) -> io::Result<()> {
    writeln!(out, "Key  {}", key_name(details))?;
    write_fields(out, &details.fields)
}

/// One block per profile, as `write_details` writes it; profiles that failed
/// are left out, the caller reports them.
pub fn write_profile_details(
    out: &mut impl Write,
    profiles: &[ProfileKeyDetailsDTO],
) -> io::Result<()> {
    let mut first = true;
    for profile in profiles {
        let Some(details) = &profile.details else {
            continue;
        };
        if !first {
            writeln!(out)?;
        }
        first = false;
        writeln!(out, "Profile  {}", profile.profile)?;
        writeln!(out, "Key      {}", key_name(details))?;
        write_fields(out, &details.fields)?;
    }
    Ok(())
}

fn key_name(details: &KeyDetailsDTO) -> String {
    details
        .key_name
        .as_deref()
        .map(|name| printable(name, MAX_CELL_CHARACTERS))
        .unwrap_or_else(|| "(no name)".to_string())
}

fn write_fields(out: &mut impl Write, fields: &[KeyFieldDTO]) -> io::Result<()> {
    if fields.is_empty() {
        return writeln!(out, "No key fields: the organization has defined none.");
    }
    writeln!(out)?;
    let rows: Vec<[String; 2]> = fields
        .iter()
        .map(|field| {
            [
                printable(&field.display_name, MAX_CELL_CHARACTERS),
                printable(&value_text(field), MAX_CELL_CHARACTERS),
            ]
        })
        .collect();
    write_table(
        out,
        &[("FIELD", Align::Left), ("VALUE", Align::Left)],
        &rows,
    )
}

/// What a person reads for the value: a select's label, `yes`/`no`, the text,
/// or `-` when the key has none.
fn value_text(field: &KeyFieldDTO) -> String {
    match (&field.value, &field.value_label) {
        (None, _) => "-".to_string(),
        (Some(_), Some(label)) => label.clone(),
        (Some(KeyFieldValueDTO::Boolean(true)), None) => "yes".to_string(),
        (Some(KeyFieldValueDTO::Boolean(false)), None) => "no".to_string(),
        (Some(KeyFieldValueDTO::Text(text)), None) => text.clone(),
        (Some(KeyFieldValueDTO::Unknown(value)), None) => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::data_transfer_objects::profile_name_dto::ProfileNameDTO;

    fn details() -> KeyDetailsDTO {
        serde_json::from_str(
            r#"{
                "key_name": "laptop",
                "fields": [
                    {"key": "team", "display_name": "Team", "type": "select", "value": "backend", "value_label": "Backend"},
                    {"key": "project", "display_name": "Project", "type": "text", "value": null, "value_label": null},
                    {"key": "is_private", "display_name": "Is private", "type": "boolean", "value": true, "value_label": null},
                    {"key": "due", "display_name": "Due", "type": "date", "value": {"year": 2027}, "value_label": null}
                ]
            }"#,
        )
        .unwrap()
    }

    fn render(details: &KeyDetailsDTO) -> String {
        let mut output = Vec::new();
        write_details(&mut output, details).unwrap();
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn shows_labels_flags_and_missing_values() {
        assert_eq!(
            render(&details()),
            "Key  laptop\n\
             \n\
             FIELD       VALUE\n\
             Team        Backend\n\
             Project     -\n\
             Is private  yes\n\
             Due         {\"year\":2027}\n"
        );
    }

    #[test]
    fn says_so_when_the_organization_has_no_fields() {
        let mut details = details();
        details.key_name = None;
        details.fields.clear();
        assert_eq!(
            render(&details),
            "Key  (no name)\nNo key fields: the organization has defined none.\n"
        );
    }

    #[test]
    fn server_text_cannot_control_the_terminal() {
        let mut details = details();
        details.key_name = Some("laptop\u{1b}]0;owned\u{7}".to_string());
        details.fields[0].display_name = "Team\r\nFake line".to_string();
        details.fields[0].value_label = Some("\u{1b}[2J\u{202E}dneckab".to_string());
        let output = render(&details);
        assert!(!output.chars().any(|character| character == '\u{1b}'
            || character == '\r'
            || character == '\u{7}'
            || character == '\u{202E}'));
        assert!(output.starts_with("Key  laptop�]0;owned�\n"));
        assert!(output.contains("Team��Fake line  �[2J�dneckab\n"));
    }

    #[test]
    fn profiles_get_a_block_each_and_failures_are_left_out() {
        let profiles = [
            ProfileKeyDetailsDTO {
                profile: ProfileNameDTO::default_name(),
                details: Some(details()),
                error: None,
            },
            ProfileKeyDetailsDTO {
                profile: ProfileNameDTO::default_name(),
                details: None,
                error: Some("rejected".to_string()),
            },
        ];
        let mut output = Vec::new();
        write_profile_details(&mut output, &profiles).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.starts_with("Profile  default\nKey      laptop\n\nFIELD"));
        assert_eq!(output.matches("Profile").count(), 1);
    }
}
