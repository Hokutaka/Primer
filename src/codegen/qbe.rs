use crate::ast::{BinaryOp, Expr, Program, Stmt, Type, UnaryOp};
use crate::semantic::{Bindings, type_of_expr};

pub fn emit_qbe(program: &Program, bindings: &Bindings) -> String {
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
        // printf format strings.
        //
        // b 10 = '\n'
        // b 0  = '\0'
        self.output
            .push_str("data $fmt_i64 = { b \"%lld\", b 10, b 0 }\n");

        self.output
            .push_str("data $fmt_f32 = { b \"%.9g\", b 10, b 0 }\n");

        self.output
            .push_str("data $fmt_f64 = { b \"%.17g\", b 10, b 0 }\n\n");

        self.output.push_str("export function w $main() {\n");

        self.output.push_str("@start\n");

        for statement in &program.statements {
            self.emit_statement(statement);
        }

        self.output.push_str("  ret 0\n");

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

                let value = self.emit_expr(value, Some(ty));

                self.output.push_str(&format!(
                    "  %primer_{name} ={} copy {}\n",
                    qbe_type(ty),
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
                    operand: qbe_float_literal(text, ty),
                }
            }

            Expr::Variable(name) => {
                let ty = self
                    .bindings
                    .get(name)
                    .copied()
                    .expect("variable must have been resolved by type checker");

                Value {
                    ty,
                    operand: format!("%primer_{name}"),
                }
            }

            Expr::Unary { op, value } => {
                let value = self.emit_expr(value, expected);

                let temp = self.next_temp();

                match op {
                    UnaryOp::Negate => {
                        self.output.push_str(&format!(
                            "  {temp} ={} neg {}\n",
                            qbe_type(value.ty),
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

                self.output.push_str(&format!(
                    "  {temp} ={} {} {}, {}\n",
                    qbe_type(ty),
                    qbe_binary_instruction(*op),
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
                let result = self.next_temp();

                self.output.push_str(&format!(
                    "  {result} =w call $printf(l $fmt_i64, ..., l {})\n",
                    value.operand,
                ));
            }

            Type::F32 => {
                // C varargs promote float to double.
                let extended = self.next_temp();

                self.output
                    .push_str(&format!("  {extended} =d exts {}\n", value.operand,));

                let result = self.next_temp();

                self.output.push_str(&format!(
                    "  {result} =w call $printf(l $fmt_f32, ..., d {extended})\n",
                ));
            }

            Type::F64 => {
                let result = self.next_temp();

                self.output.push_str(&format!(
                    "  {result} =w call $printf(l $fmt_f64, ..., d {})\n",
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

fn qbe_type(ty: Type) -> &'static str {
    match ty {
        Type::I64 => "l",
        Type::F32 => "s",
        Type::F64 => "d",
    }
}

fn qbe_binary_instruction(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "add",
        BinaryOp::Subtract => "sub",
        BinaryOp::Multiply => "mul",
        BinaryOp::Divide => "div",
    }
}

fn qbe_float_literal(text: &str, ty: Type) -> String {
    match ty {
        Type::F32 => {
            format!("s_{text}")
        }

        Type::F64 => {
            format!("d_{text}")
        }

        Type::I64 => {
            unreachable!("integer cannot be emitted as float")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::lex, parser::parse, semantic::check};

    use super::emit_qbe;

    #[test]
    fn emits_i64_add() {
        let program = parse(
            lex("x: i64 = 1 + 2;
                 print(x);")
            .unwrap(),
        )
        .unwrap();

        let bindings = check(&program).unwrap();

        let qbe = emit_qbe(&program, &bindings);

        assert!(qbe.contains("=l add 1, 2"));

        assert!(qbe.contains("%primer_x =l copy"));

        assert!(qbe.contains("call $printf(l $fmt_i64, ..., l %primer_x)"));
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

        let qbe = emit_qbe(&program, &bindings);

        assert!(qbe.contains("=s add s_0.1, s_0.2"));

        assert!(qbe.contains("=d exts %primer_x"));

        assert!(qbe.contains("call $printf(l $fmt_f32"));
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

        let qbe = emit_qbe(&program, &bindings);

        assert!(qbe.contains("=d add d_0.1, d_0.2"));

        assert!(qbe.contains("call $printf(l $fmt_f64"));
    }

    #[test]
    fn inferred_f32_uses_single() {
        let program = parse(
            lex("a: f32 = 0.1 + 0.2;
                 b: infer = a + a;")
            .unwrap(),
        )
        .unwrap();

        let bindings = check(&program).unwrap();

        let qbe = emit_qbe(&program, &bindings);

        assert!(qbe.contains("%primer_b =s copy"));
    }
}
