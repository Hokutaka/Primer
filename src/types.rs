/// Primerの整数型です。出力先のレジスタ幅や格納方法とは区別します。
///
/// 実装済みの種類だけを列挙し、種類を増やしたときに各出力先の対応漏れを検出します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntegerType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
}

impl IntegerType {
    /// 実装済みの整数型を決定的な順序で列挙します。
    pub const ALL: [Self; 7] = [
        Self::I8,
        Self::U8,
        Self::I16,
        Self::U16,
        Self::I32,
        Self::U32,
        Self::I64,
    ];

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|ty| ty.name() == name)
    }

    /// ソース、診断、Primer IRで共通の型名を返します。
    pub const fn name(self) -> &'static str {
        match self {
            Self::I8 => "i8",
            Self::U8 => "u8",
            Self::I16 => "i16",
            Self::U16 => "u16",
            Self::I32 => "i32",
            Self::U32 => "u32",
            Self::I64 => "i64",
        }
    }

    /// 負の整数を表せる型かどうかを返します。
    pub const fn is_signed(self) -> bool {
        match self {
            Self::I8 | Self::I16 | Self::I32 | Self::I64 => true,
            Self::U8 | Self::U16 | Self::U32 => false,
        }
    }

    /// 値の範囲を決めるビット数です。出力先での格納サイズではありません。
    pub const fn bit_width(self) -> u8 {
        match self {
            Self::I8 | Self::U8 => 8,
            Self::I16 | Self::U16 => 16,
            Self::I32 | Self::U32 => 32,
            Self::I64 => 64,
        }
    }

    /// この整数型で表せる最小値です。
    pub const fn minimum(self) -> i64 {
        match self {
            Self::I8 => i8::MIN as i64,
            Self::I16 => i16::MIN as i64,
            Self::I32 => i32::MIN as i64,
            Self::U8 | Self::U16 | Self::U32 => 0,
            Self::I64 => i64::MIN,
        }
    }

    /// この整数型で表せる最大値です。
    pub const fn maximum(self) -> i64 {
        match self {
            Self::I8 => i8::MAX as i64,
            Self::U8 => u8::MAX as i64,
            Self::I16 => i16::MAX as i64,
            Self::U16 => u16::MAX as i64,
            Self::I32 => i32::MAX as i64,
            Self::U32 => u32::MAX as i64,
            Self::I64 => i64::MAX,
        }
    }

    /// 格納用の値が、意味上の整数型の範囲に収まるか調べます。
    pub const fn contains(self, value: i64) -> bool {
        value >= self.minimum() && value <= self.maximum()
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
