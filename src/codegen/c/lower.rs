use crate::ir as primer_ir;

use super::ir::{
    ArrayProjection, AssignmentTarget, BinaryOp, Expr, ExprKind, FieldDefinition, FieldValue,
    Function, Module, Parameter, PrintFormat, Statement, Type, TypeDefinition, UnaryOp,
};

pub fn lower(program: &primer_ir::Program) -> Module {
    Module {
        array_types: collect_array_types(program),
        array_assignment_types: collect_array_assignment_types(program),
        type_definitions: lower_type_definitions(program),
        functions: program
            .function_definitions
            .iter()
            .map(|function| Function {
                id: function.id.0,
                name: function.name.clone(),
                parameters: function
                    .parameters
                    .iter()
                    .map(|parameter| Parameter {
                        name: parameter.name.clone(),
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
    }
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
        primer_ir::Type::Named(id) => Some(*id),
        primer_ir::Type::Array { element, .. } => named_type_dependency(element),
        primer_ir::Type::Bool
        | primer_ir::Type::I64
        | primer_ir::Type::F32
        | primer_ir::Type::F64 => None,
    }
}

fn lower_statement(statement: &primer_ir::Statement) -> Statement {
    match &statement.kind {
        primer_ir::StatementKind::Binding {
            name, ty, value, ..
        } => Statement::Binding {
            name: name.clone(),
            ty: ty.clone().into(),
            value: lower_expr(value),
        },

        primer_ir::StatementKind::Assignment { target, value } => Statement::Assignment {
            target: AssignmentTarget {
                name: target.name.clone(),
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

fn lower_expr(expr: &primer_ir::Expr) -> Expr {
    let kind = match &expr.kind {
        primer_ir::ExprKind::Boolean(value) => ExprKind::Boolean(*value),

        primer_ir::ExprKind::Integer(value) => ExprKind::Integer(*value),

        primer_ir::ExprKind::Float { text } => ExprKind::Float {
            text: text.clone(),
            suffix_f32: expr.ty == primer_ir::Type::F32,
        },

        primer_ir::ExprKind::Variable { name, .. } => ExprKind::Variable(name.clone()),

        primer_ir::ExprKind::Unary { op, value } => ExprKind::Unary {
            op: (*op).into(),
            value: Box::new(lower_expr(value)),
        },

        primer_ir::ExprKind::Binary { op, left, right } => ExprKind::Binary {
            op: (*op).into(),
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
        primer_ir::Type::Bool => PrintFormat::Bool,
        primer_ir::Type::I64 => PrintFormat::I64,
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
            primer_ir::Type::Bool => Self::Bool,
            primer_ir::Type::I64 => Self::I64,
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
            primer_ir::ExprKind::Array(values) => {
                for value in values {
                    visit_expr(value, types);
                }
            }
            primer_ir::ExprKind::Index { base, index }
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

impl From<primer_ir::UnaryOp> for UnaryOp {
    fn from(value: primer_ir::UnaryOp) -> Self {
        match value {
            primer_ir::UnaryOp::Negate => Self::Negate,
            primer_ir::UnaryOp::Not => Self::Not,
        }
    }
}

impl From<primer_ir::BinaryOp> for BinaryOp {
    fn from(value: primer_ir::BinaryOp) -> Self {
        match value {
            primer_ir::BinaryOp::Add => Self::Add,
            primer_ir::BinaryOp::Subtract => Self::Subtract,
            primer_ir::BinaryOp::Multiply => Self::Multiply,
            primer_ir::BinaryOp::Divide => Self::Divide,
            primer_ir::BinaryOp::Equal => Self::Equal,
            primer_ir::BinaryOp::NotEqual => Self::NotEqual,
            primer_ir::BinaryOp::Less => Self::Less,
            primer_ir::BinaryOp::LessEqual => Self::LessEqual,
            primer_ir::BinaryOp::Greater => Self::Greater,
            primer_ir::BinaryOp::GreaterEqual => Self::GreaterEqual,
        }
    }
}
