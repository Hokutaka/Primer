pub mod ast;
pub mod bytecode;
pub mod codegen;
pub mod diagnostic;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod source;
pub mod types;
pub mod vm;

use ast::Program;
use bytecode::InstructionOrigin;
use diagnostic::Diagnostic;

/// VM実行エラーと、失敗したbytecode命令の出自を保持します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionError {
    vm_error: vm::VmError,
    origin: Option<InstructionOrigin>,
}

impl ExecutionError {
    /// VMが報告した構造化エラーを返します。
    pub const fn vm_error(&self) -> &vm::VmError {
        &self.vm_error
    }

    /// 失敗した命令の出自を返します。
    ///
    /// 命令番号がbytecodeの範囲外で、対応する命令自体が存在しない場合は`None`です。
    pub const fn origin(&self) -> Option<InstructionOrigin> {
        self.origin
    }
}

/// `run_vm`で発生したコンパイルエラーまたはVM実行エラーを表します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    /// Primerソースからbytecodeを生成するまでに見つかった問題です。
    Compilation(Diagnostic),

    /// 生成されたbytecodeをPrimer VMで実行中に見つかった問題です。
    Execution(ExecutionError),
}

pub fn compile(source: &str) -> Result<Program, Diagnostic> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;

    semantic::check(&program)?;

    Ok(program)
}

pub fn compile_to_ir(source: &str) -> Result<ir::Program, Diagnostic> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;

    ir::builder::build(&program)
}

pub fn compile_to_ir_text(source: &str) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;

    Ok(ir::text::emit(&program))
}

// C コンパイラ
pub fn compile_to_c(source: &str) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;

    codegen::emit_c(&program)
}

// LLVM コンパイラ
pub fn compile_to_llvm(source: &str) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;

    codegen::emit_llvm(&program)
}

/// LLVMの実行環境を明示して生成します。Noneでは文字列にターゲット指定を求めます。
pub fn compile_to_llvm_with_target(
    source: &str,
    target: Option<codegen::llvm::Target>,
) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;
    codegen::llvm::emit_llvm_with_target(&program, target)
}

// Wasm コンパイラ
pub fn compile_to_wat(source: &str) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;

    codegen::emit_wat(&program)
}

// QBE コンパイラ
pub fn compile_to_qbe(source: &str) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;

    codegen::emit_qbe(&program)
}

/// QBEの実行環境を明示します。文字列ではLinux x86-64の指定が必要です。
pub fn compile_to_qbe_with_target(
    source: &str,
    target: Option<codegen::qbe::Target>,
) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;
    codegen::qbe::emit_qbe_with_target(&program, target)
}

// Windows x86-64 Direct Assembly コンパイラ
pub fn compile_to_x86_64_win_asm(source: &str) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;

    codegen::emit_x86_64_win_asm(&program)
}

pub fn compile_to_bytecode(source: &str) -> Result<bytecode::BytecodeProgram, Diagnostic> {
    let program = compile_to_ir(source)?;

    bytecode::lower(&program)
}

pub fn compile_to_bytecode_text(source: &str) -> Result<String, Diagnostic> {
    let bytecode = compile_to_bytecode(source)?;

    Ok(bytecode::format_program(&bytecode))
}

pub fn run_vm(source: &str) -> Result<String, RunError> {
    let bytecode = compile_to_bytecode(source).map_err(RunError::Compilation)?;

    vm::run(&bytecode).map_err(|vm_error| {
        let instructions = vm_error
            .function_id()
            .and_then(|function_id| bytecode.functions.get(function_id))
            .map_or(bytecode.instructions.as_slice(), |function| {
                function.instructions.as_slice()
            });
        let origin = instructions
            .get(vm_error.instruction_index())
            .map(|instruction| instruction.origin);

        RunError::Execution(ExecutionError { vm_error, origin })
    })
}

#[cfg(test)]
mod tests {
    use super::{
        RunError, compile_to_bytecode, compile_to_bytecode_text, compile_to_c, compile_to_ir_text,
        compile_to_llvm, compile_to_qbe, compile_to_wat, compile_to_x86_64_win_asm, run_vm,
    };
    use crate::{
        bytecode::{InstructionKind, InstructionOrigin},
        ir::NodeId,
        source::Span,
        vm::VmErrorKind,
    };

    #[test]
    fn emits_primer_ir_with_resolved_types() {
        let ir = compile_to_ir_text("x: f32 = 0.1 + 0.2; print(x);").unwrap();

        assert_eq!(
            ir,
            concat!(
                "; Primer IR v0.2\n",
                "; #N identifies one statement or expression in this compilation\n\n",
                "#0 %x@0: f32 = #1 add.f32(#2 0.1f32, #3 0.2f32)\n",
                "#4 print.f32 #5 %x@0:f32\n",
            )
        );
    }

    #[test]
    fn distinguishes_compilation_and_execution_errors() {
        let compilation_error = run_vm("print(missing);").unwrap_err();
        let execution_error = run_vm("print(1 / 0);").unwrap_err();

        assert!(matches!(compilation_error, RunError::Compilation(_)));

        match execution_error {
            RunError::Execution(error) => {
                assert_eq!(error.vm_error().kind(), VmErrorKind::DivisionByZero);
                assert_eq!(error.vm_error().instruction_index(), 2);
                assert_eq!(
                    error.origin(),
                    Some(InstructionOrigin::Source {
                        node_id: NodeId(1),
                        span: Span::new(6, 11),
                    })
                );
            }
            RunError::Compilation(diagnostic) => {
                panic!("expected execution error, found {diagnostic:?}");
            }
        }
    }

    #[test]
    fn emits_product_types_and_field_origins_in_primer_ir() {
        let source = "
            type Point { x: f64 = 0.0, y: f64, }
            point: Point = Point { y: 2.0, };
            print(point.x);
        ";
        let ir = compile_to_ir_text(source).unwrap();

        assert_eq!(
            ir,
            concat!(
                "; Primer IR v0.2\n",
                "; #N identifies one statement or expression in this compilation\n\n",
                "type %Point@0 {\n",
                "  field %x@0: f64 = #0 0.0f64\n",
                "  field %y@1: f64\n",
                "}\n\n",
                "#1 %point@0: %Point@0 = #2 construct %Point@0 { ",
                "field %y@1 = #3 2.0f64 [explicit]; ",
                "field %x@0 = #4 0.0f64 [default]; }\n",
                "#5 print.f64 #6 field(#7 %point@0:%Point@0, %x@0):f64\n",
            )
        );
    }

    #[test]
    fn c_backend_emits_product_layout_and_field_access() {
        let source = "
            type Point { x: f64 = 0.0, y: f64, }
            point: Point = Point { y: 2.0, };
            print(point.x);
        ";
        let c = compile_to_c(source).unwrap();

        assert!(c.contains("typedef struct primer_type_Point_0"));
        assert!(c.contains("primer_type_Point_0 primer_binding_0_point"));
        assert!(c.contains(".y = 2.0, .x = 0.0"));
        assert!(c.contains("(primer_binding_0_point).x"));
    }

    #[test]
    fn product_types_run_with_value_semantics() {
        let source = "
            type Point { x: i64, y: i64 = 2, }
            mut a: Point = Point { x: 1, };
            b: Point = a;
            a = Point { x: 3, y: 4, };
            print(b.x);
            print(a.y);
        ";

        assert_eq!(run_vm(source).unwrap(), "1\n4\n");
    }

    #[test]
    fn fixed_arrays_run_with_value_semantics_and_checked_indexing() {
        let source = "
            mut values: [i64; 4] = [2, 4, 6, 8];
            copy: [i64; 4] = values;
            values = [1, 2, 3, 4];
            mut total: i64 = 0;
            for (mut index: i64 = 0; index < 4; index = index + 1) {
                total = total + copy[index];
            }
            print(total);
            print(values[2]);
        ";

        assert_eq!(run_vm(source).unwrap(), "20\n3\n");

        let ir = compile_to_ir_text(source).unwrap();
        assert!(ir.contains("[i64; 4]"));
        assert!(ir.contains("array[#"));
        assert!(ir.contains("]:[i64; 4]"));
        assert!(ir.contains("index(#"));
        assert!(ir.contains("%copy@1:[i64; 4]"));
        assert!(ir.contains("%index@3:i64"));

        let bytecode = compile_to_bytecode_text(source).unwrap();
        assert!(bytecode.contains("array.new i64 4"));
        assert!(bytecode.contains("array.get i64 4"));

        let c = compile_to_c(source).unwrap();
        assert!(c.contains("primer_array_i64_4"));
        assert!(c.contains("primer_array_get_i64_4"));

        let llvm = compile_to_llvm(source).unwrap();
        assert!(llvm.contains("@primer.array.get.i64.4"));
        assert!(llvm.contains("icmp sge i64 %index, 4"));

        let wat = compile_to_wat(source).unwrap();
        assert!(wat.contains("i64.lt_s"));
        assert!(wat.contains("unreachable"));

        let qbe = compile_to_qbe(source).unwrap();
        assert!(qbe.contains("array_index_out_of_bounds"));
        assert!(qbe.contains("call $abort()"));

        let asm = compile_to_x86_64_win_asm(source).unwrap();
        assert!(asm.contains("array_oob"));
        assert!(asm.contains("ud2"));
    }

    #[test]
    fn fixed_array_elements_can_be_updated_without_changing_copies() {
        let source = "
            type Point { x: i64, y: i64, }
            mut matrix: [[i64; 2]; 2] = [[1, 2], [3, 4]];
            original: [[i64; 2]; 2] = matrix;
            matrix[0][1] = 20;
            matrix[1] = [30, 40];
            mut points: [Point; 2] = [
                Point { x: 1, y: 2, },
                Point { x: 3, y: 4, },
            ];
            points[0] = Point { x: 10, y: 20, };
            print(original[0][1]);
            print(matrix[0][1]);
            print(matrix[1][0]);
            print(points[0].y);
        ";

        assert_eq!(run_vm(source).unwrap(), "2\n20\n30\n20\n");
        let ir = compile_to_ir_text(source).unwrap();
        assert!(ir.contains("set %matrix@0:[[i64; 2]; 2]["));
        assert!(compile_to_c(source).is_ok());
        assert!(compile_to_llvm(source).is_ok());
        assert!(compile_to_qbe(source).is_ok());
        assert!(compile_to_wat(source).is_ok());
        assert!(compile_to_x86_64_win_asm(source).is_ok());
    }

    #[test]
    fn array_assignment_checks_indices_before_evaluating_its_value() {
        let source = "
            fn row() -> i64 { print(1); return 0; }
            fn column() -> i64 { print(2); return 1; }
            fn replacement() -> i64 { print(3); return 9; }
            mut matrix: [[i64; 2]; 2] = [[0, 0], [0, 0]];
            matrix[row()][column()] = replacement();
            print(matrix[0][1]);
        ";

        assert_eq!(run_vm(source).unwrap(), "1\n2\n3\n9\n");

        let bytecode = compile_to_bytecode(source).unwrap();
        let observable_order: Vec<String> = bytecode
            .instructions
            .iter()
            .filter_map(|instruction| match &instruction.kind {
                InstructionKind::Call { function_id, .. } => Some(format!("call {function_id}")),
                InstructionKind::ArrayCheck { path, .. } => Some(format!("check {}", path.len())),
                InstructionKind::ArrayAssign { .. } => Some("assign".into()),
                _ => None,
            })
            .collect();
        assert_eq!(
            observable_order,
            ["call 0", "check 1", "call 1", "check 2", "call 2", "assign"]
        );
    }

    #[test]
    fn array_assignment_out_of_bounds_points_to_the_failing_projection() {
        let source = "mut values: [i64; 2] = [1, 2]; values[2] = 3;";
        let RunError::Execution(error) = run_vm(source).unwrap_err() else {
            panic!("expected a VM execution error");
        };

        assert_eq!(
            error.vm_error().kind(),
            VmErrorKind::ArrayIndexOutOfBounds {
                index: 2,
                length: 2,
            }
        );
        let start = source.rfind("[2]").unwrap();
        assert_eq!(
            error.origin(),
            Some(InstructionOrigin::Source {
                node_id: NodeId(4),
                span: Span::new(start, start + 3),
            })
        );
    }

    #[test]
    fn product_types_can_hold_fixed_arrays() {
        let source = "
            type Row { values: [i64; 3], }
            mut first: Row = Row { values: [1, 2, 3], };
            second: Row = first;
            first = Row { values: [4, 5, 6], };
            print(second.values[1]);
            print(first.values[2]);
        ";

        assert_eq!(run_vm(source).unwrap(), "2\n6\n");
        assert!(compile_to_c(source).is_ok());
        assert!(compile_to_llvm(source).is_ok());
        assert!(compile_to_qbe(source).is_ok());
        assert!(compile_to_wat(source).is_ok());
        assert!(compile_to_x86_64_win_asm(source).is_ok());
    }

    #[test]
    fn fixed_arrays_can_hold_product_values() {
        let source = "
            type Point { x: i64, y: i64, }
            type Path { points: [Point; 2], }
            mut paths: [Path; 2] = [
                Path { points: [Point { x: 1, y: 2, }, Point { x: 3, y: 4, }], },
                Path { points: [Point { x: 5, y: 6, }, Point { x: 7, y: 8, }], },
            ];
            copy: [Path; 2] = paths;
            paths = [
                Path { points: [Point { x: 9, y: 10, }, Point { x: 11, y: 12, }], },
                Path { points: [Point { x: 13, y: 14, }, Point { x: 15, y: 16, }], },
            ];
            print(copy[1].points[0].x);
            print(paths[0].points[1].y);
        ";

        assert_eq!(run_vm(source).unwrap(), "5\n12\n");
        let bytecode = compile_to_bytecode_text(source).unwrap();
        assert!(bytecode.contains("array.new %Point@0 2"));
        assert!(bytecode.contains("array.new %Path@1 2"));
        assert!(bytecode.contains("array.get %Path@1 2"));
        assert!(compile_to_c(source).is_ok());
        assert!(compile_to_llvm(source).is_ok());
        assert!(compile_to_qbe(source).is_ok());
        assert!(compile_to_wat(source).is_ok());
        assert!(compile_to_x86_64_win_asm(source).is_ok());
    }

    #[test]
    fn fixed_arrays_can_be_nested() {
        let source = "
            mut matrix: [[i64; 3]; 2] = [[1, 2, 3], [4, 5, 6]];
            copy: [[i64; 3]; 2] = matrix;
            matrix = [[7, 8, 9], [10, 11, 12]];
            print(copy[1][2]);
            print(matrix[0][1]);
        ";

        assert_eq!(run_vm(source).unwrap(), "6\n8\n");
        let bytecode = compile_to_bytecode_text(source).unwrap();
        assert!(bytecode.contains("array.new [i64; 3] 2"));
        assert!(bytecode.contains("array.get [i64; 3] 2"));
        assert!(compile_to_c(source).is_ok());
        assert!(compile_to_llvm(source).is_ok());
        assert!(compile_to_qbe(source).is_ok());
        assert!(compile_to_wat(source).is_ok());
        assert!(compile_to_x86_64_win_asm(source).is_ok());
    }

    #[test]
    fn nested_fixed_arrays_can_hold_product_values() {
        let source = "
            type Point { x: i64, y: i64, }
            grid: [[Point; 2]; 2] = [
                [Point { x: 1, y: 2, }, Point { x: 3, y: 4, }],
                [Point { x: 5, y: 6, }, Point { x: 7, y: 8, }],
            ];
            print(grid[1][0].x);
            print(grid[0][1].y);
        ";

        assert_eq!(run_vm(source).unwrap(), "5\n4\n");
        let bytecode = compile_to_bytecode_text(source).unwrap();
        assert!(bytecode.contains("array.new [%Point@0; 2] 2"));
        assert!(bytecode.contains("array.get [%Point@0; 2] 2"));
        assert!(compile_to_c(source).is_ok());
        assert!(compile_to_llvm(source).is_ok());
        assert!(compile_to_qbe(source).is_ok());
        assert!(compile_to_wat(source).is_ok());
        assert!(compile_to_x86_64_win_asm(source).is_ok());
    }

    #[test]
    fn fixed_array_fields_support_defaults() {
        let source = "
            type Row { id: i64, values: [i64; 2] = [7, 8], }
            row: Row = Row { id: 1, };
            print(row.values[0]);
        ";

        assert_eq!(run_vm(source).unwrap(), "7\n");
        assert!(compile_to_ir_text(source).unwrap().contains("[default]"));
    }

    #[test]
    fn c_backend_declares_unused_array_field_types() {
        let c = compile_to_c("type Row { values: [i64; 3], }").unwrap();
        let array = c.find("typedef struct primer_array_i64_3").unwrap();
        let product = c.find("typedef struct primer_type_Row_0").unwrap();

        assert!(array < product);
    }

    #[test]
    fn fixed_array_out_of_bounds_keeps_its_source_origin() {
        let source = "values: [i64; 2] = [10, 20]; print(values[2]);";
        let RunError::Execution(error) = run_vm(source).unwrap_err() else {
            panic!("expected a VM execution error");
        };

        assert_eq!(
            error.vm_error().kind(),
            VmErrorKind::ArrayIndexOutOfBounds {
                index: 2,
                length: 2,
            }
        );
        assert_eq!(
            error.origin(),
            Some(InstructionOrigin::Source {
                node_id: NodeId(5),
                span: Span::new(35, 44),
            })
        );

        let RunError::Execution(error) =
            run_vm("values: [i64; 2] = [10, 20]; print(values[-1]);").unwrap_err()
        else {
            panic!("expected a VM execution error");
        };
        assert_eq!(
            error.vm_error().kind(),
            VmErrorKind::ArrayIndexOutOfBounds {
                index: -1,
                length: 2,
            }
        );
    }

    #[test]
    fn fixed_array_literal_can_be_indexed_directly() {
        let source = "print([1, 2][1]);";

        assert_eq!(run_vm(source).unwrap(), "2\n");
        assert!(
            compile_to_llvm(source)
                .unwrap()
                .contains("define internal i64 @primer.array.get.i64.2")
        );
    }

    #[test]
    fn fixed_arrays_support_every_scalar_element_type() {
        let source = "
            flags: [bool; 2] = [false, true];
            integers: [i64; 2] = [1, 2];
            singles: [f32; 2] = [0.5, 1.25];
            doubles: [f64; 2] = [1.5, 2.5];
            print(flags[1]);
            print(integers[1]);
            print(singles[1]);
            print(doubles[1]);
        ";

        assert_eq!(run_vm(source).unwrap(), "true\n2\n1.25\n2.5\n");
        assert!(compile_to_c(source).is_ok());
        assert!(compile_to_llvm(source).is_ok());
        assert!(compile_to_qbe(source).is_ok());
        assert!(compile_to_wat(source).is_ok());
        assert!(compile_to_x86_64_win_asm(source).is_ok());
    }

    #[test]
    fn fixed_arrays_can_be_local_to_functions() {
        let source = "
            fn pick(index: i64) -> i64 {
                values: [i64; 3] = [10, 20, 30];
                return values[index];
            }
            print(pick(1));
        ";

        assert_eq!(run_vm(source).unwrap(), "20\n");
        assert!(compile_to_c(source).is_ok());
        assert!(compile_to_llvm(source).is_ok());
        assert!(compile_to_qbe(source).is_ok());
        assert!(compile_to_wat(source).is_ok());
        assert!(compile_to_x86_64_win_asm(source).is_ok());
    }

    #[test]
    fn functions_run_in_independent_vm_frames() {
        let source = "
            fn add(left: i64, right: i64) -> i64 {
                result: i64 = left + right;
                return result;
            }
            fn show(value: i64) -> void { print(value); }
            answer: i64 = add(20, 22);
            show(answer);
        ";

        assert_eq!(run_vm(source).unwrap(), "42\n");
    }

    #[test]
    fn product_values_cross_function_boundaries_by_value() {
        let source = "
            type Point { x: i64, y: i64, }

            fn move_x(point: Point, amount: i64) -> Point {
                return Point { x: point.x + amount, y: point.y, };
            }

            fn move_twice(point: Point, amount: i64) -> Point {
                return move_x(move_x(point, amount), amount);
            }

            original: Point = Point { x: 2, y: 3, };
            moved: Point = move_twice(original, 5);
            print(original.x);
            print(moved.x);
            print(moved.y);
        ";

        assert_eq!(run_vm(source).unwrap(), "2\n12\n3\n");
        assert!(compile_to_c(source).is_ok());
        assert!(compile_to_llvm(source).is_ok());
        assert!(compile_to_qbe(source).is_ok());
        assert!(compile_to_wat(source).is_ok());
        assert!(compile_to_x86_64_win_asm(source).is_ok());
    }

    #[test]
    fn array_values_cross_function_boundaries_by_value() {
        let source = "
            fn first_row(matrix: [[i64; 2]; 2]) -> [i64; 2] {
                return matrix[0];
            }

            fn duplicate(row: [i64; 2]) -> [[i64; 2]; 2] {
                return [row, row];
            }

            fn duplicate_first_row(matrix: [[i64; 2]; 2]) -> [[i64; 2]; 2] {
                return duplicate(first_row(matrix));
            }

            matrix: [[i64; 2]; 2] = [[1, 2], [3, 4]];
            result: [[i64; 2]; 2] = duplicate_first_row(matrix);
            print(matrix[1][0]);
            print(result[0][1]);
            print(result[1][0]);
        ";

        assert_eq!(run_vm(source).unwrap(), "3\n2\n1\n");
        assert!(compile_to_c(source).is_ok());
        assert!(compile_to_llvm(source).is_ok());
        assert!(compile_to_qbe(source).is_ok());
        assert!(compile_to_wat(source).is_ok());
        assert!(compile_to_x86_64_win_asm(source).is_ok());
    }

    #[test]
    fn preserves_the_origin_of_vm_errors_inside_functions() {
        let source = "fn fail(value: i64) -> i64 { return value / 0; }
                      answer: i64 = fail(1);";
        let error = run_vm(source).unwrap_err();
        let RunError::Execution(error) = error else {
            panic!("expected a VM execution error");
        };

        assert_eq!(error.vm_error().kind(), VmErrorKind::DivisionByZero);
        assert_eq!(error.vm_error().function_id(), Some(0));
        assert_eq!(error.vm_error().instruction_index(), 2);
        assert_eq!(
            error.origin(),
            Some(InstructionOrigin::Source {
                node_id: NodeId(1),
                span: Span::new(36, 45),
            })
        );
    }

    #[test]
    fn bytecode_exposes_product_construction_and_field_access() {
        let source = "
            type Point { x: i64, y: i64 = 2, }
            point: Point = Point { x: 1, };
            print(point.y);
        ";
        let bytecode = compile_to_bytecode_text(source).unwrap();

        assert!(bytecode.contains(".type 0 Point"));
        assert!(bytecode.contains("construct %Point@0 [%x@0:explicit, %y@1:default]"));
        assert!(bytecode.contains("field.get %Point@0.%y@1"));
    }
}
