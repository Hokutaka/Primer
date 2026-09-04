use crate::ir as primer_ir;

use super::ir::{
    BinaryOp, Expr, ExprKind, FieldDefinition, FieldValue, Function, Module, Parameter,
    PrintFormat, Statement, Type, TypeDefinition, UnaryOp,
};

pub fn lower(program: &primer_ir::Program) -> Module {
    Module {
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
                        ty: parameter.ty.into(),
                    })
                    .collect(),
                return_type: match function.return_type {
                    primer_ir::ReturnType::Void => None,
                    primer_ir::ReturnType::Value(ty) => Some(ty.into()),
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
            if let primer_ir::Type::Named(dependency) = field.ty {
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
                    ty: field.ty.into(),
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

fn lower_statement(statement: &primer_ir::Statement) -> Statement {
    match &statement.kind {
        primer_ir::StatementKind::Binding {
            name, ty, value, ..
        } => Statement::Binding {
            name: name.clone(),
            ty: (*ty).into(),
            value: lower_expr(value),
        },

        primer_ir::StatementKind::Assignment { name, value, .. } => Statement::Assignment {
            name: name.clone(),
            value: lower_expr(value),
        },

        primer_ir::StatementKind::Print { value } => Statement::Print {
            format: print_format(value.ty),
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
        ty: expr.ty.into(),
        kind,
    }
}

fn print_format(ty: primer_ir::Type) -> PrintFormat {
    match ty {
        primer_ir::Type::Bool => PrintFormat::Bool,
        primer_ir::Type::I64 => PrintFormat::I64,
        primer_ir::Type::F32 => PrintFormat::F32,
        primer_ir::Type::F64 => PrintFormat::F64,
        primer_ir::Type::Named(_) => unreachable!("semantic analysis rejects aggregate printing"),
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
        }
    }
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
