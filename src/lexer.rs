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
    pub offset: usize,
}

pub fn lex(source: &str) -> Result<Vec<Token>, String> {
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
                        return Err(format!("expected digit after decimal point at byte {i}"));
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
                        return Err(format!("expected exponent digits at byte {i}"));
                    }

                    while i < bytes.len() && bytes[i].is_ascii_digit() {
                        i += 1;
                    }
                }

                // Explicit floating-point suffix
                if bytes[i..].starts_with(b"f32") {
                    is_float = true;
                    i += 3;
                } else if bytes[i..].starts_with(b"f64") {
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
                    return Err(format!("invalid numeric literal at byte {start}"));
                }

                let text = &source[start..i];

                if is_float {
                    TokenKind::Float(text.to_owned())
                } else {
                    let value = text
                        .parse::<i64>()
                        .map_err(|_| format!("integer literal out of range at byte {start}"))?;

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
                return Err(format!(
                    "unexpected character {:?} at byte {i}",
                    source[i..].chars().next().unwrap()
                ));
            }
        };

        tokens.push(Token { kind, offset });
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        offset: source.len(),
    });

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::{TokenKind, lex};

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
        let error = lex("x: f64 = 0.1foo;").unwrap_err();

        assert!(error.contains("invalid numeric literal"));
    }
}
