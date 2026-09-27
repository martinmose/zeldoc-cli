use std::io::{self, Write};

use super::data_transfer_objects::search_result_dto::SearchResultDTO;

/// Numbered results with the URL and snippet indented under each title,
/// separated by blank lines.
pub fn write_results(out: &mut impl Write, results: &[SearchResultDTO]) -> io::Result<()> {
    for (index, result) in results.iter().enumerate() {
        if index > 0 {
            writeln!(out)?;
        }
        write!(out, "{}. {}", index + 1, single_line(&result.title))?;
        if let Some(date) = result.date.as_deref().filter(|date| !date.is_empty()) {
            write!(out, " ({date})")?;
        }
        writeln!(out)?;
        writeln!(out, "   {}", result.url)?;
        if let Some(snippet) = result.snippet.as_deref().map(single_line)
            && !snippet.is_empty()
        {
            writeln!(out, "   {snippet}")?;
        }
    }
    Ok(())
}

fn single_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::data_transfer_objects::search_response_dto::SearchResponseDTO;

    #[test]
    fn results_are_numbered_with_url_and_single_line_snippet() {
        let response: SearchResponseDTO = serde_json::from_str(
            r#"{"results":[
                {"title":"Rootless podman\nports","url":"https://example.com/a","snippet":"Use  a\n port above 1024.","date":null,"last_updated":null},
                {"title":"Second","url":"https://example.com/b","snippet":"","date":"2026-05-01"}
            ]}"#,
        )
        .unwrap();
        let mut out = Vec::new();
        write_results(&mut out, &response.results).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "\
1. Rootless podman ports
   https://example.com/a
   Use a port above 1024.

2. Second (2026-05-01)
   https://example.com/b
"
        );
    }
}
