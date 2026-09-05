use primer_lang::{
    RunError,
    bytecode::{InstructionKind, InstructionOrigin},
    compile_to_bytecode, run_vm,
    vm::VmErrorKind,
};

#[test]
fn vm_calls_explicit_main_once_at_its_actual_function_id() {
    let source = "fn helper() -> i64 { return 42; } fn main() -> void { print(helper()); }";
    assert_eq!(run_vm(source).unwrap(), "42\n");
    let program = compile_to_bytecode(source).unwrap();
    assert_eq!(program.instructions.len(), 2);
    assert!(matches!(
        program.instructions[0].kind,
        InstructionKind::Call {
            function_id: 1,
            argument_count: 0
        }
    ));
    assert_eq!(program.instructions[0].origin, InstructionOrigin::Synthetic);
    assert!(matches!(
        program.instructions[1].kind,
        InstructionKind::Halt
    ));
}

#[test]
fn errors_inside_main_keep_their_function_and_source_origin() {
    let source = "fn main() -> void { print(1 / 0); }";
    let RunError::Execution(error) = run_vm(source).unwrap_err() else {
        panic!("expected VM error");
    };
    assert_eq!(error.vm_error().function_id(), Some(0));
    assert_eq!(error.vm_error().kind(), VmErrorKind::DivisionByZero);
    let Some(InstructionOrigin::Source { span, .. }) = error.origin() else {
        panic!("expected source origin");
    };
    assert_eq!(&source[span.start()..span.end()], "1 / 0");
}

#[test]
fn ordinary_functions_are_not_implicitly_run() {
    assert_eq!(run_vm("fn helper() -> void { print(42); }").unwrap(), "");
    assert_eq!(run_vm("print(42);").unwrap(), "42\n");
}
