use crate::ast::{BinaryOp, Expr, Program, Stmt, Type, UnaryOp};
use crate::semantic::{Bindings, type_of_expr};

pub fn emit_llvm(program: &Program, bindings: &Bindings) -> String {
    let mut generator = Generator {
        output: String::new(),
        bindings,
        temp: 0,
    };

    generator.emit_program(program);
    generator.output
}

struct Generator<'a> {
    output: String,
    bindings: &'a Bindings,
    temp: usize,
}

struct Value {
    ty: Type,
    operand: String,
}

impl Generator<'_> {
    fn emit_program(&mut self, program: &Program) {
        self.output
            .push_str("@.fmt_i64 = private unnamed_addr constant [6 x i8] c\"%lld\\0A\\00\"\n");
        self.output
            .push_str("@.fmt_f32 = private unnamed_addr constant [6 x i8] c\"%.9g\\0A\\00\"\n");
        self.output
            .push_str("@.fmt_f64 = private unnamed_addr constant [7 x i8] c\"%.17g\\0A\\00\"\n\n");

        self.output.push_str("declare i32 @printf(ptr, ...)\n\n");

        self.output.push_str("define i32 @main() {\n");
        self.output.push_str("entry:\n");

        for statement in &program.statements {
            self.emit_statement(statement);
        }

        self.output.push_str("  ret i32 0\n");
        self.output.push_str("}\n");
    }

    fn emit_statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Binding { name, value, .. } => {
                let ty = self
                    .bindings
                    .get(name)
                    .copied()
                    .expect("binding must have been resolved by type checker");

                let pointer = format!("%primer_{name}");

                self.output
                    .push_str(&format!("  {pointer} = alloca {}\n", llvm_type(ty),));

                let value = self.emit_expr(value, Some(ty));

                self.output.push_str(&format!(
                    "  store {} {}, ptr {pointer}\n",
                    llvm_type(ty),
                    value.operand,
                ));
            }

            Stmt::Print { value } => {
                let ty =
                    type_of_expr(value, self.bindings).expect("expression must have been checked");

                let value = self.emit_expr(value, Some(ty));

                self.emit_print(value);
            }
        }
    }

    fn emit_expr(&mut self, expr: &Expr, expected: Option<Type>) -> Value {
        match expr {
            Expr::Integer(value) => Value {
                ty: Type::I64,
                operand: value.to_string(),
            },

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

                Value {
                    ty,
                    operand: llvm_float_literal(text, ty),
                }
            }

            Expr::Variable(name) => {
                let ty = self
                    .bindings
                    .get(name)
                    .copied()
                    .expect("variable must have been resolved by type checker");

                let temp = self.next_temp();

                self.output.push_str(&format!(
                    "  {temp} = load {}, ptr %primer_{name}\n",
                    llvm_type(ty),
                ));

                Value { ty, operand: temp }
            }

            Expr::Unary { op, value } => {
                let value = self.emit_expr(value, expected);

                let temp = self.next_temp();

                match (op, value.ty) {
                    (UnaryOp::Negate, Type::I64) => {
                        self.output
                            .push_str(&format!("  {temp} = sub i64 0, {}\n", value.operand,));
                    }

                    (UnaryOp::Negate, Type::F32 | Type::F64) => {
                        self.output.push_str(&format!(
                            "  {temp} = fneg {} {}\n",
                            llvm_type(value.ty),
                            value.operand,
                        ));
                    }
                }

                Value {
                    ty: value.ty,
                    operand: temp,
                }
            }

            Expr::Binary { op, left, right } => {
                let ty = match expected {
                    Some(ty) => ty,

                    None => type_of_expr(expr, self.bindings)
                        .expect("expression must have been checked"),
                };

                let left = self.emit_expr(left, Some(ty));

                let right = self.emit_expr(right, Some(ty));

                let temp = self.next_temp();

                let instruction = llvm_binary_instruction(*op, ty);

                self.output.push_str(&format!(
                    "  {temp} = {instruction} {} {}, {}\n",
                    llvm_type(ty),
                    left.operand,
                    right.operand,
                ));

                Value { ty, operand: temp }
            }
        }
    }

    fn emit_print(&mut self, value: Value) {
        match value.ty {
            Type::I64 => {
                self.output.push_str(&format!(
                    "  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 {})\n",
                    value.operand,
                ));
            }

            Type::F32 => {
                // C varargs promote float to double.
                let extended = self.next_temp();

                self.output.push_str(&format!(
                    "  {extended} = fpext float {} to double\n",
                    value.operand,
                ));

                self.output.push_str(&format!(
                    "  call i32 (ptr, ...) @printf(ptr @.fmt_f32, double {extended})\n",
                ));
            }

            Type::F64 => {
                self.output.push_str(&format!(
                    "  call i32 (ptr, ...) @printf(ptr @.fmt_f64, double {})\n",
                    value.operand,
                ));
            }
        }
    }

    fn next_temp(&mut self) -> String {
        let temp = format!("%tmp{}", self.temp);

        self.temp += 1;

        temp
    }
}

fn llvm_type(ty: Type) -> &'static str {
    match ty {
        Type::I64 => "i64",
        Type::F32 => "float",
        Type::F64 => "double",
    }
}

fn llvm_binary_instruction(op: BinaryOp, ty: Type) -> &'static str {
    match (op, ty) {
        (BinaryOp::Add, Type::I64) => "add",
        (BinaryOp::Subtract, Type::I64) => "sub",
        (BinaryOp::Multiply, Type::I64) => "mul",
        (BinaryOp::Divide, Type::I64) => "sdiv",

        (BinaryOp::Add, Type::F32 | Type::F64) => "fadd",

        (BinaryOp::Subtract, Type::F32 | Type::F64) => "fsub",

        (BinaryOp::Multiply, Type::F32 | Type::F64) => "fmul",

        (BinaryOp::Divide, Type::F32 | Type::F64) => "fdiv",
    }
}

fn llvm_float_literal(text: &str, ty: Type) -> String {
    match ty {
        Type::F32 => {
            let value = text
                .parse::<f32>()
                .expect("validated floating-point literal");

            // LLVM 22 legacy hexadecimal syntax represents
            // float constants using the corresponding exact
            // double representation.
            let as_double = value as f64;

            format!("0x{:016X}", as_double.to_bits())
        }

        Type::F64 => {
            let value = text
                .parse::<f64>()
                .expect("validated floating-point literal");

            format!("0x{:016X}", value.to_bits())
        }

        Type::I64 => {
            unreachable!("integer cannot be emitted as float")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::lex, parser::parse, semantic::check};

    use super::emit_llvm;

    #[test]
    fn emits_i64_add() {
        let program = parse(
            lex("x: i64 = 1 + 2;
                 print(x);")
            .unwrap(),
        )
        .unwrap();

        let bindings = check(&program).unwrap();

        let llvm = emit_llvm(&program, &bindings);

        assert!(llvm.contains("add i64 1, 2"));
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

        let llvm = emit_llvm(&program, &bindings);

        assert!(llvm.contains("fadd float"));

        assert!(llvm.contains("fpext float"));
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

        let llvm = emit_llvm(&program, &bindings);

        assert!(llvm.contains("fadd double"));
    }

    #[test]
    fn emits_llvm_22_compatible_float_literals() {
        let program = parse(
            lex("x: f32 = 0.1 + 0.2;
                print(x);")
            .unwrap(),
        )
        .unwrap();

        let bindings = check(&program).unwrap();

        let llvm = emit_llvm(&program, &bindings);

        assert!(llvm.contains("fadd float 0x3FB99999A0000000, 0x3FC99999A0000000"));
    }
}
