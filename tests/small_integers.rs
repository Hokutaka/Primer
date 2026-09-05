use primer_lang::{
    RunError, bytecode, compile, compile_to_bytecode_text, compile_to_ir, ir, run_vm,
    types::IntegerType,
    vm::{IntegerOperation, VmErrorKind},
};

const SMALL_TYPES: [IntegerType; 4] = [
    IntegerType::I8,
    IntegerType::U8,
    IntegerType::I16,
    IntegerType::U16,
];

#[test]
fn names_widths_and_ranges_match_the_declared_types() {
    for (ty, name, width, signed, min, max) in [
        (IntegerType::I8, "i8", 8, true, -128, 127),
        (IntegerType::U8, "u8", 8, false, 0, 255),
        (IntegerType::I16, "i16", 16, true, -32768, 32767),
        (IntegerType::U16, "u16", 16, false, 0, 65535),
    ] {
        assert_eq!(
            (
                ty.name(),
                ty.bit_width(),
                ty.is_signed(),
                ty.minimum(),
                ty.maximum()
            ),
            (name, width, signed, min, max)
        );
        assert_eq!(IntegerType::from_name(name), Some(ty));
        for value in [min, 0, max] {
            for annotation in [name, "infer"] {
                let source = format!("value: {annotation} = {value}{name}; print(value);");
                assert_eq!(run_vm(&source).unwrap(), format!("{value}\n"));
                let program = compile_to_ir(&source).unwrap();
                let ir::StatementKind::Binding { ty: actual, .. } = &program.statements[0].kind
                else {
                    panic!()
                };
                assert_eq!(*actual, ir::Type::Integer(ty));
                assert!(
                    compile_to_bytecode_text(&source)
                        .unwrap()
                        .contains(&format!("push.{name}"))
                );
            }
        }
        for value in [min - 1, max + 1] {
            assert!(compile(&format!("value: {name} = {value};")).is_err());
            assert!(compile(&format!("print({value}{name});")).is_err());
        }
        assert!(compile(&format!("fn {name}() -> void {{ }}")).is_err());
        assert!(compile(&format!("type {name} {{ value: bool, }}")).is_err());
    }
    let mut names = IntegerType::ALL.map(IntegerType::name).to_vec();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), IntegerType::ALL.len());
}

#[test]
fn small_integer_arithmetic_matches_mathematical_results_or_reports_failure() {
    for ty in SMALL_TYPES {
        let candidates = [
            ty.minimum(),
            ty.minimum() + 1,
            -1,
            0,
            1,
            ty.maximum() - 1,
            ty.maximum(),
        ];
        let values = candidates
            .into_iter()
            .filter(|value| ty.contains(*value))
            .collect::<Vec<_>>();
        for &left in &values {
            for &right in &values {
                for op in ["+", "-", "*", "/"] {
                    let source = format!(
                        "left: {0} = {left}; right: {0} = {right}; print(left {op} right);",
                        ty.name()
                    );
                    let result = match op {
                        "+" => Some(left + right),
                        "-" => Some(left - right),
                        "*" => Some(left * right),
                        "/" if right != 0 => Some(left / right),
                        _ => None,
                    };
                    if let Some(value) = result.filter(|value| ty.contains(*value)) {
                        assert_eq!(run_vm(&source).unwrap(), format!("{value}\n"), "{source}");
                    } else {
                        let Err(RunError::Execution(error)) = run_vm(&source) else {
                            panic!("{source}")
                        };
                        let expected = match op {
                            "/" if right == 0 => VmErrorKind::DivisionByZero,
                            "/" => VmErrorKind::DivisionOverflow,
                            _ => VmErrorKind::IntegerOverflow {
                                operation: match op {
                                    "+" => IntegerOperation::Add,
                                    "-" => IntegerOperation::Subtract,
                                    "*" => IntegerOperation::Multiply,
                                    _ => unreachable!(),
                                },
                                ty: bytecode::Type::Integer(ty),
                            },
                        };
                        assert_eq!(error.vm_error().kind(), expected, "{source}");
                        assert!(error.origin().is_some());
                    }
                }
            }
        }
        if ty.is_signed() {
            let source = format!("value: {} = {}; print(-value);", ty.name(), ty.minimum());
            let Err(RunError::Execution(error)) = run_vm(&source) else {
                panic!()
            };
            assert_eq!(
                error.vm_error().kind(),
                VmErrorKind::IntegerOverflow {
                    operation: IntegerOperation::Negate,
                    ty: bytecode::Type::Integer(ty)
                }
            );
        } else {
            assert!(compile(&format!("print(-0{});", ty.name())).is_err());
        }
    }
}

#[test]
fn every_eight_bit_value_round_trips_without_changing_its_number() {
    for ty in [IntegerType::I8, IntegerType::U8] {
        for value in ty.minimum()..=ty.maximum() {
            let source = format!(
                "value: {0} = {value}; print(convert<{0}>(i16(value))); print({0}(convert<i64>(value)));",
                ty.name()
            );
            assert_eq!(run_vm(&source).unwrap(), format!("{value}\n{value}\n"));
        }
    }
}

#[test]
fn context_and_nested_value_types_keep_small_integer_kinds() {
    for ty in SMALL_TYPES {
        let name = ty.name();
        let source = format!(
            "
            type Pair {{ values: [{name}; 2], extra: {name} = 2, }}
            fn increment(value: {name}) -> {name} {{ return 1 + value; }}
            pair: Pair = Pair {{ values: [1, 3], }};
            mut values: [{name}; 2] = pair.values;
            values[1] = increment((1 + 2) + pair.extra);
            print(values[1]); print(pair.values[1]); print(2 < values[1]);
        "
        );
        assert_eq!(run_vm(&source).unwrap(), "6\n3\ntrue\n");
        assert!(compile(&format!("value: {name} = 1; wider: i64 = value;")).is_err());
        assert!(compile(&format!("value: i64 = 1; smaller: {name} = value;")).is_err());
        assert!(compile(&format!("print(1.0{name});")).is_err());
        assert!(compile(&format!("print(1{name}abc);")).is_err());
    }
}
