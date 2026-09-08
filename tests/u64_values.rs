use primer_lang::{RunError, compile_to_bytecode_text, compile_to_ir, compile_to_ir_text, run_vm};

#[test]
fn u64_literals_preserve_values_and_observable_types() {
    let source = "maximum: u64 = 18446744073709551615; print(maximum);";
    assert_eq!(run_vm(source).unwrap(), "18446744073709551615\n");
    for text in [
        compile_to_ir_text(source).unwrap(),
        compile_to_bytecode_text(source).unwrap(),
    ] {
        assert!(text.contains("18446744073709551615"));
        assert!(text.contains("u64"));
    }
    for source in [
        "print(18446744073709551616u64);",
        "print(18446744073709551615);",
        "print(-1u64);",
        "print(-0u64);",
        "print(1i64 + 1u64);",
        "print(1u64 < 1i64);",
        "values: [u64; 1] = [1]; print(values[0u64]);",
    ] {
        assert!(compile_to_ir(source).is_err(), "{source}");
    }
}

#[test]
fn failures_keep_the_source_origin_and_never_wrap() {
    #[path = "support/u64_cases.rs"]
    mod cases;
    for (source, expected) in [cases::EXAMPLE, cases::BOUNDARIES] {
        assert_eq!(run_vm(source).unwrap(), expected);
    }
    for source in cases::FAILURES {
        let Err(RunError::Execution(error)) = run_vm(source) else {
            panic!("{source}")
        };
        assert!(error.origin().is_some(), "{source}");
    }
}
