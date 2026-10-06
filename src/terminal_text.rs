//! Text from Zeldoc.ai made safe to print on a terminal.
//!
//! Values such as key field names and values are typed in by other people. A
//! control character in one (an ANSI escape sequence, a carriage return, a
//! bidirectional override) could recolour the terminal, overwrite earlier
//! lines or reorder what is shown, so printing must never pass one through.

/// Longest text a table cell shows; the rest is cut off with `…`. `--json`
/// keeps the whole text.
pub const MAX_CELL_CHARACTERS: usize = 80;

/// `text` on one line with every control and bidirectional formatting
/// character replaced by `�`, cut off after `max_characters`.
pub fn printable(text: &str, max_characters: usize) -> String {
    let mut printable: String = text
        .chars()
        .map(|character| {
            if is_unsafe(character) {
                char::REPLACEMENT_CHARACTER
            } else {
                character
            }
        })
        .take(max_characters)
        .collect();
    if text.chars().count() > max_characters {
        printable.push('…');
    }
    printable
}

/// Control characters (C0, DEL, C1: escape sequences, line breaks, tabs) and
/// the Unicode characters that change the direction text is shown in.
fn is_unsafe(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{061C}' | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_is_unchanged() {
        assert_eq!(
            printable("Backend – Team ÆØÅ 日本", 80),
            "Backend – Team ÆØÅ 日本"
        );
    }

    #[test]
    fn escape_sequences_and_line_breaks_are_replaced() {
        assert_eq!(
            printable("\u{1b}[31mred\u{1b}[0m\r\nnext\tcell\u{7f}\u{9b}", 80),
            "�[31mred�[0m��next�cell��"
        );
    }

    #[test]
    fn bidirectional_overrides_are_replaced() {
        assert_eq!(
            printable("abc\u{202E}fed\u{2066}x\u{200F}", 80),
            "abc�fed�x�"
        );
    }

    #[test]
    fn long_text_is_cut_off() {
        assert_eq!(printable("abcdef", 3), "abc…");
        assert_eq!(printable("abc", 3), "abc");
    }
}
