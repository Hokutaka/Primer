use primer_lang::{
    compile, compile_to_ir,
    ir::{self, ExprKind, StatementKind},
    semantic,
    types::IntegerType,
};

#[test]
fn explicit_and_inferred_integers_keep_their_kind_and_boundary_values() {
    for annotation in ["i64", "infer"] {
        for value in [i64::MIN, 0, i64::MAX] {
            let source = format!("value: {annotation} = {value};");
            let ast = compile(&source).unwrap();
            let model = semantic::analyze(&ast).unwrap();
            let semantic_ty = semantic::Type::Integer(IntegerType::I64);

            assert_eq!(model.bindings["value"].ty, semantic_ty);
            assert_eq!(model.type_name(semantic_ty), "i64");

            let program = compile_to_ir(&source).unwrap();
            let StatementKind::Binding {
                ty, value: expr, ..
            } = &program.statements[0].kind
            else {
                panic!("expected binding");
            };
            assert_eq!(*ty, ir::Type::Integer(IntegerType::I64));
            assert_eq!(expr.ty, *ty);
            assert_eq!(expr.kind, ExprKind::Integer(value));
        }
    }
}

#[test]
fn integer_kinds_survive_nested_arrays_fields_and_function_signatures() {
    let source = "
        type Grid { cells: [[i64; 2]; 2], }
        fn first(cells: [[i64; 2]; 2]) -> i64 {
            return cells[0][0];
        }
        grid: Grid = Grid { cells: [[1, 2], [3, 4]], };
        value: infer = first(grid.cells);
    ";
    let ast = compile(source).unwrap();
    let model = semantic::analyze(&ast).unwrap();
    let semantic_integer = semantic::Type::Integer(IntegerType::I64);
    let semantic_array = semantic::Type::Array {
        element: Box::new(semantic::Type::Array {
            element: Box::new(semantic_integer.clone()),
            length: 2,
        }),
        length: 2,
    };

    assert_eq!(model.type_definitions[0].fields[0].ty, semantic_array);
    assert_eq!(
        model.function_definitions[0].parameters[0].ty,
        semantic_array
    );
    assert_eq!(
        model.function_definitions[0].return_type,
        semantic::ReturnType::Value(semantic_integer.clone())
    );
    assert_eq!(model.bindings["value"].ty, semantic_integer);

    let program = compile_to_ir(source).unwrap();
    let integer = ir::Type::Integer(IntegerType::I64);
    let array = ir::Type::Array {
        element: Box::new(ir::Type::Array {
            element: Box::new(integer.clone()),
            length: 2,
        }),
        length: 2,
    };
    assert_eq!(program.type_definitions[0].fields[0].ty, array);
    let function = &program.function_definitions[0];
    assert_eq!(function.parameters[0].ty, array);
    assert_eq!(function.return_type, ir::ReturnType::Value(integer.clone()));
    let StatementKind::Return { value: Some(value) } = &function.body[0].kind else {
        panic!("expected return value");
    };
    assert_eq!(value.ty, integer);
    let StatementKind::Binding { ty, value, .. } = &program.statements[1].kind else {
        panic!("expected binding");
    };
    assert_eq!(*ty, integer);
    assert_eq!(value.ty, integer);
}

#[test]
fn unimplemented_integer_types_are_not_implicitly_treated_as_i64() {
    for name in ["i8", "i16", "u8", "u16", "u64"] {
        let source = format!("value: {name} = 1;");
        let error = compile_to_ir(&source).unwrap_err();

        assert_eq!(error.message(), format!("unknown type `{name}`"));
    }
}
