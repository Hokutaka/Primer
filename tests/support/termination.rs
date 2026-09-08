use std::process::Output;

#[derive(Debug, Clone, Copy)]
pub enum Expected {
    Abort,
    CheckedCFailure,
    IllegalInstruction,
}

fn matches(output: &Output, expected: Expected) -> bool {
    // 非ゼロ終了だけでは、アクセス違反や起動失敗まで合格になってしまいます。
    #[cfg(unix)]
    let matches = {
        use std::os::unix::process::ExitStatusExt;
        output.status.signal()
            == Some(match expected {
                Expected::Abort | Expected::CheckedCFailure => 6,
                Expected::IllegalInstruction => 4,
            })
    };
    #[cfg(windows)]
    let matches = match expected {
        Expected::Abort | Expected::CheckedCFailure => matches!(
            output.status.code().map(|code| code as u32),
            Some(3 | 0xc0000409)
        ),
        Expected::IllegalInstruction => {
            output.status.code().map(|code| code as u32) == Some(0xc000001d)
        }
    };
    matches
        && (!matches!(expected, Expected::CheckedCFailure)
            || output.stderr.starts_with(b"primer: "))
}

pub fn assert_expected(output: &Output, expected: Expected, context: &str) {
    assert!(
        matches(output, expected),
        "{context}: expected {expected:?}, received {}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "{context}: unexpected output before failure: {:?}",
        output.stdout
    );
    if matches!(expected, Expected::CheckedCFailure) {
        use primer_lang::vm::{
            IntegerOperation as Op, NumericConversionFailure as Conversion, VmErrorKind as Error,
        };
        let primer_lang::RunError::Execution(error) = primer_lang::run_vm(context).unwrap_err()
        else {
            panic!("expected runtime failure")
        };
        let reason = match error.vm_error().kind() {
            Error::IntegerOverflow { operation, .. } => match operation {
                Op::Add => "u64 add overflow",
                Op::Subtract => "u64 subtract overflow",
                Op::Multiply => "u64 multiply overflow",
                Op::ShiftLeft => "u64 left shift overflow",
                Op::Negate => panic!("unexpected unsigned negation"),
            },
            Error::DivisionByZero => "integer division by zero",
            Error::RemainderByZero => "integer remainder by zero",
            Error::InvalidShiftCount { .. } => "u64 invalid shift count",
            Error::IntegerConversionOutOfRange { .. } => "integer conversion out of range",
            Error::NumericConversionFailed { reason, .. } => match reason {
                Conversion::OutOfRange => "numeric conversion out of range",
                Conversion::Inexact => "numeric conversion inexact",
                Conversion::NotFinite => "numeric conversion not finite",
                Conversion::NegativeZero => "numeric conversion negative zero",
                Conversion::NaN => panic!("unexpected width-changing NaN conversion"),
            },
            Error::ArrayIndexOutOfBounds { .. } => "array index out of bounds",
            other => panic!("unhandled expected C failure: {other:?}"),
        };
        assert_eq!(
            String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
            format!("primer: {reason}\n"),
            "{context}"
        );
    }
}

#[test]
fn unrelated_crashes_and_success_are_not_expected_failures() {
    #[cfg(unix)]
    fn status(code: i32) -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(code)
    }
    #[cfg(windows)]
    fn status(code: i32) -> std::process::ExitStatus {
        use std::os::windows::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(code as u32)
    }
    for expected in [
        Expected::Abort,
        Expected::CheckedCFailure,
        Expected::IllegalInstruction,
    ] {
        for code in [0, 11, 0xc0000005u32 as i32] {
            let output = Output {
                status: status(code),
                stdout: vec![],
                stderr: vec![],
            };
            assert!(!matches(&output, expected));
        }
    }
    let illegal = if cfg!(windows) {
        0xc000001du32 as i32
    } else {
        4
    };
    assert_expected(
        &Output {
            status: status(illegal),
            stdout: vec![],
            stderr: vec![],
        },
        Expected::IllegalInstruction,
        "expected trap control",
    );
}
