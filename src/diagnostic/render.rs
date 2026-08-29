use super::Diagnostic;
use crate::source::SourceLocation;

const MAX_MESSAGE_CHARACTERS: usize = 4096;

/// 診断をソース本文やファイルパスを含まない簡潔な形式で描画します。
pub fn render_compact(diagnostic: &Diagnostic, source: &str) -> String {
    let message = sanitize_message(diagnostic.message());

    match diagnostic.primary_span() {
        Some(span) => match SourceLocation::from_offset(source, span.start()) {
            Some(location) => {
                format!("{message} at {}:{}", location.line(), location.column())
            }
            None => format!("{message} at byte {}", span.start()),
        },
        None => message,
    }
}

fn sanitize_message(message: &str) -> String {
    let mut characters = message.chars();
    let mut sanitized = String::new();

    for character in characters.by_ref().take(MAX_MESSAGE_CHARACTERS) {
        if character.is_control() {
            sanitized.extend(character.escape_default());
        } else {
            sanitized.push(character);
        }
    }

    if characters.next().is_some() {
        sanitized.push_str("...");
    }

    sanitized
}

#[cfg(test)]
mod tests {
    use super::{MAX_MESSAGE_CHARACTERS, render_compact};
    use crate::{diagnostic::Diagnostic, source::Span};

    #[test]
    fn renders_line_and_column() {
        let source = "x: i64 = 1;\nprint(missing);";
        let offset = source.find("missing").unwrap();
        let diagnostic =
            Diagnostic::new("unknown binding `missing`", Span::empty(offset));

        assert_eq!(
            render_compact(&diagnostic, source),
            "unknown binding `missing` at 2:7"
        );
    }

    #[test]
    fn renders_diagnostic_without_span() {
        let diagnostic = Diagnostic::without_span("compilation failed");

        assert_eq!(render_compact(&diagnostic, ""), "compilation failed");
    }

    #[test]
    fn falls_back_to_byte_offset_for_invalid_location() {
        let diagnostic = Diagnostic::new("invalid location", Span::empty(2));

        assert_eq!(
            render_compact(&diagnostic, "x"),
            "invalid location at byte 2"
        );
    }

    #[test]
    fn escapes_control_characters() {
        let diagnostic = Diagnostic::without_span("line\n\u{1b}[31m");

        assert_eq!(
            render_compact(&diagnostic, ""),
            r"line\n\u{1b}[31m"
        );
    }

    #[test]
    fn preserves_regular_backslashes_and_quotes() {
        let diagnostic =
            Diagnostic::without_span(r#"quoted "value" at \path"#);

        assert_eq!(
            render_compact(&diagnostic, ""),
            r#"quoted "value" at \path"#
        );
    }

    #[test]
    fn truncates_long_messages() {
        let message = "a".repeat(MAX_MESSAGE_CHARACTERS + 1);
        let diagnostic = Diagnostic::without_span(message);

        assert_eq!(
            render_compact(&diagnostic, ""),
            format!("{}...", "a".repeat(MAX_MESSAGE_CHARACTERS))
        );
    }
}
