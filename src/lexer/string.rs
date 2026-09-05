use std::str::CharIndices;

use crate::{diagnostic::Diagnostic, source::Span};

pub(super) fn lex_string(source: &str, start: usize) -> Result<(String, usize), Diagnostic> {
    let base = start + 1;
    let mut characters = source[base..].char_indices();
    let mut value = String::new();

    // 値を復元しても、診断位置は元のUTF-8ソース上のバイト位置で計算します。
    while let Some((relative, character)) = characters.next() {
        let offset = base + relative;
        match character {
            '"' => return Ok((value, offset + 1)),
            '\n' | '\r' => {
                return Err(Diagnostic::new(
                    r"string literal cannot contain a line break; use `\n` or `\r`",
                    Span::new(offset, offset + 1),
                ));
            }
            '\\' => {
                let Some((relative, escaped)) = characters.next() else {
                    return Err(Diagnostic::new(
                        "expected an escape character after `\\`",
                        Span::new(offset, offset + 1),
                    ));
                };
                let decoded = match escaped {
                    '"' => '"',
                    '\\' => '\\',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '0' => '\0',
                    'u' => unicode_escape(&mut characters, base, offset, source.len())?,
                    _ => {
                        return Err(Diagnostic::new(
                            "unknown string escape",
                            Span::new(offset, base + relative + escaped.len_utf8()),
                        ));
                    }
                };
                value.push(decoded);
            }
            _ => value.push(character),
        }
    }

    Err(Diagnostic::new(
        "expected closing `\"` for string literal",
        Span::new(start, source.len()),
    ))
}

fn unicode_escape(
    characters: &mut CharIndices<'_>,
    base: usize,
    start: usize,
    source_end: usize,
) -> Result<char, Diagnostic> {
    let error = |message: &str, end| Diagnostic::new(message, Span::new(start, end));
    match characters.next() {
        Some((_, '{')) => {}
        Some((relative, character)) => {
            return Err(error(
                "expected `{` after `\\u`",
                base + relative + character.len_utf8(),
            ));
        }
        None => return Err(error("expected `{` after `\\u`", source_end)),
    }

    let mut digits = String::new();
    for (relative, character) in characters.by_ref() {
        let end = base + relative + character.len_utf8();
        if character == '}' {
            let code = u32::from_str_radix(&digits, 16)
                .map_err(|_| error("expected 1 to 6 hexadecimal digits in Unicode escape", end))?;
            return char::from_u32(code)
                .ok_or_else(|| error("Unicode escape does not name a valid character", end));
        }
        if !character.is_ascii_hexdigit() || digits.len() == 6 {
            return Err(error(
                "expected 1 to 6 hexadecimal digits followed by `}` in Unicode escape",
                end,
            ));
        }
        digits.push(character);
    }

    Err(error("expected `}` to close Unicode escape", source_end))
}
