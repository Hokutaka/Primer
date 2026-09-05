use primer_lang::{
    diagnostic::render::render_compact,
    lexer::{TokenKind, lex},
    source::Span,
};

fn assert_literal(source: &str, expected: &str) {
    let tokens = lex(source).unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::String(expected.to_owned()));
    assert_eq!(tokens[0].span, Span::new(0, source.len()));
    assert_eq!(tokens[1].kind, TokenKind::Eof);
    assert_eq!(tokens[1].span, Span::empty(source.len()));
}

#[test]
fn reads_empty_ascii_and_unicode_strings() {
    assert_literal(r#""""#, "");
    assert_literal(r#""hello, Primer""#, "hello, Primer");
    assert_literal("\"日本語と文字列\"", "日本語と文字列");
    assert_literal("\"\u{1f600}\"", "\u{1f600}");
}

#[test]
fn decodes_simple_escapes_without_changing_the_source_span() {
    assert_literal(r#""\"\\\n\r\t\0""#, "\"\\\n\r\t\0");
    assert_literal(r#""\\n""#, r"\n");
    assert_literal(r#""\0after""#, "\0after");
}

#[test]
fn decodes_unicode_scalars_including_boundary_values() {
    for (digits, value) in [
        ("0", '\0'),
        ("000041", 'A'),
        ("65e5", '日'),
        ("D7FF", '\u{d7ff}'),
        ("e000", '\u{e000}'),
        ("1f600", '\u{1f600}'),
        ("10FFFF", '\u{10ffff}'),
    ] {
        let source = format!("\"\\u{{{digits}}}\"");
        assert_literal(&source, &value.to_string());
    }
}

#[test]
fn treats_comments_and_operators_inside_quotes_as_text() {
    assert_literal(r#""// + ; \"quoted\"""#, "// + ; \"quoted\"");
    let tokens = lex("// \"ignored\"\n\"first\" // ignored\n\"second\"").unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].kind, TokenKind::String("first".into()));
    assert_eq!(tokens[1].kind, TokenKind::String("second".into()));
}

#[test]
fn resumes_lexing_at_the_closing_quote() {
    let source = "print(\"日本語\\n\");";
    let tokens = lex(source).unwrap();
    let start = source.find('"').unwrap();
    let end = source.rfind('"').unwrap() + 1;
    assert_eq!(tokens[2].kind, TokenKind::String("日本語\n".into()));
    assert_eq!(tokens[2].span, Span::new(start, end));
    assert_eq!(tokens[3].kind, TokenKind::RightParen);
    assert_eq!(tokens[3].span, Span::new(end, end + 1));
    assert_eq!(tokens[4].kind, TokenKind::Semicolon);
}

#[test]
fn does_not_concatenate_adjacent_literals_or_normalize_unicode() {
    let tokens = lex("\"\u{e9}\"\"e\u{301}\"").unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].kind, TokenKind::String("\u{e9}".into()));
    assert_eq!(tokens[1].kind, TokenKind::String("e\u{301}".into()));
    assert_ne!(tokens[0].kind, tokens[1].kind);
}

#[test]
fn reports_unclosed_strings_from_the_opening_quote() {
    for literal in ["\"", "\"日本語", "\"escaped\\\""] {
        let source = format!("print({literal}");
        let error = lex(&source).unwrap_err();
        assert_eq!(error.message(), "expected closing `\"` for string literal");
        assert_eq!(error.primary_span(), Some(Span::new(6, source.len())));
    }
}

#[test]
fn rejects_physical_line_breaks_including_crlf() {
    for line_break in ["\n", "\r", "\r\n"] {
        let prefix = "\"日本語";
        let source = format!("{prefix}{line_break}text\"");
        let error = lex(&source).unwrap_err();
        assert!(error.message().contains("cannot contain a line break"));
        assert_eq!(
            error.primary_span(),
            Some(Span::new(prefix.len(), prefix.len() + 1))
        );
    }
}

#[test]
fn reports_unknown_escapes_at_the_original_utf8_range() {
    for escaped in ["q", "日", "x", "\n"] {
        let source = format!("\"日本語\\{escaped}\"");
        let start = source.find('\\').unwrap();
        let error = lex(&source).unwrap_err();
        assert_eq!(error.message(), "unknown string escape");
        assert_eq!(
            error.primary_span(),
            Some(Span::new(start, start + 1 + escaped.len()))
        );
    }
}

#[test]
fn reports_a_trailing_backslash_without_panicking() {
    let source = "\"日本語\\";
    let error = lex(source).unwrap_err();
    assert_eq!(error.message(), "expected an escape character after `\\`");
    assert_eq!(
        error.primary_span(),
        Some(Span::new(source.len() - 1, source.len()))
    );
}

#[test]
fn rejects_invalid_unicode_scalar_values() {
    for digits in ["D800", "DFFF", "110000", "FFFFFF"] {
        let source = format!("\"\\u{{{digits}}}\"");
        let error = lex(&source).unwrap_err();
        assert_eq!(
            error.message(),
            "Unicode escape does not name a valid character"
        );
        assert_eq!(error.primary_span(), Some(Span::new(1, source.len() - 1)));
    }
}

#[test]
fn rejects_malformed_unicode_escapes() {
    for escape in [
        r"\u41",
        r"\u{}",
        r"\u{G}",
        r"\u{0000041}",
        r"\u{日}",
        r"\u{41",
    ] {
        let source = format!("\"{escape}\"");
        let error = lex(&source).unwrap_err();
        let span = error.primary_span().unwrap();
        assert_eq!(span.start(), 1);
        assert!(span.end() <= source.len());
        assert!(source.is_char_boundary(span.end()));
    }
}

#[test]
fn all_truncated_utf8_prefixes_produce_valid_tokens_or_diagnostics() {
    let source = "\"日本語\\u{1F600}\\n\\\"end\"";
    for end in (0..=source.len()).filter(|end| source.is_char_boundary(*end)) {
        let prefix = &source[..end];
        match lex(prefix) {
            Ok(tokens) => assert_eq!(tokens.last().unwrap().span, Span::empty(end)),
            Err(error) => {
                let span = error.primary_span().unwrap();
                assert!(span.start() <= span.end());
                assert!(span.end() <= end);
                assert!(prefix.is_char_boundary(span.start()));
                assert!(prefix.is_char_boundary(span.end()));
            }
        }
    }
}

#[test]
fn parser_reports_that_string_values_are_not_available_yet() {
    let source = "print(\"日本語\\n\");";
    let error = primer_lang::compile(source).unwrap_err();
    assert_eq!(error.message(), "string values are not supported yet");
    assert_eq!(error.primary_span(), Some(Span::new(6, source.len() - 2)));
    assert_eq!(
        render_compact(&error, source),
        "string values are not supported yet at 1:7"
    );
}

#[test]
fn unicode_escape_errors_keep_byte_positions_after_multibyte_text() {
    let source = "print(\"日本語\\u{D800}\");";
    let error = lex(source).unwrap_err();
    assert_eq!(
        error.primary_span(),
        Some(Span::new(
            source.find('\\').unwrap(),
            source.find('}').unwrap() + 1
        ))
    );
}

#[test]
fn reports_incomplete_unicode_escape_delimiters() {
    for (source, expected) in [
        (r#""\u"#, "expected `{` after `\\u`"),
        (r#""\u{"#, "expected `}` to close Unicode escape"),
        (r#""\u{41"#, "expected `}` to close Unicode escape"),
    ] {
        let error = lex(source).unwrap_err();
        assert_eq!(error.message(), expected);
        assert_eq!(error.primary_span(), Some(Span::new(1, source.len())));
    }
}
