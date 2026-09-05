pub mod c;
pub mod llvm;
pub mod qbe;
mod support;
pub mod wat;
pub mod x86_64_win;

pub use c::emit_c;
pub use llvm::emit_llvm;
pub use qbe::emit_qbe;
pub use wat::emit_wat;
pub use x86_64_win::emit_x86_64_win_asm;

/// 値を変えない数値変換です。Emitterがソース構文を解釈し直す必要はありません。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NumericConversion {
    pub from: crate::types::NumericType,
    pub to: crate::types::NumericType,
}

impl NumericConversion {
    pub fn helper(self) -> String {
        format!("primer_convert_{}_{}", self.from.name(), self.to.name())
    }
}

/// 演算は64ビットで行い、値の範囲は元の整数型で検査します。
/// 格納幅と意味上の幅を区別し、狭い整数の桁あふれを隠しません。
fn integer_range_check(expr: &crate::ir::Expr) -> Option<crate::types::IntegerType> {
    use crate::{
        ir::{ExprKind, Type},
        types::IntegerType,
    };
    if !matches!(
        expr.kind,
        ExprKind::Binary { .. } | ExprKind::Unary { .. } | ExprKind::ConvertInteger { .. }
    ) {
        return None;
    }
    let Type::Integer(ty) = expr.ty else {
        return None;
    };
    match ty {
        IntegerType::I8
        | IntegerType::U8
        | IntegerType::I16
        | IntegerType::U16
        | IntegerType::I32
        | IntegerType::U32 => Some(ty),
        IntegerType::I64 => None,
    }
}

/// 整数専用の演算を、元の型の幅とともに各出力先へ渡します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntegerBinaryOp {
    Remainder,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

impl IntegerBinaryOp {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Remainder => "rem",
            Self::BitAnd => "bit_and",
            Self::BitOr => "bit_or",
            Self::BitXor => "bit_xor",
            Self::ShiftLeft => "shl",
            Self::ShiftRight => "shr",
        }
    }
    pub fn helper(self, ty: crate::types::IntegerType) -> String {
        format!("primer_{}_{}", ty.name(), self.name())
    }
}

fn integer_binary_op(op: crate::ir::BinaryOp) -> Option<IntegerBinaryOp> {
    use crate::ir::BinaryOp;
    match op {
        BinaryOp::Remainder => Some(IntegerBinaryOp::Remainder),
        BinaryOp::BitAnd => Some(IntegerBinaryOp::BitAnd),
        BinaryOp::BitOr => Some(IntegerBinaryOp::BitOr),
        BinaryOp::BitXor => Some(IntegerBinaryOp::BitXor),
        BinaryOp::ShiftLeft => Some(IntegerBinaryOp::ShiftLeft),
        BinaryOp::ShiftRight => Some(IntegerBinaryOp::ShiftRight),
        BinaryOp::Add
        | BinaryOp::Subtract
        | BinaryOp::Multiply
        | BinaryOp::Divide
        | BinaryOp::Equal
        | BinaryOp::NotEqual
        | BinaryOp::Less
        | BinaryOp::LessEqual
        | BinaryOp::Greater
        | BinaryOp::GreaterEqual => None,
    }
}

fn integer_type(ty: &crate::ir::Type) -> crate::types::IntegerType {
    let crate::ir::Type::Integer(ty) = ty else {
        unreachable!("integer operation must have integer operands")
    };
    *ty
}

fn complement_mask(ty: &crate::ir::Type) -> i64 {
    let ty = integer_type(ty);
    if ty.is_signed() { -1 } else { ty.maximum() }
}
