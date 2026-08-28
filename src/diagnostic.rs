use crate::source::Span;

/// コンパイル中に見つかった問題を表します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    message: String,
    primary_span: Option<Span>,
}

impl Diagnostic {
    /// 主な問題箇所を持つ診断を作成します。
    pub fn new(message: impl Into<String>, primary_span: Span) -> Self {
        Self {
            message: message.into(),
            primary_span: Some(primary_span),
        }
    }

    /// ソースコード上の位置を持たない診断を作成します。
    pub fn without_span(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            primary_span: None,
        }
    }

    /// 診断メッセージを返します。
    pub fn message(&self) -> &str {
        &self.message
    }

    /// 主な問題箇所を返します。
    pub const fn primary_span(&self) -> Option<Span> {
        self.primary_span
    }
}
