use std::fmt::Write;

use crate::ast::{BinaryOp, Expr, Program, Stmt, Type, UnaryOp};
use crate::semantic::{Bindings, type_of_expr};

pub fn emit_wat(program: &Program, bindings: &Bindings) -> String {
    let mut output = String::new();

    writeln!(output, "(module").unwrap();

    // print() is provided by the host.
    writeln!(
        output,
        "  (import \"primer\" \"print_i64\" (func $print_i64 (param i64)))"
    )
    .unwrap();

    writeln!(
        output,
        "  (import \"primer\" \"print_f32\" (func $print_f32 (param f32)))"
    )
    .unwrap();

    writeln!(
        output,
        "  (import \"primer\" \"print_f64\" (func $print_f64 (param f64)))"
    )
    .unwrap();

    writeln!(output).unwrap();

    writeln!(output, "  (func $main").unwrap();

    // WebAssembly locals must be declared as part of the function,
    // so declare all Primer bindings first.
    for statement in &program.statements {
        if let Stmt::Binding { name, .. } = statement {
            let ty = bindings
                .get(name)
                .copied()
                .expect("binding must have been resolved by type checker");

            writeln!(output, "    (local $primer_{} {})", name, wat_type(ty),).unwrap();
        }
    }

    if program
        .statements
        .iter()
        .any(|statement| matches!(statement, Stmt::Binding { .. }))
    {
        writeln!(output).unwrap();
    }

    for statement in &program.statements {
        emit_statement(statement, bindings, &mut output);
    }

    writeln!(output, "  )").unwrap();

    writeln!(output, "  (export \"main\" (func $main))").unwrap();

    writeln!(output, ")").unwrap();

    output
}

fn emit_statement(statement: &Stmt, bindings: &Bindings, output: &mut String) {
    match statement {
        Stmt::Binding { name, value, .. } => {
            let ty = bindings
                .get(name)
                .copied()
                .expect("binding must have been resolved by type checker");

            emit_expr(value, Some(ty), bindings, output);

            writeln!(output, "    local.set $primer_{name}").unwrap();
        }

        Stmt::Print { value } => {
            let ty = type_of_expr(value, bindings).expect("expression must have been checked");

            emit_expr(value, Some(ty), bindings, output);

            let print_function = match ty {
                Type::I64 => "$print_i64",
                Type::F32 => "$print_f32",
                Type::F64 => "$print_f64",
            };

            writeln!(output, "    call {print_function}").unwrap();
        }
    }
}

fn emit_expr(expr: &Expr, expected: Option<Type>, bindings: &Bindings, output: &mut String) {
    match expr {
        Expr::Integer(value) => {
            writeln!(output, "    i64.const {value}").unwrap();
        }

        Expr::Float {
            text,
            explicit_type,
        } => {
            let ty = match explicit_type {
                Some(ty) => *ty,

                None => match expected {
                    Some(Type::F32) => Type::F32,
                    _ => Type::F64,
                },
            };

            writeln!(output, "    {}.const {}", wat_type(ty), text,).unwrap();
        }

        Expr::Variable(name) => {
            writeln!(output, "    local.get $primer_{name}").unwrap();
        }

        Expr::Unary { op, value } => {
            let ty = match expected {
                Some(ty) => ty,

                None => type_of_expr(expr, bindings).expect("expression must have been checked"),
            };

            match (op, ty) {
                (UnaryOp::Negate, Type::I64) => {
                    writeln!(output, "    i64.const 0").unwrap();

                    emit_expr(value, Some(Type::I64), bindings, output);

                    writeln!(output, "    i64.sub").unwrap();
                }

                (UnaryOp::Negate, Type::F32) => {
                    emit_expr(value, Some(Type::F32), bindings, output);

                    writeln!(output, "    f32.neg").unwrap();
                }

                (UnaryOp::Negate, Type::F64) => {
                    emit_expr(value, Some(Type::F64), bindings, output);

                    writeln!(output, "    f64.neg").unwrap();
                }
            }
        }

        Expr::Binary { op, left, right } => {
            let ty = match expected {
                Some(ty) => ty,

                None => type_of_expr(expr, bindings).expect("expression must have been checked"),
            };

            emit_expr(left, Some(ty), bindings, output);

            emit_expr(right, Some(ty), bindings, output);

            writeln!(output, "    {}", wat_binary_instruction(*op, ty),).unwrap();
        }
    }
}

fn wat_type(ty: Type) -> &'static str {
    match ty {
        Type::I64 => "i64",
        Type::F32 => "f32",
        Type::F64 => "f64",
    }
}

fn wat_binary_instruction(op: BinaryOp, ty: Type) -> &'static str {
    match (op, ty) {
        (BinaryOp::Add, Type::I64) => "i64.add",

        (BinaryOp::Subtract, Type::I64) => "i64.sub",

        (BinaryOp::Multiply, Type::I64) => "i64.mul",

        (BinaryOp::Divide, Type::I64) => "i64.div_s",

        (BinaryOp::Add, Type::F32) => "f32.add",

        (BinaryOp::Subtract, Type::F32) => "f32.sub",

        (BinaryOp::Multiply, Type::F32) => "f32.mul",

        (BinaryOp::Divide, Type::F32) => "f32.div",

        (BinaryOp::Add, Type::F64) => "f64.add",

        (BinaryOp::Subtract, Type::F64) => "f64.sub",

        (BinaryOp::Multiply, Type::F64) => "f64.mul",

        (BinaryOp::Divide, Type::F64) => "f64.div",
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::lex, parser::parse, semantic::check};

    use super::emit_wat;

    #[test]
    fn emits_i64_add() {
        let program = parse(
            lex("x: i64 = 1 + 2;
                 print(x);")
            .unwrap(),
        )
        .unwrap();

        let bindings = check(&program).unwrap();

        let wat = emit_wat(&program, &bindings);

        assert!(wat.contains("i64.add"));

        assert!(wat.contains("(local $primer_x i64)"));

        assert!(wat.contains("call $print_i64"));
    }

    #[test]
    fn emits_f32_add() {
        let program = parse(
            lex("x: f32 = 0.1 + 0.2;
                 print(x);")
            .unwrap(),
        )
        .unwrap();

        let bindings = check(&program).unwrap();

        let wat = emit_wat(&program, &bindings);

        assert!(wat.contains("f32.const 0.1"));

        assert!(wat.contains("f32.const 0.2"));

        assert!(wat.contains("f32.add"));

        assert!(wat.contains("call $print_f32"));
    }

    #[test]
    fn emits_f64_add() {
        let program = parse(
            lex("x: f64 = 0.1 + 0.2;
                 print(x);")
            .unwrap(),
        )
        .unwrap();

        let bindings = check(&program).unwrap();

        let wat = emit_wat(&program, &bindings);

        assert!(wat.contains("f64.add"));

        assert!(wat.contains("call $print_f64"));
    }

    #[test]
    fn inferred_f32_uses_f32() {
        let program = parse(
            lex("a: f32 = 0.1 + 0.2;
                 b: infer = a + a;")
            .unwrap(),
        )
        .unwrap();

        let bindings = check(&program).unwrap();

        let wat = emit_wat(&program, &bindings);

        assert!(wat.contains("(local $primer_b f32)"));

        assert!(wat.contains("f32.add"));
    }
}
