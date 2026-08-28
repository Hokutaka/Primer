/// ソースコード上のUTF-8バイト範囲を表します。
///
/// `start` は範囲に含み、`end` は含みません。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    start: usize,
    end: usize,
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
