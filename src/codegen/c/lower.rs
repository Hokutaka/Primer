use crate::ir as primer_ir;

use super::ir::{
    ArrayProjection, AssignmentTarget, BinaryOp, Expr, ExprKind, FieldDefinition, FieldValue,
    Function, Module, Parameter, PrintFormat, Statement, Type, TypeDefinition, UnaryOp,
};

pub fn lower(program: &primer_ir::Program) -> Module {
    let mut module = Module {
        temporaries: Vec::new(),
        array_types: collect_array_types(program),
        array_assignment_types: collect_array_assignment_types(program),
        type_definitions: lower_type_definitions(program),
        functions: program
            .function_definitions
            .iter()
            .map(|function| Function {
                temporaries: Vec::new(),
                id: function.id.0,
                name: function.name.clone(),
                parameters: function
                    .parameters
                    .iter()
                    .map(|parameter| Parameter {
                        name: binding_name(parameter.id, &parameter.name),
                        ty: parameter.ty.clone().into(),
                    })
                    .collect(),
                return_type: match &function.return_type {
                    primer_ir::ReturnType::Void => None,
                    primer_ir::ReturnType::Value(ty) => Some(ty.clone().into()),
                },
                body: function.body.iter().map(lower_statement).collect(),
            })
            .collect(),
        explicit_main: program
            .function_definitions
            .iter()
            .find(|function| function.name == "main")
            .map(|function| function.id.0),
        statements: program.statements.iter().map(lower_statement).collect(),
    };
    super::sequence::lower(&mut module);
    module
}

fn collect_array_assignment_types(program: &primer_ir::Program) -> Vec<Type> {
    fn visit(statements: &[primer_ir::Statement], result: &mut Vec<Type>) {
        for statement in statements {
            match &statement.kind {
                primer_ir::StatementKind::Assignment { target, .. } => {
                    for projection in &target.projections {
                        let primer_ir::AssignmentProjection::Index {
                            element, length, ..
                        } = projection;
                        let ty = Type::Array {
                            element: Box::new(element.clone().into()),
                            length: *length,
                        };
                        if !result.contains(&ty) {
                            result.push(ty);
                        }
                    }
                }
                primer_ir::StatementKind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    visit(then_body, result);
                    visit(else_body, result);
                }
                primer_ir::StatementKind::While { body, .. } => visit(body, result),
                primer_ir::StatementKind::For {
                    initializer,
                    update,
                    body,
                    ..
                } => {
                    visit(std::slice::from_ref(initializer), result);
                    visit(std::slice::from_ref(update), result);
                    visit(body, result);
                }
                primer_ir::StatementKind::Binding { .. }
                | primer_ir::StatementKind::Print { .. }
                | primer_ir::StatementKind::Call { .. }
                | primer_ir::StatementKind::Return { .. }
                | primer_ir::StatementKind::Break
                | primer_ir::StatementKind::Continue => {}
            }
        }
    }

    let mut result = Vec::new();
    for function in &program.function_definitions {
        visit(&function.body, &mut result);
    }
    visit(&program.statements, &mut result);
    result
}

fn lower_type_definitions(program: &primer_ir::Program) -> Vec<TypeDefinition> {
    fn visit(
        id: usize,
        program: &primer_ir::Program,
        visited: &mut [bool],
        definitions: &mut Vec<TypeDefinition>,
    ) {
        if visited[id] {
            return;
        }
        visited[id] = true;
        let definition = &program.type_definitions[id];
        for field in &definition.fields {
            if let Some(dependency) = named_type_dependency(&field.ty) {
                visit(dependency.0, program, visited, definitions);
            }
        }
        definitions.push(TypeDefinition {
            id,
            name: definition.name.clone(),
            fields: definition
                .fields
                .iter()
                .map(|field| FieldDefinition {
                    name: field.name.clone(),
                    ty: field.ty.clone().into(),
                })
                .collect(),
        });
    }

    let mut visited = vec![false; program.type_definitions.len()];
    let mut definitions = Vec::new();
    for id in 0..program.type_definitions.len() {
        visit(id, program, &mut visited, &mut definitions);
    }
    definitions
}

fn named_type_dependency(ty: &primer_ir::Type) -> Option<primer_ir::TypeId> {
    match ty {
        primer_ir::Type::String => None,
        primer_ir::Type::Named(id) => Some(*id),
        primer_ir::Type::Array { element, .. } => named_type_dependency(element),
        primer_ir::Type::Bool
        | primer_ir::Type::Integer(_)
        | primer_ir::Type::F32
        | primer_ir::Type::F64 => None,
    }
}

fn lower_statement(statement: &primer_ir::Statement) -> Statement {
    match &statement.kind {
        primer_ir::StatementKind::Binding {
            id,
            name,
            ty,
            value,
            ..
        } => Statement::Binding {
            name: binding_name(*id, name),
            ty: ty.clone().into(),
            value: lower_expr(value),
        },

        primer_ir::StatementKind::Assignment { target, value } => Statement::Assignment {
            target: AssignmentTarget {
                name: binding_name(target.id, &target.name),
                projections: target
                    .projections
                    .iter()
                    .map(|projection| {
                        let primer_ir::AssignmentProjection::Index {
                            index,
                            element,
                            length,
                            ..
                        } = projection;
                        ArrayProjection {
                            index: lower_expr(index),
                            element: element.clone().into(),
                            length: *length,
                        }
                    })
                    .collect(),
                ty: target.ty.clone().into(),
            },
            value: lower_expr(value),
        },

        primer_ir::StatementKind::Print { value } => Statement::Print {
            format: print_format(&value.ty),
            value: lower_expr(value),
        },

        primer_ir::StatementKind::Call {
            function_id,
            function_name,
            arguments,
        } => Statement::Call {
            evaluation: Vec::new(),
            function_id: function_id.0,
            function_name: function_name.clone(),
            arguments: arguments.iter().map(lower_expr).collect(),
        },

        primer_ir::StatementKind::Return { value } => {
            Statement::Return(value.as_ref().map(lower_expr))
        }

        primer_ir::StatementKind::If {
            condition,
            then_body,
            else_body,
        } => Statement::If {
            condition: lower_expr(condition),
            then_body: then_body.iter().map(lower_statement).collect(),
            else_body: else_body.iter().map(lower_statement).collect(),
        },

        primer_ir::StatementKind::While { condition, body } => Statement::While {
            condition: lower_expr(condition),
            body: body.iter().map(lower_statement).collect(),
        },

        primer_ir::StatementKind::For {
            initializer,
            condition,
            update,
            body,
        } => Statement::For {
            initializer: Box::new(lower_statement(initializer)),
            condition: lower_expr(condition),
            update: Box::new(lower_statement(update)),
            body: body.iter().map(lower_statement).collect(),
        },

        primer_ir::StatementKind::Break => Statement::Break,
        primer_ir::StatementKind::Continue => Statement::Continue,
    }
}

// Cの宣言スコープや補助関数名に影響されず、解決済みの束縛を参照します。
fn binding_name(id: primer_ir::BindingId, name: &str) -> String {
    format!("binding_{}_{name}", id.0)
}

fn lower_expr(expr: &primer_ir::Expr) -> Expr {
    let value = lower_expr_unchecked(expr);
    if let Some(ty) = super::super::integer_range_check(expr) {
        Expr {
            ty: value.ty.clone(),
            kind: ExprKind::CheckIntegerRange {
                value: Box::new(value),
                ty,
            },
        }
    } else {
        value
    }
}

fn lower_expr_unchecked(expr: &primer_ir::Expr) -> Expr {
    let kind = match &expr.kind {
        primer_ir::ExprKind::String(value) => ExprKind::String(value.clone()),
        primer_ir::ExprKind::Logical { op, left, right } => ExprKind::Logical {
            op: match op {
                primer_ir::LogicalOp::And => super::ir::LogicalOp::And,
                primer_ir::LogicalOp::Or => super::ir::LogicalOp::Or,
            },
            left: Box::new(lower_expr(left)),
            right: Box::new(lower_expr(right)),
        },
        primer_ir::ExprKind::ConvertNumeric {
            value, from, to, ..
        } => {
            if from == to {
                return lower_expr(value);
            }
            ExprKind::ConvertNumeric {
                value: Box::new(lower_expr(value)),
                conversion: crate::codegen::NumericConversion {
                    from: *from,
                    to: *to,
                },
            }
        }
        primer_ir::ExprKind::ConvertInteger { value, .. } => return lower_expr(value),
        primer_ir::ExprKind::Boolean(value) => ExprKind::Boolean(*value),

        primer_ir::ExprKind::Integer(value) => ExprKind::Integer(*value),

        primer_ir::ExprKind::Float { text } => ExprKind::Float {
            text: text.clone(),
            suffix_f32: expr.ty == primer_ir::Type::F32,
        },

        primer_ir::ExprKind::Variable { id, name } => ExprKind::Variable(binding_name(*id, name)),

        primer_ir::ExprKind::Unary {
            op: primer_ir::UnaryOp::BitNot,
            value,
        } => ExprKind::IntegerBinary {
            scratch: expr.id.0,
            op: crate::codegen::IntegerBinaryOp::BitXor,
            ty: crate::codegen::integer_type(&expr.ty),
            left: Box::new(lower_expr(value)),
            right: Box::new(Expr {
                ty: Type::I64,
                kind: ExprKind::Integer(crate::codegen::complement_mask(&expr.ty)),
            }),
        },
        primer_ir::ExprKind::Unary { op, value } => ExprKind::Unary {
            op: lower_unary_op(*op, &expr.ty),
            value: Box::new(lower_expr(value)),
        },

        primer_ir::ExprKind::Binary { op, left, right }
            if crate::codegen::integer_binary_op(*op).is_some() =>
        {
            ExprKind::IntegerBinary {
                scratch: expr.id.0,
                op: crate::codegen::integer_binary_op(*op).unwrap(),
                ty: crate::codegen::integer_type(&expr.ty),
                left: Box::new(lower_expr(left)),
                right: Box::new(lower_expr(right)),
            }
        }
        primer_ir::ExprKind::Binary { op, left, right } => ExprKind::Binary {
            op: lower_binary_op(*op, &left.ty),
            left: Box::new(lower_expr(left)),
            right: Box::new(lower_expr(right)),
        },

        primer_ir::ExprKind::Construct {
            type_id, fields, ..
        } => ExprKind::Construct {
            type_id: type_id.0,
            fields: fields
                .iter()
                .map(|field| FieldValue {
                    name: field.name.clone(),
                    value: lower_expr(&field.value),
                })
                .collect(),
        },

        primer_ir::ExprKind::FieldAccess {
            field_name, base, ..
        } => ExprKind::FieldAccess {
            field_name: field_name.clone(),
            base: Box::new(lower_expr(base)),
        },
        primer_ir::ExprKind::Array(values) => {
            ExprKind::Array(values.iter().map(lower_expr).collect())
        }
        primer_ir::ExprKind::Index { base, index } => ExprKind::Index {
            base: Box::new(lower_expr(base)),
            index: Box::new(lower_expr(index)),
        },
        primer_ir::ExprKind::Call {
            function_id,
            function_name,
            arguments,
        } => ExprKind::Call {
            function_id: function_id.0,
            function_name: function_name.clone(),
            arguments: arguments.iter().map(lower_expr).collect(),
        },
    };

    Expr {
        ty: expr.ty.clone().into(),
        kind,
    }
}

fn print_format(ty: &primer_ir::Type) -> PrintFormat {
    match ty {
        primer_ir::Type::String => PrintFormat::String,
        primer_ir::Type::Bool => PrintFormat::Bool,
        primer_ir::Type::Integer(_) => PrintFormat::I64,
        primer_ir::Type::F32 => PrintFormat::F32,
        primer_ir::Type::F64 => PrintFormat::F64,
        primer_ir::Type::Named(_) | primer_ir::Type::Array { .. } => {
            unreachable!("semantic analysis rejects aggregate printing")
        }
    }
}

impl From<primer_ir::Type> for Type {
    fn from(value: primer_ir::Type) -> Self {
        match value {
            primer_ir::Type::String => Self::String,
            primer_ir::Type::Bool => Self::Bool,
            primer_ir::Type::Integer(_) => Self::I64,
            primer_ir::Type::F32 => Self::Float,
            primer_ir::Type::F64 => Self::Double,
            primer_ir::Type::Named(id) => Self::Named(id.0),
            primer_ir::Type::Array { element, length } => Self::Array {
                element: Box::new((*element).into()),
                length,
            },
        }
    }
}

fn collect_array_types(program: &primer_ir::Program) -> Vec<Type> {
    fn add(ty: &primer_ir::Type, types: &mut Vec<Type>) {
        if let primer_ir::Type::Array { element, .. } = ty {
            add(element, types);
            let ty = ty.clone().into();
            if !types.contains(&ty) {
                types.push(ty);
            }
        }
    }

    fn visit_expr(expr: &primer_ir::Expr, types: &mut Vec<Type>) {
        add(&expr.ty, types);
        match &expr.kind {
            primer_ir::ExprKind::String(_) => {}
            primer_ir::ExprKind::Array(values) => {
                for value in values {
                    visit_expr(value, types);
                }
            }
            primer_ir::ExprKind::Index { base, index }
            | primer_ir::ExprKind::Logical {
                left: base,
                right: index,
                ..
            }
            | primer_ir::ExprKind::Binary {
                left: base,
                right: index,
                ..
            } => {
                visit_expr(base, types);
                visit_expr(index, types);
            }
            primer_ir::ExprKind::Construct { fields, .. } => {
                for field in fields {
                    visit_expr(&field.value, types);
                }
            }
            primer_ir::ExprKind::FieldAccess { base, .. }
            | primer_ir::ExprKind::ConvertNumeric { value: base, .. }
            | primer_ir::ExprKind::ConvertInteger { value: base, .. }
            | primer_ir::ExprKind::Unary { value: base, .. } => visit_expr(base, types),
            primer_ir::ExprKind::Call { arguments, .. } => {
                for argument in arguments {
                    visit_expr(argument, types);
                }
            }
            primer_ir::ExprKind::Boolean(_)
            | primer_ir::ExprKind::Integer(_)
            | primer_ir::ExprKind::Float { .. }
            | primer_ir::ExprKind::Variable { .. } => {}
        }
    }

    fn visit_statement(statement: &primer_ir::Statement, types: &mut Vec<Type>) {
        match &statement.kind {
            primer_ir::StatementKind::Binding { ty, value, .. } => {
                add(ty, types);
                visit_expr(value, types);
            }
            primer_ir::StatementKind::Assignment { target, value } => {
                add(&target.root_ty, types);
                for projection in &target.projections {
                    let primer_ir::AssignmentProjection::Index { index, .. } = projection;
                    visit_expr(index, types);
                }
                visit_expr(value, types);
            }
            primer_ir::StatementKind::Print { value }
            | primer_ir::StatementKind::Return { value: Some(value) } => visit_expr(value, types),
            primer_ir::StatementKind::Call { arguments, .. } => {
                for argument in arguments {
                    visit_expr(argument, types);
                }
            }
            primer_ir::StatementKind::If {
                condition,
                then_body,
                else_body,
            } => {
                visit_expr(condition, types);
                for statement in then_body.iter().chain(else_body) {
                    visit_statement(statement, types);
                }
            }
            primer_ir::StatementKind::While { condition, body } => {
                visit_expr(condition, types);
                for statement in body {
                    visit_statement(statement, types);
                }
            }
            primer_ir::StatementKind::For {
                initializer,
                condition,
                update,
                body,
            } => {
                visit_statement(initializer, types);
                visit_expr(condition, types);
                visit_statement(update, types);
                for statement in body {
                    visit_statement(statement, types);
                }
            }
            primer_ir::StatementKind::Return { value: None }
            | primer_ir::StatementKind::Break
            | primer_ir::StatementKind::Continue => {}
        }
    }

    let mut types = Vec::new();
    for definition in &program.type_definitions {
        for field in &definition.fields {
            add(&field.ty, &mut types);
        }
    }
    for function in &program.function_definitions {
        for statement in &function.body {
            visit_statement(statement, &mut types);
        }
    }
    for statement in &program.statements {
        visit_statement(statement, &mut types);
    }
    types
}

fn lower_unary_op(op: primer_ir::UnaryOp, ty: &primer_ir::Type) -> UnaryOp {
    match (op, ty) {
        (primer_ir::UnaryOp::BitNot, _) => unreachable!("bit complement uses integer lowering"),
        (primer_ir::UnaryOp::Negate, primer_ir::Type::Integer(_)) => UnaryOp::CheckedI64Negate,
        (primer_ir::UnaryOp::Negate, _) => UnaryOp::Negate,
        (primer_ir::UnaryOp::Not, _) => UnaryOp::Not,
    }
}

fn lower_binary_op(op: primer_ir::BinaryOp, operand_ty: &primer_ir::Type) -> BinaryOp {
    match (op, operand_ty) {
        (
            primer_ir::BinaryOp::Remainder
            | primer_ir::BinaryOp::BitAnd
            | primer_ir::BinaryOp::BitOr
            | primer_ir::BinaryOp::BitXor
            | primer_ir::BinaryOp::ShiftLeft
            | primer_ir::BinaryOp::ShiftRight,
            _,
        ) => unreachable!("integer operation uses separate lowering"),
        (primer_ir::BinaryOp::Add, primer_ir::Type::Integer(_)) => BinaryOp::CheckedI64Add,
        (primer_ir::BinaryOp::Subtract, primer_ir::Type::Integer(_)) => {
            BinaryOp::CheckedI64Subtract
        }
        (primer_ir::BinaryOp::Multiply, primer_ir::Type::Integer(_)) => {
            BinaryOp::CheckedI64Multiply
        }
        (primer_ir::BinaryOp::Divide, primer_ir::Type::Integer(_)) => BinaryOp::CheckedI64Divide,
        (primer_ir::BinaryOp::Add, _) => BinaryOp::Add,
        (primer_ir::BinaryOp::Subtract, _) => BinaryOp::Subtract,
        (primer_ir::BinaryOp::Multiply, _) => BinaryOp::Multiply,
        (primer_ir::BinaryOp::Divide, _) => BinaryOp::Divide,
        (primer_ir::BinaryOp::Equal, _) => BinaryOp::Equal,
        (primer_ir::BinaryOp::NotEqual, _) => BinaryOp::NotEqual,
        (primer_ir::BinaryOp::Less, _) => BinaryOp::Less,
        (primer_ir::BinaryOp::LessEqual, _) => BinaryOp::LessEqual,
        (primer_ir::BinaryOp::Greater, _) => BinaryOp::Greater,
        (primer_ir::BinaryOp::GreaterEqual, _) => BinaryOp::GreaterEqual,
    }
}
