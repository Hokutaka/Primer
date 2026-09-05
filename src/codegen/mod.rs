pub mod c;
pub mod llvm;
pub mod qbe;
pub mod wat;
pub mod x86_64_win;

pub use c::emit_c;
pub use llvm::emit_llvm;
pub use qbe::emit_qbe;
pub use wat::emit_wat;
pub use x86_64_win::emit_x86_64_win_asm;

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
