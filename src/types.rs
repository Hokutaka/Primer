/// Primerの整数型です。出力先のレジスタ幅や格納方法とは区別します。
///
/// 実装済みの種類だけを列挙し、種類を増やしたときに各出力先の対応漏れを検出します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntegerType {
    I64,
}

impl IntegerType {
    /// ソース、診断、Primer IRで共通の型名を返します。
    pub const fn name(self) -> &'static str {
        match self {
            Self::I64 => "i64",
        }
    }

    /// 負の整数を表せる型かどうかを返します。
    pub const fn is_signed(self) -> bool {
        match self {
            Self::I64 => true,
        }
    }

    /// 値の範囲を決めるビット数です。出力先での格納サイズではありません。
    pub const fn bit_width(self) -> u8 {
        match self {
            Self::I64 => 64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IntegerType;

    #[test]
    fn i64_has_a_signed_64_bit_range() {
        let ty = IntegerType::I64;

        assert_eq!(ty.name(), "i64");
        assert!(ty.is_signed());
        assert_eq!(ty.bit_width(), 64);
    }
}
