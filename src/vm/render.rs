use crate::bytecode::Type;

use super::{VmError, VmErrorKind};

/// VM実行エラーをソース本文やファイルパスを含まない簡潔な形式で描画します。
pub fn render_compact(error: &VmError) -> String {
    let message = match error.kind() {
        VmErrorKind::InstructionOutOfBounds => {
            "Primer VM reached an instruction outside the bytecode program".to_owned()
        }
        VmErrorKind::InvalidSlot { slot } => {
            format!("Primer VM tried to access slot {slot}, but that slot does not exist")
        }
        VmErrorKind::UninitializedSlot { slot } => {
            format!("Primer VM tried to read slot {slot} before it was initialized")
        }
        VmErrorKind::SlotAlreadyInitialized { slot } => {
            format!("Primer VM tried to initialize slot {slot} more than once")
        }
        VmErrorKind::StackUnderflow => {
            "Primer VM needed another value, but the stack was empty".to_owned()
        }
        VmErrorKind::TypeMismatch { expected, actual } => format!(
            "Primer VM expected an {} value, but found an {} value",
            type_name(expected),
            type_name(actual)
        ),
        VmErrorKind::DivisionByZero => "cannot divide an integer by zero".to_owned(),
        VmErrorKind::DivisionOverflow => {
            "integer division produced a value outside the supported range".to_owned()
        }
        VmErrorKind::UnusedStackValues { count: 1 } => {
            "Primer VM stopped with 1 unused value on the stack".to_owned()
        }
        VmErrorKind::UnusedStackValues { count } => {
            format!("Primer VM stopped with {count} unused values on the stack")
        }
    };

    format!(
        "{message} at bytecode instruction {:04}",
        error.instruction_index()
    )
}

const fn type_name(ty: Type) -> &'static str {
    match ty {
        Type::I64 => "i64",
        Type::F32 => "f32",
        Type::F64 => "f64",
    }
}

#[cfg(test)]
mod tests {
    use super::render_compact;
    use crate::vm::{VmError, VmErrorKind};

    #[test]
    fn renders_division_by_zero_in_plain_language() {
        let error = VmError::new(VmErrorKind::DivisionByZero, 2);

        assert_eq!(
            render_compact(&error),
            "cannot divide an integer by zero at bytecode instruction 0002"
        );
    }

    #[test]
    fn renders_error_specific_values() {
        let error = VmError::new(VmErrorKind::InvalidSlot { slot: 7 }, 12);

        assert_eq!(
            render_compact(&error),
            "Primer VM tried to access slot 7, but that slot does not exist at bytecode instruction 0012"
        );
    }
}
