/// 変換の意味とは別に、ソース上で選ばれた書式を保持します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionSyntax {
    /// `i64(value)`という簡潔な書式です。
    Compact,
    /// `convert<i64>(value)`という操作名を明示した書式です。
    Explicit,
}

/// ソースコード上のUTF-8バイト範囲を表します。
///
/// `start` は範囲に含み、`end` は含みません。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    start: usize,
    end: usize,
}

/// ソースコード上の1から始まる行番号と列番号を表します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    line: usize,
    column: usize,
}

impl SourceLocation {
    /// UTF-8バイト位置に対応する行番号と列番号を求めます。
    ///
    /// 列番号はUnicodeスカラー値単位で数えます。
    /// 位置がソースの範囲外またはUTF-8文字の途中の場合は`None`を返します。
    pub fn from_offset(source: &str, offset: usize) -> Option<Self> {
        let prefix = source.get(..offset)?;

        let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
        let column = prefix
            .rsplit('\n')
            .next()
            .expect("split always contains at least one element")
            .chars()
            .count()
            + 1;

        Some(Self { line, column })
    }

    /// 1から始まる行番号を返します。
    pub const fn line(self) -> usize {
        self.line
    }

    /// 1から始まる列番号を返します。
    pub const fn column(self) -> usize {
        self.column
    }
}

impl Span {
    /// 指定した開始位置と終了位置から範囲を作成します。
    pub const fn new(start: usize, end: usize) -> Self {
        assert!(start <= end, "span start must not exceed end");

        Self { start, end }
    }

    /// 指定した位置に空の範囲を作成します。
    pub const fn empty(offset: usize) -> Self {
        Self::new(offset, offset)
    }

    /// 範囲の開始バイト位置を返します。
    pub const fn start(self) -> usize {
        self.start
    }

    /// 範囲の終了バイト位置を返します。
    pub const fn end(self) -> usize {
        self.end
    }
}

#[cfg(test)]
mod tests {
    use super::SourceLocation;

    #[test]
    fn locates_start_of_source() {
        let location = SourceLocation::from_offset("", 0).unwrap();

        assert_eq!(location.line(), 1);
        assert_eq!(location.column(), 1);
    }

    #[test]
    fn locates_position_on_second_line() {
        let location = SourceLocation::from_offset("one\ntwo", 5).unwrap();

        assert_eq!(location.line(), 2);
        assert_eq!(location.column(), 2);
    }

    #[test]
    fn counts_unicode_scalar_values_in_column() {
        let location = SourceLocation::from_offset("a\n日本語", 8).unwrap();

        assert_eq!(location.line(), 2);
        assert_eq!(location.column(), 3);
    }

    #[test]
    fn locates_position_after_trailing_newline() {
        let location = SourceLocation::from_offset("x\n", 2).unwrap();

        assert_eq!(location.line(), 2);
        assert_eq!(location.column(), 1);
    }

    #[test]
    fn handles_crlf_line_endings() {
        let location = SourceLocation::from_offset("a\r\nb", 3).unwrap();

        assert_eq!(location.line(), 2);
        assert_eq!(location.column(), 1);
    }

    #[test]
    fn rejects_offset_inside_utf8_character() {
        assert_eq!(SourceLocation::from_offset("日", 1), None);
    }

    #[test]
    fn rejects_offset_past_end() {
        assert_eq!(SourceLocation::from_offset("x", 2), None);
    }
}
