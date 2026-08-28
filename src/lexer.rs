use crate::{diagnostic::Diagnostic, source::Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Print,

    Identifier(String),

    Integer(i64),
    Float(String),

    Equal,
    Colon,

    Plus,
    Minus,
    Star,
    Slash,

    LeftParen,
    RightParen,
    Semicolon,

    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub fn lex(source: &str) -> Result<Vec<Token>, Diagnostic> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];

        // Whitespace
        if b.is_ascii_whitespace() {
            i += 1;
            continue;
        }

        // Line comment
        if b == b'/' && bytes.get(i + 1) == Some(&b'/') {
            i += 2;

            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }

            continue;
        }

        let offset = i;

        let kind = match b {
            b':' => {
                i += 1;
                TokenKind::Colon
            }

            b'=' => {
                i += 1;
                TokenKind::Equal
            }

            b'+' => {
                i += 1;
                TokenKind::Plus
            }

            b'-' => {
                i += 1;
                TokenKind::Minus
            }

            b'*' => {
                i += 1;
                TokenKind::Star
            }

            b'/' => {
                i += 1;
                TokenKind::Slash
            }

            b'(' => {
                i += 1;
                TokenKind::LeftParen
            }

            b')' => {
                i += 1;
                TokenKind::RightParen
            }

            b';' => {
                i += 1;
                TokenKind::Semicolon
            }

            // Numeric literal
            b'0'..=b'9' => {
                let start = i;
                let mut is_float = false;

                // Integer part
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }

                // Fractional part
                if bytes.get(i) == Some(&b'.') {
                    is_float = true;
                    i += 1;

                    if i >= bytes.len() || !bytes[i].is_ascii_digit() {
                        return Err(Diagnostic::new(
                            "expected digit after decimal point",
                            Span::empty(i),
                        ));
                    }

                    while i < bytes.len() && bytes[i].is_ascii_digit() {
                        i += 1;
                    }
                }

                // Scientific notation
                if matches!(bytes.get(i), Some(b'e') | Some(b'E')) {
                    is_float = true;
                    i += 1;

                    if matches!(bytes.get(i), Some(b'+') | Some(b'-')) {
                        i += 1;
                    }

                    if i >= bytes.len() || !bytes[i].is_ascii_digit() {
                        return Err(Diagnostic::new("expected exponent digits", Span::empty(i)));
                    }

                    while i < bytes.len() && bytes[i].is_ascii_digit() {
                        i += 1;
                    }
                }

                // Explicit floating-point suffix
                if bytes[i..].starts_with(b"f32") || bytes[i..].starts_with(b"f64") {
                    is_float = true;
                    i += 3;
                }

                // Prevent things such as:
                //
                //   1foo
                //   0.1f32abc
                //
                // from silently becoming multiple tokens.
                if i < bytes.len() && (bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') {
                    return Err(Diagnostic::new(
                        "invalid numeric literal",
                        Span::new(start, i),
                    ));
                }

                let text = &source[start..i];

                if is_float {
                    TokenKind::Float(text.to_owned())
                } else {
                    let value = text.parse::<i64>().map_err(|_| {
                        Diagnostic::new("integer literal out of range", Span::new(start, i))
                    })?;
                    TokenKind::Integer(value)
                }
            }

            // Identifier / keyword
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let start = i;
                i += 1;

                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }

                match &source[start..i] {
                    "print" => TokenKind::Print,
                    name => TokenKind::Identifier(name.to_owned()),
                }
            }

            _ => {
                let character = source[i..].chars().next().unwrap();

                return Err(Diagnostic::new(
                    format!("unexpected character {character:?}"),
                    Span::new(i, i + character.len_utf8()),
                ));
            }
        };

        tokens.push(Token {
            kind,
            span: Span::new(offset, i),
        });
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span::empty(source.len()),
    });

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::{TokenKind, lex};
    use crate::source::Span;

    #[test]
    fn lexes_minimal_program() {
        let tokens = lex("x: i64 = 1 + 2; print(x);").unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Identifier("x".into()));
        assert_eq!(tokens[1].kind, TokenKind::Colon);
        assert_eq!(tokens[2].kind, TokenKind::Identifier("i64".into()));
        assert_eq!(tokens[3].kind, TokenKind::Equal);
        assert_eq!(tokens[4].kind, TokenKind::Integer(1));
        assert_eq!(tokens[5].kind, TokenKind::Plus);
        assert_eq!(tokens[6].kind, TokenKind::Integer(2));
        assert_eq!(tokens[8].kind, TokenKind::Print);
    }

    #[test]
    fn lexes_float_literals() {
        let tokens = lex("a: f32 = 0.1f32; \
             b: f64 = 0.2f64; \
             c: infer = 0.3;")
        .unwrap();

        assert!(
            tokens
                .iter()
                .any(|token| token.kind == TokenKind::Float("0.1f32".into()))
        );

        assert!(
            tokens
                .iter()
                .any(|token| token.kind == TokenKind::Float("0.2f64".into()))
        );

        assert!(
            tokens
                .iter()
                .any(|token| token.kind == TokenKind::Float("0.3".into()))
        );
    }

    #[test]
    fn lexes_integer_float_suffixes() {
        let tokens = lex("a: f32 = 1f32; b: f64 = 2f64;").unwrap();

        assert!(
            tokens
                .iter()
                .any(|token| token.kind == TokenKind::Float("1f32".into()))
        );

        assert!(
            tokens
                .iter()
                .any(|token| token.kind == TokenKind::Float("2f64".into()))
        );
    }

    #[test]
    fn lexes_scientific_notation() {
        let tokens = lex("x: f64 = 1.5e-3;").unwrap();

        assert!(
            tokens
                .iter()
                .any(|token| token.kind == TokenKind::Float("1.5e-3".into()))
        );
    }

    #[test]
    fn skips_line_comments() {
        let tokens = lex("x: i64 = 1; // hello\nprint(x);").unwrap();

        assert!(tokens.iter().any(|token| token.kind == TokenKind::Print));
    }

    #[test]
    fn rejects_invalid_numeric_suffix() {
        let error = lex("0.1foo").unwrap_err();

        assert_eq!(error.message(), "invalid numeric literal");
        assert_eq!(error.primary_span(), Some(Span::new(0, 3)));
    }

    #[test]
    fn records_token_spans() {
        let tokens = lex("x: i64").unwrap();

        assert_eq!(tokens[0].span, Span::new(0, 1));
        assert_eq!(tokens[1].span, Span::new(1, 2));
        assert_eq!(tokens[2].span, Span::new(3, 6));
        assert_eq!(tokens[3].span, Span::empty(6));
    }

    #[test]
    fn reports_utf8_character_span() {
        let error = lex("あ").unwrap_err();

        assert_eq!(error.primary_span(), Some(Span::new(0, 3)));
    }

    #[test]
    fn reports_missing_exponent_digits() {
        let error = lex("1e").unwrap_err();

        assert_eq!(error.message(), "expected exponent digits");
        assert_eq!(error.primary_span(), Some(Span::empty(2)));
    }
}
