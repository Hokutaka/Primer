use std::fmt::Write;

use super::{
    AssignmentProjection, BinaryOp, Expr, ExprKind, FieldValueOrigin, Program, ReturnType,
    Statement, StatementKind, Type, UnaryOp,
};

pub fn emit(program: &Program) -> String {
    let mut output = String::new();
    writeln!(output, "; Primer IR v0.2").unwrap();
    writeln!(
        output,
        "; #N identifies one statement or expression in this compilation"
    )
    .unwrap();

    if !program.type_definitions.is_empty()
        || !program.function_definitions.is_empty()
        || !program.statements.is_empty()
    {
        writeln!(output).unwrap();
    }

    for definition in &program.type_definitions {
        writeln!(output, "type %{}@{} {{", definition.name, definition.id.0).unwrap();
        for field in &definition.fields {
            write!(
                output,
                "  field %{}@{}: {}",
                field.name,
                field.id.0,
                type_name(&field.ty, program)
            )
            .unwrap();
            if let Some(default) = &field.default {
                output.push_str(" = ");
                emit_expr(default, program, &mut output);
            }
            writeln!(output).unwrap();
        }
        writeln!(output, "}}").unwrap();
        if !program.function_definitions.is_empty() || !program.statements.is_empty() {
            writeln!(output).unwrap();
        }
    }

    for (index, function) in program.function_definitions.iter().enumerate() {
        write!(output, "fn %{}@{}(", function.name, function.id.0).unwrap();
        for (parameter_index, parameter) in function.parameters.iter().enumerate() {
            if parameter_index > 0 {
                output.push_str(", ");
            }
            write!(
                output,
                "%{}@{}: {}",
                parameter.name,
                parameter.id.0,
                type_name(&parameter.ty, program)
            )
            .unwrap();
        }
        match &function.return_type {
            ReturnType::Void => writeln!(output, ") -> void {{").unwrap(),
            ReturnType::Value(ty) => {
                writeln!(output, ") -> {} {{", type_name(ty, program)).unwrap()
            }
        }
        for statement in &function.body {
            emit_statement(statement, 1, program, &mut output);
        }
        writeln!(output, "}}").unwrap();
        if index + 1 < program.function_definitions.len() || !program.statements.is_empty() {
            writeln!(output).unwrap();
        }
    }

    for statement in &program.statements {
        emit_statement(statement, 0, program, &mut output);
    }

    output
}

fn emit_statement(statement: &Statement, indent: usize, program: &Program, output: &mut String) {
    let prefix = "  ".repeat(indent);
    let node = format!("#{} ", statement.id.0);

    match &statement.kind {
        StatementKind::Binding {
            id,
            mutable,
            name,
            ty,
            value,
        } => {
            if *mutable {
                output.push_str(&prefix);
                output.push_str(&node);
                output.push_str("mut ");
            } else {
                output.push_str(&prefix);
                output.push_str(&node);
            }
            write!(output, "%{name}@{}: {} = ", id.0, type_name(ty, program)).unwrap();
            emit_expr(value, program, output);
            writeln!(output).unwrap();
        }
        StatementKind::Assignment { target, value } => {
            write!(
                output,
                "{prefix}{node}set %{}@{}:{}",
                target.name,
                target.id.0,
                type_name(&target.root_ty, program)
            )
            .unwrap();
            for projection in &target.projections {
                let AssignmentProjection::Index { index, .. } = projection;
                output.push('[');
                emit_expr(index, program, output);
                output.push(']');
            }
            if !target.projections.is_empty() {
                write!(output, ":{}", type_name(&target.ty, program)).unwrap();
            }
            output.push_str(" = ");
            emit_expr(value, program, output);
            writeln!(output).unwrap();
        }
        StatementKind::Print { value } => {
            write!(
                output,
                "{prefix}{node}print.{} ",
                type_name(&value.ty, program)
            )
            .unwrap();
            emit_expr(value, program, output);
            writeln!(output).unwrap();
        }
        StatementKind::Call {
            function_id,
            function_name,
            arguments,
        } => {
            write!(
                output,
                "{prefix}{node}call %{function_name}@{}(",
                function_id.0
            )
            .unwrap();
            emit_arguments(arguments, program, output);
            writeln!(output, ")").unwrap();
        }
        StatementKind::Return { value } => {
            write!(output, "{prefix}{node}return").unwrap();
            if let Some(value) = value {
                output.push(' ');
                emit_expr(value, program, output);
            }
            writeln!(output).unwrap();
        }
        StatementKind::If {
            condition,
            then_body,
            else_body,
        } => {
            write!(output, "{prefix}{node}if.bool ").unwrap();
            emit_expr(condition, program, output);
            writeln!(output, " {{").unwrap();
            for statement in then_body {
                emit_statement(statement, indent + 1, program, output);
            }
            write!(output, "{prefix}}}").unwrap();

            if else_body.is_empty() {
                writeln!(output).unwrap();
            } else {
                writeln!(output, " else {{").unwrap();
                for statement in else_body {
                    emit_statement(statement, indent + 1, program, output);
                }
                writeln!(output, "{prefix}}}").unwrap();
            }
        }
        StatementKind::While { condition, body } => {
            write!(output, "{prefix}{node}while.bool ").unwrap();
            emit_expr(condition, program, output);
            writeln!(output, " {{").unwrap();
            for statement in body {
                emit_statement(statement, indent + 1, program, output);
            }
            writeln!(output, "{prefix}}}").unwrap();
        }
        StatementKind::For {
            initializer,
            condition,
            update,
            body,
        } => {
            writeln!(output, "{prefix}{node}for.loop {{").unwrap();
            writeln!(output, "{prefix}  start {{").unwrap();
            emit_statement(initializer, indent + 2, program, output);
            writeln!(output, "{prefix}  }}").unwrap();
            write!(output, "{prefix}  condition.bool ").unwrap();
            emit_expr(condition, program, output);
            writeln!(output).unwrap();
            writeln!(output, "{prefix}  body {{").unwrap();
            for statement in body {
                emit_statement(statement, indent + 2, program, output);
            }
            writeln!(output, "{prefix}  }}").unwrap();
            writeln!(output, "{prefix}  update {{").unwrap();
            emit_statement(update, indent + 2, program, output);
            writeln!(output, "{prefix}  }}").unwrap();
            writeln!(output, "{prefix}}}").unwrap();
        }
        StatementKind::Break => {
            writeln!(output, "{prefix}{node}break").unwrap();
        }
        StatementKind::Continue => {
            writeln!(output, "{prefix}{node}continue").unwrap();
        }
    }
}

fn emit_expr(expr: &Expr, program: &Program, output: &mut String) {
    write!(output, "#{} ", expr.id.0).unwrap();

    match &expr.kind {
        ExprKind::StringByteLength { value } => {
            output.push_str("byte_len.string(");
            emit_expr(value, program, output);
            output.push(')');
        }
        ExprKind::ConvertNumeric {
            value,
            from,
            to,
            syntax,
        } => {
            let spelling = match syntax {
                crate::source::ConversionSyntax::Compact => "compact",
                crate::source::ConversionSyntax::Explicit => "explicit",
            };
            write!(
                output,
                "convert.exact.{}->{}[{spelling}](",
                from.name(),
                to.name()
            )
            .unwrap();
            emit_expr(value, program, output);
            output.push(')');
        }
        ExprKind::ConvertInteger {
            value,
            from,
            to,
            syntax,
        } => {
            let spelling = match syntax {
                crate::source::ConversionSyntax::Compact => "compact",
                crate::source::ConversionSyntax::Explicit => "explicit",
            };
            write!(
                output,
                "convert.checked.{}->{}[{spelling}](",
                from.name(),
                to.name()
            )
            .unwrap();
            emit_expr(value, program, output);
            output.push(')');
        }
        ExprKind::Boolean(value) => {
            write!(output, "{value}:bool").unwrap();
        }
        ExprKind::String(value) => {
            // 改行や制御文字をエスケープし、一つの観測行に収めます。
            write!(output, "{value:?}:string").unwrap();
        }
        ExprKind::Integer(value) => {
            write!(output, "{value}{}", type_name(&expr.ty, program)).unwrap();
        }
        ExprKind::Float { text } => {
            write!(output, "{text}{}", type_name(&expr.ty, program)).unwrap();
        }
        ExprKind::Variable { id, name } => {
            write!(output, "%{name}@{}:{}", id.0, type_name(&expr.ty, program)).unwrap();
        }
        ExprKind::Construct {
            type_id,
            type_name,
            fields,
        } => {
            write!(output, "construct %{type_name}@{} {{", type_id.0).unwrap();
            for field in fields {
                write!(output, " field %{}@{} = ", field.name, field.id.0).unwrap();
                emit_expr(&field.value, program, output);
                match field.origin {
                    FieldValueOrigin::Explicit { .. } => output.push_str(" [explicit]"),
                    FieldValueOrigin::Default { .. } => output.push_str(" [default]"),
                }
                output.push(';');
            }
            output.push_str(" }");
        }
        ExprKind::FieldAccess {
            field_id,
            field_name,
            base,
            ..
        } => {
            output.push_str("field(");
            emit_expr(base, program, output);
            write!(
                output,
                ", %{field_name}@{}):{}",
                field_id.0,
                type_name(&expr.ty, program)
            )
            .unwrap();
        }
        ExprKind::Array(values) => {
            output.push_str("array[");
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                emit_expr(value, program, output);
            }
            write!(output, "]:{}", type_name(&expr.ty, program)).unwrap();
        }
        ExprKind::Index { base, index } => {
            output.push_str("index(");
            emit_expr(base, program, output);
            output.push_str(", ");
            emit_expr(index, program, output);
            write!(output, "):{}", type_name(&expr.ty, program)).unwrap();
        }
        ExprKind::Call {
            function_id,
            function_name,
            arguments,
        } => {
            write!(output, "call %{function_name}@{}(", function_id.0).unwrap();
            emit_arguments(arguments, program, output);
            write!(output, "):{}", type_name(&expr.ty, program)).unwrap();
        }
        ExprKind::Unary { op, value } => {
            write!(
                output,
                "{}.{}(",
                unary_name(*op),
                type_name(&expr.ty, program)
            )
            .unwrap();
            emit_expr(value, program, output);
            output.push(')');
        }
        ExprKind::Logical { op, left, right } => {
            output.push_str(match op {
                super::LogicalOp::And => "and.short_circuit.bool(",
                super::LogicalOp::Or => "or.short_circuit.bool(",
            });
            emit_expr(left, program, output);
            output.push_str(", ");
            emit_expr(right, program, output);
            output.push(')');
        }
        ExprKind::Binary { op, left, right } => {
            let operation_type = if is_comparison(*op) {
                &left.ty
            } else {
                &expr.ty
            };

            write!(
                output,
                "{}.{}(",
                binary_name(*op),
                type_name(operation_type, program)
            )
            .unwrap();
            emit_expr(left, program, output);
            output.push_str(", ");
            emit_expr(right, program, output);
            output.push(')');
        }
    }
}

fn emit_arguments(arguments: &[Expr], program: &Program, output: &mut String) {
    for (index, argument) in arguments.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        emit_expr(argument, program, output);
    }
}

fn type_name(ty: &Type, program: &Program) -> String {
    match ty {
        Type::Bool => "bool".into(),
        Type::String => "string".into(),
        Type::Integer(integer) => integer.name().into(),
        Type::F32 => "f32".into(),
        Type::F64 => "f64".into(),
        Type::Named(id) => {
            let definition = &program.type_definitions[id.0];
            format!("%{}@{}", definition.name, id.0)
        }
        Type::Array { element, length } => {
            format!("[{}; {length}]", type_name(element, program))
        }
    }
}

fn unary_name(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Negate => "neg",
        UnaryOp::Not => "not",
        UnaryOp::BitNot => "bit_not",
    }
}

fn binary_name(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "add",
        BinaryOp::Subtract => "sub",
        BinaryOp::Multiply => "mul",
        BinaryOp::Divide => "div",
        BinaryOp::Remainder => "rem",
        BinaryOp::BitAnd => "bit_and",
        BinaryOp::BitOr => "bit_or",
        BinaryOp::BitXor => "bit_xor",
        BinaryOp::ShiftLeft => "shl.checked",
        BinaryOp::ShiftRight => "shr",
        BinaryOp::Equal => "eq",
        BinaryOp::NotEqual => "ne",
        BinaryOp::Less => "lt",
        BinaryOp::LessEqual => "le",
        BinaryOp::Greater => "gt",
        BinaryOp::GreaterEqual => "ge",
    }
}

const fn is_comparison(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual
    )
}

#[cfg(test)]
mod tests {
    use crate::ast::{
        BinaryOp as AstBinaryOp, Expr as AstExpr, ExprKind as AstExprKind, Item as AstItem,
        Program as AstProgram, Stmt, StmtKind as AstStmtKind, TypeRef, TypeSpec,
    };
    use crate::ir::builder::build;
    use crate::source::Span;

    use super::emit;

    fn ast_expr(kind: AstExprKind) -> AstExpr {
        AstExpr {
            kind,
            span: Span::empty(0),
        }
    }

    fn ast_stmt(kind: AstStmtKind) -> Stmt {
        Stmt {
            kind,
            span: Span::empty(0),
        }
    }

    #[test]
    fn emits_resolved_types() {
        let ast = AstProgram {
            items: vec![
                AstItem::Statement(ast_stmt(AstStmtKind::Binding {
                    mutable: false,
                    name: "x".into(),
                    type_spec: TypeSpec::Explicit(TypeRef {
                        kind: crate::ast::TypeRefKind::Named("f32".into()),
                        span: Span::empty(0),
                    }),
                    value: ast_expr(AstExprKind::Binary {
                        op: AstBinaryOp::Add,
                        left: Box::new(ast_expr(AstExprKind::Float {
                            text: "0.1".into(),
                            explicit_type: None,
                        })),
                        right: Box::new(ast_expr(AstExprKind::Float {
                            text: "0.2".into(),
                            explicit_type: None,
                        })),
                    }),
                })),
                AstItem::Statement(ast_stmt(AstStmtKind::Print {
                    value: ast_expr(AstExprKind::Variable("x".into())),
                })),
            ],
        };

        let text = emit(&build(&ast).unwrap());
        assert_eq!(
            text,
            concat!(
                "; Primer IR v0.2\n",
                "; #N identifies one statement or expression in this compilation\n\n",
                "#0 %x@0: f32 = #1 add.f32(#2 0.1f32, #3 0.2f32)\n",
                "#4 print.f32 #5 %x@0:f32\n",
            )
        );
    }
}
