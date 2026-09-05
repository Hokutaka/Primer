use std::collections::HashMap;

use crate::{
    ast::{
        self, AssignmentProjection, BinaryOp, Expr, ExprKind, Item, Program, Stmt, StmtKind,
        TypeSpec,
    },
    diagnostic::Diagnostic,
    source::Span,
    types::IntegerType,
};

pub type Bindings = HashMap<String, BindingInfo>;
type SemanticResult<T> = Result<T, Diagnostic>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Bool,
    Integer(IntegerType),
    F32,
    F64,
    Named(TypeId),
    Array { element: Box<Type>, length: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingInfo {
    pub ty: Type,
    pub mutable: bool,
}

#[derive(Debug, Clone)]
pub struct FieldDefinition {
    pub id: FieldId,
    pub name: String,
    pub name_span: Span,
    pub ty: Type,
    pub type_span: Span,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub id: TypeId,
    pub name: String,
    pub name_span: Span,
    pub fields: Vec<FieldDefinition>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnType {
    Void,
    Value(Type),
}

#[derive(Debug, Clone)]
pub struct ParameterDefinition {
    pub name: String,
    pub name_span: Span,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub id: FunctionId,
    pub name: String,
    pub name_span: Span,
    pub parameters: Vec<ParameterDefinition>,
    pub return_type: ReturnType,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SemanticModel {
    pub bindings: Bindings,
    pub type_definitions: Vec<TypeDefinition>,
    pub function_definitions: Vec<FunctionDefinition>,
    type_names: HashMap<String, TypeId>,
    function_names: HashMap<String, FunctionId>,
}

impl SemanticModel {
    pub fn resolve_type_ref(&self, type_ref: &ast::TypeRef) -> SemanticResult<Type> {
        resolve_type_ref(type_ref, &self.type_names)
    }

    pub fn type_definition(&self, id: TypeId) -> &TypeDefinition {
        &self.type_definitions[id.0]
    }

    pub fn function_definition(&self, id: FunctionId) -> &FunctionDefinition {
        &self.function_definitions[id.0]
    }

    pub fn resolve_function_name(&self, name: &str, span: Span) -> SemanticResult<FunctionId> {
        self.function_names
            .get(name)
            .copied()
            .ok_or_else(|| Diagnostic::new(format!("unknown function `{name}`"), span))
    }

    pub fn resolve_type_name(&self, name: &str, span: Span) -> SemanticResult<TypeId> {
        self.type_names
            .get(name)
            .copied()
            .ok_or_else(|| Diagnostic::new(format!("unknown type `{name}`"), span))
    }

    pub fn type_of_expr(&self, expr: &Expr, bindings: &Bindings) -> SemanticResult<Type> {
        type_of_expr_expected(expr, bindings, None, self)
    }

    pub(crate) fn type_of_expr_expected(
        &self,
        expr: &Expr,
        bindings: &Bindings,
        expected: Option<Type>,
    ) -> SemanticResult<Type> {
        type_of_expr_expected(expr, bindings, expected, self)
    }

    pub fn type_name(&self, ty: Type) -> String {
        match ty {
            Type::Bool => "bool".into(),
            Type::Integer(integer) => integer.name().into(),
            Type::F32 => "f32".into(),
            Type::F64 => "f64".into(),
            Type::Named(id) => self.type_definition(id).name.clone(),
            Type::Array { element, length } => {
                format!("[{}; {length}]", self.type_name(*element))
            }
        }
    }
}

pub fn check(program: &Program) -> SemanticResult<Bindings> {
    Ok(analyze(program)?.bindings)
}

pub fn analyze(program: &Program) -> SemanticResult<SemanticModel> {
    let type_names = register_type_names(program)?;
    let type_definitions = resolve_type_definitions(program, &type_names)?;
    let function_names = register_function_names(program)?;
    let function_definitions = resolve_function_definitions(program, &type_names, &function_names)?;
    let mut model = SemanticModel {
        bindings: HashMap::new(),
        type_definitions,
        function_definitions,
        type_names,
        function_names,
    };

    reject_infinite_types(&model)?;
    check_defaults(&model)?;

    let has_main = model.function_names.contains_key("main");
    let has_top_level_statements = program
        .items
        .iter()
        .any(|item| matches!(item, Item::Statement(_)));
    if has_main && has_top_level_statements {
        let main = model.function_definition(model.function_names["main"]);
        return Err(Diagnostic::new(
            "an explicit `main` function cannot be combined with top-level statements",
            main.name_span,
        ));
    }

    let mut scopes = vec![HashMap::new()];
    for item in &program.items {
        match item {
            Item::TypeDefinition(_) => {}
            Item::FunctionDefinition(function) => {
                check_function(function, &model)?;
            }
            Item::Statement(statement) => {
                check_statements(
                    std::slice::from_ref(statement),
                    &mut scopes,
                    0,
                    None,
                    &model,
                )?;
            }
        }
    }
    reject_recursive_functions(program, &model)?;
    model.bindings = scopes.pop().expect("top-level scope must exist");
    Ok(model)
}

fn register_function_names(program: &Program) -> SemanticResult<HashMap<String, FunctionId>> {
    let mut names = HashMap::new();
    for item in &program.items {
        let Item::FunctionDefinition(definition) = item else {
            continue;
        };
        let id = FunctionId(names.len());
        if names.insert(definition.name.clone(), id).is_some() {
            return Err(Diagnostic::new(
                format!("duplicate function `{}`", definition.name),
                definition.name_span,
            ));
        }
    }
    Ok(names)
}

fn resolve_function_definitions(
    program: &Program,
    type_names: &HashMap<String, TypeId>,
    function_names: &HashMap<String, FunctionId>,
) -> SemanticResult<Vec<FunctionDefinition>> {
    let mut definitions = Vec::new();
    for item in &program.items {
        let Item::FunctionDefinition(definition) = item else {
            continue;
        };
        if definition.parameters.len() > 4 {
            return Err(Diagnostic::new(
                "functions currently support at most four parameters",
                definition.span,
            ));
        }
        let mut parameter_names = HashMap::new();
        let mut parameters = Vec::new();
        for parameter in &definition.parameters {
            if parameter_names.insert(parameter.name.clone(), ()).is_some() {
                return Err(Diagnostic::new(
                    format!("duplicate parameter `{}`", parameter.name),
                    parameter.name_span,
                ));
            }
            let ty = resolve_type_ref(&parameter.type_ref, type_names)?;
            parameters.push(ParameterDefinition {
                name: parameter.name.clone(),
                name_span: parameter.name_span,
                ty,
                span: parameter.span,
            });
        }
        let return_type = match &definition.return_type {
            ast::ReturnTypeRef::Void(_) => ReturnType::Void,
            ast::ReturnTypeRef::Value(type_ref) => {
                let ty = resolve_type_ref(type_ref, type_names)?;
                ReturnType::Value(ty)
            }
        };
        let id = function_names[&definition.name];
        if definition.name == "main" && (!parameters.is_empty() || return_type != ReturnType::Void)
        {
            return Err(Diagnostic::new(
                "`main` must have no parameters and return void",
                definition.span,
            ));
        }
        definitions.push(FunctionDefinition {
            id,
            name: definition.name.clone(),
            name_span: definition.name_span,
            parameters,
            return_type,
            span: definition.span,
        });
    }
    Ok(definitions)
}

fn check_function(function: &ast::FunctionDefinition, model: &SemanticModel) -> SemanticResult<()> {
    let definition = model.function_definition(model.function_names[&function.name]);
    let mut parameter_scope = HashMap::new();
    for parameter in &definition.parameters {
        parameter_scope.insert(
            parameter.name.clone(),
            BindingInfo {
                ty: parameter.ty.clone(),
                mutable: false,
            },
        );
    }
    let mut scopes = vec![parameter_scope];
    check_statements(
        &function.body,
        &mut scopes,
        0,
        Some(&definition.return_type),
        model,
    )?;
    if matches!(definition.return_type, ReturnType::Value(_))
        && !statements_guarantee_return(&function.body)
    {
        return Err(Diagnostic::new(
            format!(
                "function `{}` may finish without returning a value",
                function.name
            ),
            function.span,
        ));
    }
    Ok(())
}

fn reject_recursive_functions(program: &Program, model: &SemanticModel) -> SemanticResult<()> {
    let mut calls = vec![Vec::new(); model.function_definitions.len()];
    for item in &program.items {
        let Item::FunctionDefinition(function) = item else {
            continue;
        };
        let function_id = model.function_names[&function.name];
        collect_function_calls(&function.body, model, &mut calls[function_id.0]);
    }

    // 0は未訪問、1は訪問中、2は確認済みです。
    let mut states = vec![0; calls.len()];
    for function_id in 0..calls.len() {
        if states[function_id] == 0
            && let Some(span) = find_recursive_call(function_id, &calls, &mut states)
        {
            return Err(Diagnostic::new(
                "recursive function calls are not supported yet",
                span,
            ));
        }
    }
    Ok(())
}

fn find_recursive_call(
    function_id: usize,
    calls: &[Vec<(FunctionId, Span)>],
    states: &mut [u8],
) -> Option<Span> {
    states[function_id] = 1;
    for &(called, span) in &calls[function_id] {
        match states[called.0] {
            1 => return Some(span),
            0 => {
                if let Some(span) = find_recursive_call(called.0, calls, states) {
                    return Some(span);
                }
            }
            _ => {}
        }
    }
    states[function_id] = 2;
    None
}

fn collect_function_calls(
    statements: &[Stmt],
    model: &SemanticModel,
    calls: &mut Vec<(FunctionId, Span)>,
) {
    for statement in statements {
        match &statement.kind {
            StmtKind::Binding { value, .. }
            | StmtKind::Print { value }
            | StmtKind::Call { value } => collect_calls_in_expr(value, model, calls),
            StmtKind::Assignment { target, value } => {
                for projection in &target.projections {
                    let AssignmentProjection::Index { index, .. } = projection;
                    collect_calls_in_expr(index, model, calls);
                }
                collect_calls_in_expr(value, model, calls);
            }
            StmtKind::Return { value } => {
                if let Some(value) = value {
                    collect_calls_in_expr(value, model, calls);
                }
            }
            StmtKind::If {
                condition,
                then_body,
                else_body,
            } => {
                collect_calls_in_expr(condition, model, calls);
                collect_function_calls(then_body, model, calls);
                collect_function_calls(else_body, model, calls);
            }
            StmtKind::While { condition, body } => {
                collect_calls_in_expr(condition, model, calls);
                collect_function_calls(body, model, calls);
            }
            StmtKind::For {
                initializer,
                condition,
                update,
                body,
            } => {
                collect_function_calls(std::slice::from_ref(initializer), model, calls);
                collect_calls_in_expr(condition, model, calls);
                collect_function_calls(std::slice::from_ref(update), model, calls);
                collect_function_calls(body, model, calls);
            }
            StmtKind::Break | StmtKind::Continue => {}
        }
    }
}

fn collect_calls_in_expr(expr: &Expr, model: &SemanticModel, calls: &mut Vec<(FunctionId, Span)>) {
    match &expr.kind {
        ExprKind::Call {
            name,
            name_span,
            arguments,
        } => {
            if let Some(id) = model.function_names.get(name) {
                calls.push((*id, *name_span));
            }
            for argument in arguments {
                collect_calls_in_expr(argument, model, calls);
            }
        }
        ExprKind::Construct { fields, .. } => {
            for field in fields {
                collect_calls_in_expr(&field.value, model, calls);
            }
        }
        ExprKind::Array(values) => {
            for value in values {
                collect_calls_in_expr(value, model, calls);
            }
        }
        ExprKind::Index { base, index } => {
            collect_calls_in_expr(base, model, calls);
            collect_calls_in_expr(index, model, calls);
        }
        ExprKind::FieldAccess { base, .. } | ExprKind::Unary { value: base, .. } => {
            collect_calls_in_expr(base, model, calls);
        }
        ExprKind::Binary { left, right, .. } => {
            collect_calls_in_expr(left, model, calls);
            collect_calls_in_expr(right, model, calls);
        }
        ExprKind::Boolean(_)
        | ExprKind::Integer(_)
        | ExprKind::Float { .. }
        | ExprKind::Variable(_) => {}
    }
}

fn statements_guarantee_return(statements: &[Stmt]) -> bool {
    statements.iter().any(|statement| match &statement.kind {
        StmtKind::Return { .. } => true,
        StmtKind::If {
            then_body,
            else_body,
            ..
        } => {
            !else_body.is_empty()
                && statements_guarantee_return(then_body)
                && statements_guarantee_return(else_body)
        }
        _ => false,
    })
}

fn register_type_names(program: &Program) -> SemanticResult<HashMap<String, TypeId>> {
    let mut names = HashMap::new();

    for item in &program.items {
        let Item::TypeDefinition(definition) = item else {
            continue;
        };
        let id = TypeId(names.len());

        if names.insert(definition.name.clone(), id).is_some() {
            return Err(Diagnostic::new(
                format!("duplicate type `{}`", definition.name),
                definition.name_span,
            ));
        }
    }

    Ok(names)
}

fn resolve_type_definitions(
    program: &Program,
    type_names: &HashMap<String, TypeId>,
) -> SemanticResult<Vec<TypeDefinition>> {
    let mut definitions = Vec::new();

    for item in &program.items {
        let Item::TypeDefinition(definition) = item else {
            continue;
        };
        let id = *type_names
            .get(&definition.name)
            .expect("registered type must have an id");
        let mut field_names = HashMap::new();
        let mut fields = Vec::new();

        for field in &definition.fields {
            let field_id = FieldId(fields.len());
            if field_names.insert(field.name.clone(), field_id).is_some() {
                return Err(Diagnostic::new(
                    format!(
                        "duplicate field `{}` in type `{}`",
                        field.name, definition.name
                    ),
                    field.name_span,
                ));
            }

            fields.push(FieldDefinition {
                id: field_id,
                name: field.name.clone(),
                name_span: field.name_span,
                ty: resolve_type_ref(&field.type_ref, type_names)?,
                type_span: field.type_ref.span,
                default: field.default.clone(),
                span: field.span,
            });
        }

        definitions.push(TypeDefinition {
            id,
            name: definition.name.clone(),
            name_span: definition.name_span,
            fields,
            span: definition.span,
        });
    }

    definitions.sort_by_key(|definition| definition.id.0);
    Ok(definitions)
}

fn resolve_type_name(
    name: &str,
    span: Span,
    type_names: &HashMap<String, TypeId>,
) -> SemanticResult<Type> {
    match name {
        "bool" => Ok(Type::Bool),
        "i64" => Ok(Type::Integer(IntegerType::I64)),
        "f32" => Ok(Type::F32),
        "f64" => Ok(Type::F64),
        _ => type_names
            .get(name)
            .copied()
            .map(Type::Named)
            .ok_or_else(|| Diagnostic::new(format!("unknown type `{name}`"), span)),
    }
}

fn resolve_type_ref(
    type_ref: &ast::TypeRef,
    type_names: &HashMap<String, TypeId>,
) -> SemanticResult<Type> {
    let (element, length) = match &type_ref.kind {
        ast::TypeRefKind::Named(name) => {
            return resolve_type_name(name, type_ref.span, type_names);
        }
        ast::TypeRefKind::Array { element, length } => (element, *length),
    };
    let element = resolve_type_ref(element, type_names)?;
    Ok(Type::Array {
        element: Box::new(element),
        length,
    })
}

fn reject_infinite_types(model: &SemanticModel) -> SemanticResult<()> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Visit {
        Unvisited,
        Visiting,
        Done,
    }

    fn visit(id: TypeId, model: &SemanticModel, states: &mut [Visit]) -> SemanticResult<()> {
        states[id.0] = Visit::Visiting;

        for field in &model.type_definition(id).fields {
            let Some(next) = named_type_dependency(&field.ty) else {
                continue;
            };

            match states[next.0] {
                Visit::Visiting => {
                    return Err(Diagnostic::new(
                        format!(
                            "type `{}` has infinite size through field `{}`",
                            model.type_definition(id).name,
                            field.name
                        ),
                        field.type_span,
                    ));
                }
                Visit::Unvisited => visit(next, model, states)?,
                Visit::Done => {}
            }
        }

        states[id.0] = Visit::Done;
        Ok(())
    }

    fn named_type_dependency(ty: &Type) -> Option<TypeId> {
        match ty {
            Type::Named(id) => Some(*id),
            Type::Array { element, .. } => named_type_dependency(element),
            Type::Bool | Type::Integer(IntegerType::I64) | Type::F32 | Type::F64 => None,
        }
    }

    let mut states = vec![Visit::Unvisited; model.type_definitions.len()];
    for index in 0..states.len() {
        if states[index] == Visit::Unvisited {
            visit(TypeId(index), model, &mut states)?;
        }
    }
    Ok(())
}

fn check_defaults(model: &SemanticModel) -> SemanticResult<()> {
    let bindings = HashMap::new();

    for definition in &model.type_definitions {
        for field in &definition.fields {
            let Some(default) = &field.default else {
                continue;
            };
            let actual = model.type_of_expr_expected(default, &bindings, Some(field.ty.clone()))?;
            if actual != field.ty {
                return Err(Diagnostic::new(
                    format!(
                        "default for field `{}` expects {}, found {}",
                        field.name,
                        model.type_name(field.ty.clone()),
                        model.type_name(actual)
                    ),
                    default.span,
                ));
            }
        }
    }
    Ok(())
}

fn check_statements(
    statements: &[Stmt],
    scopes: &mut Vec<Bindings>,
    loop_depth: usize,
    return_type: Option<&ReturnType>,
    model: &SemanticModel,
) -> SemanticResult<()> {
    for statement in statements {
        let bindings = visible_bindings(scopes);

        match &statement.kind {
            StmtKind::Binding {
                mutable,
                name,
                type_spec,
                value,
            } => {
                if scopes
                    .last()
                    .expect("current scope must exist")
                    .contains_key(name)
                {
                    return Err(Diagnostic::new(
                        format!("duplicate binding `{name}`"),
                        statement.span,
                    ));
                }

                let value_type = match type_spec {
                    TypeSpec::Explicit(expected) => {
                        let expected = model.resolve_type_ref(expected)?;
                        // 左辺の明示型を右辺へ渡す
                        let actual = model.type_of_expr_expected(
                            value,
                            &bindings,
                            Some(expected.clone()),
                        )?;

                        if actual != expected {
                            return Err(Diagnostic::new(
                                format!(
                                    "type mismatch for `{name}`: expected {}, found {}",
                                    model.type_name(expected),
                                    model.type_name(actual),
                                ),
                                value.span,
                            ));
                        }

                        expected
                    }

                    TypeSpec::Infer => {
                        // inferの場合は文脈なしで推論
                        model.type_of_expr(value, &bindings)?
                    }
                };

                scopes.last_mut().expect("current scope must exist").insert(
                    name.clone(),
                    BindingInfo {
                        ty: value_type,
                        mutable: *mutable,
                    },
                );
            }

            StmtKind::Assignment { target, value } => {
                let binding = bindings.get(&target.name).cloned().ok_or_else(|| {
                    Diagnostic::new(
                        format!("unknown binding `{}`", target.name),
                        target.name_span,
                    )
                })?;

                if !binding.mutable {
                    return Err(Diagnostic::new(
                        format!("cannot assign to immutable binding `{}`", target.name),
                        target.name_span,
                    ));
                }

                let mut target_ty = binding.ty;
                for projection in &target.projections {
                    let AssignmentProjection::Index { index, span } = projection;
                    let Type::Array { element, .. } = target_ty else {
                        return Err(Diagnostic::new(
                            format!(
                                "cannot index assignment target of type {}",
                                model.type_name(target_ty)
                            ),
                            *span,
                        ));
                    };

                    let index_ty = model.type_of_expr_expected(
                        index,
                        &bindings,
                        Some(Type::Integer(IntegerType::I64)),
                    )?;
                    if index_ty != Type::Integer(IntegerType::I64) {
                        return Err(Diagnostic::new(
                            format!(
                                "array index must be i64, found {}",
                                model.type_name(index_ty)
                            ),
                            index.span,
                        ));
                    }
                    target_ty = *element;
                }

                let actual =
                    model.type_of_expr_expected(value, &bindings, Some(target_ty.clone()))?;

                if actual != target_ty {
                    return Err(Diagnostic::new(
                        format!(
                            "type mismatch for assignment to `{}`: expected {}, found {}",
                            target.name,
                            model.type_name(target_ty),
                            model.type_name(actual),
                        ),
                        value.span,
                    ));
                }
            }

            StmtKind::Print { value } => {
                let ty = model.type_of_expr(value, &bindings)?;
                if matches!(ty, Type::Named(_) | Type::Array { .. }) {
                    return Err(Diagnostic::new(
                        format!("cannot print value of type {}", model.type_name(ty)),
                        value.span,
                    ));
                }
            }

            StmtKind::Call { value } => {
                let ExprKind::Call {
                    name,
                    name_span,
                    arguments,
                } = &value.kind
                else {
                    unreachable!("parser only creates call statements from calls")
                };
                if let ReturnType::Value(ty) =
                    check_call(name, *name_span, arguments, &bindings, model)?
                {
                    return Err(Diagnostic::new(
                        format!(
                            "the {} result of function `{name}` must be used",
                            model.type_name(ty)
                        ),
                        value.span,
                    ));
                }
            }

            StmtKind::Return { value } => {
                let Some(expected) = return_type else {
                    return Err(Diagnostic::new(
                        "return can only be used inside a function",
                        statement.span,
                    ));
                };
                match (expected, value) {
                    (ReturnType::Void, None) => {}
                    (ReturnType::Void, Some(value)) => {
                        return Err(Diagnostic::new(
                            "a void function cannot return a value",
                            value.span,
                        ));
                    }
                    (ReturnType::Value(expected), Some(value)) => {
                        let actual = model.type_of_expr_expected(
                            value,
                            &bindings,
                            Some(expected.clone()),
                        )?;
                        if actual != *expected {
                            return Err(Diagnostic::new(
                                format!(
                                    "return expects {}, found {}",
                                    model.type_name(expected.clone()),
                                    model.type_name(actual)
                                ),
                                value.span,
                            ));
                        }
                    }
                    (ReturnType::Value(expected), None) => {
                        return Err(Diagnostic::new(
                            format!(
                                "return requires a {} value",
                                model.type_name(expected.clone())
                            ),
                            statement.span,
                        ));
                    }
                }
            }

            StmtKind::If {
                condition,
                then_body,
                else_body,
            } => {
                let condition_ty =
                    model.type_of_expr_expected(condition, &bindings, Some(Type::Bool))?;

                if condition_ty != Type::Bool {
                    return Err(Diagnostic::new(
                        format!(
                            "if condition must be bool, found {}",
                            model.type_name(condition_ty)
                        ),
                        condition.span,
                    ));
                }

                scopes.push(HashMap::new());
                check_statements(then_body, scopes, loop_depth, return_type, model)?;
                scopes.pop();

                scopes.push(HashMap::new());
                check_statements(else_body, scopes, loop_depth, return_type, model)?;
                scopes.pop();
            }

            StmtKind::While { condition, body } => {
                let condition_ty =
                    model.type_of_expr_expected(condition, &bindings, Some(Type::Bool))?;

                if condition_ty != Type::Bool {
                    return Err(Diagnostic::new(
                        format!(
                            "while condition must be bool, found {}",
                            model.type_name(condition_ty)
                        ),
                        condition.span,
                    ));
                }

                scopes.push(HashMap::new());
                check_statements(body, scopes, loop_depth + 1, return_type, model)?;
                scopes.pop();
            }

            StmtKind::For {
                initializer,
                condition,
                update,
                body,
            } => {
                scopes.push(HashMap::new());
                check_statements(
                    std::slice::from_ref(initializer),
                    scopes,
                    loop_depth,
                    return_type,
                    model,
                )?;

                let bindings = visible_bindings(scopes);
                let condition_ty =
                    model.type_of_expr_expected(condition, &bindings, Some(Type::Bool))?;

                if condition_ty != Type::Bool {
                    return Err(Diagnostic::new(
                        format!(
                            "for condition must be bool, found {}",
                            model.type_name(condition_ty)
                        ),
                        condition.span,
                    ));
                }

                check_statements(
                    std::slice::from_ref(update),
                    scopes,
                    loop_depth,
                    return_type,
                    model,
                )?;

                scopes.push(HashMap::new());
                check_statements(body, scopes, loop_depth + 1, return_type, model)?;
                scopes.pop();
                scopes.pop();
            }

            StmtKind::Break => {
                if loop_depth == 0 {
                    return Err(Diagnostic::new(
                        "break can only be used inside a loop",
                        statement.span,
                    ));
                }
            }

            StmtKind::Continue => {
                if loop_depth == 0 {
                    return Err(Diagnostic::new(
                        "continue can only be used inside a loop",
                        statement.span,
                    ));
                }
            }
        }
    }

    Ok(())
}

fn visible_bindings(scopes: &[Bindings]) -> Bindings {
    let mut visible = HashMap::new();

    for scope in scopes {
        visible.extend(
            scope
                .iter()
                .map(|(name, binding)| (name.clone(), binding.clone())),
        );
    }

    visible
}

fn type_of_expr_expected(
    expr: &Expr,
    bindings: &Bindings,
    expected: Option<Type>,
    model: &SemanticModel,
) -> SemanticResult<Type> {
    match &expr.kind {
        ExprKind::Boolean(_) => Ok(Type::Bool),

        ExprKind::Integer(literal) => {
            resolve_i64_literal(literal, expr.span)?;
            Ok(Type::Integer(IntegerType::I64))
        }

        ExprKind::Float { explicit_type, .. } => {
            // suffix付きなら絶対その型
            if let Some(ty) = explicit_type {
                return Ok(scalar_type(*ty));
            }

            // suffixなしなら文脈を見る
            match expected {
                Some(Type::F32) => Ok(Type::F32),
                Some(Type::F64) => Ok(Type::F64),

                // 文脈なしのfloatはf64
                _ => Ok(Type::F64),
            }
        }

        ExprKind::Variable(name) => bindings
            .get(name)
            .map(|binding| binding.ty.clone())
            .ok_or_else(|| Diagnostic::new(format!("unknown binding `{name}`"), expr.span)),

        ExprKind::Array(values) => {
            let (element, length) = match expected {
                Some(Type::Array { element, length }) => {
                    if values.len() != length {
                        return Err(Diagnostic::new(
                            format!(
                                "array length mismatch: expected {length} values, found {}",
                                values.len()
                            ),
                            expr.span,
                        ));
                    }
                    (element, length)
                }
                _ => {
                    let first = values.first().expect("parser rejects empty array literals");
                    let first_ty = model.type_of_expr(first, bindings)?;
                    (Box::new(first_ty), values.len())
                }
            };
            let expected_element = *element;
            for value in values {
                let actual =
                    model.type_of_expr_expected(value, bindings, Some(expected_element.clone()))?;
                if actual != expected_element {
                    return Err(Diagnostic::new(
                        format!(
                            "array element expects {}, found {}",
                            model.type_name(expected_element.clone()),
                            model.type_name(actual)
                        ),
                        value.span,
                    ));
                }
            }
            Ok(Type::Array {
                element: Box::new(expected_element),
                length,
            })
        }

        ExprKind::Index { base, index } => {
            let base_ty = model.type_of_expr(base, bindings)?;
            let Type::Array { element, .. } = base_ty else {
                return Err(Diagnostic::new(
                    format!("cannot index value of type {}", model.type_name(base_ty)),
                    base.span,
                ));
            };
            let index_ty = model.type_of_expr_expected(
                index,
                bindings,
                Some(Type::Integer(IntegerType::I64)),
            )?;
            if index_ty != Type::Integer(IntegerType::I64) {
                return Err(Diagnostic::new(
                    format!(
                        "array index must be i64, found {}",
                        model.type_name(index_ty)
                    ),
                    index.span,
                ));
            }
            Ok(*element)
        }

        ExprKind::Construct {
            type_name,
            type_name_span,
            fields,
        } => {
            let type_id = model.resolve_type_name(type_name, *type_name_span)?;
            let definition = model.type_definition(type_id);
            let mut supplied = vec![false; definition.fields.len()];

            for field_value in fields {
                let field = definition
                    .fields
                    .iter()
                    .find(|field| field.name == field_value.name)
                    .ok_or_else(|| {
                        Diagnostic::new(
                            format!("type `{type_name}` has no field `{}`", field_value.name),
                            field_value.name_span,
                        )
                    })?;

                if supplied[field.id.0] {
                    return Err(Diagnostic::new(
                        format!("field `{}` is specified more than once", field.name),
                        field_value.name_span,
                    ));
                }

                let actual = model.type_of_expr_expected(
                    &field_value.value,
                    bindings,
                    Some(field.ty.clone()),
                )?;
                if actual != field.ty {
                    return Err(Diagnostic::new(
                        format!(
                            "field `{}` expects {}, found {}",
                            field.name,
                            model.type_name(field.ty.clone()),
                            model.type_name(actual)
                        ),
                        field_value.value.span,
                    ));
                }
                supplied[field.id.0] = true;
            }

            for field in &definition.fields {
                if !supplied[field.id.0] && field.default.is_none() {
                    return Err(Diagnostic::new(
                        format!("missing field `{}` for type `{type_name}`", field.name),
                        expr.span,
                    ));
                }
            }

            Ok(Type::Named(type_id))
        }

        ExprKind::FieldAccess {
            base,
            field_name,
            field_name_span,
        } => {
            let base_type = model.type_of_expr(base, bindings)?;
            let Type::Named(type_id) = base_type else {
                return Err(Diagnostic::new(
                    format!(
                        "cannot access field `{field_name}` on {}",
                        model.type_name(base_type)
                    ),
                    *field_name_span,
                ));
            };
            let definition = model.type_definition(type_id);
            definition
                .fields
                .iter()
                .find(|field| field.name == *field_name)
                .map(|field| field.ty.clone())
                .ok_or_else(|| {
                    Diagnostic::new(
                        format!("type `{}` has no field `{field_name}`", definition.name),
                        *field_name_span,
                    )
                })
        }

        ExprKind::Call {
            name,
            name_span,
            arguments,
        } => match check_call(name, *name_span, arguments, bindings, model)? {
            ReturnType::Value(ty) => Ok(ty),
            ReturnType::Void => Err(Diagnostic::new(
                format!("void function `{name}` does not produce a value"),
                expr.span,
            )),
        },

        ExprKind::Unary { op, value } => match op {
            crate::ast::UnaryOp::Negate => {
                if let ExprKind::Integer(literal) = &value.kind {
                    resolve_negated_i64_literal(literal, expr.span)?;
                    return Ok(Type::Integer(IntegerType::I64));
                }

                let ty = type_of_expr_expected(value, bindings, expected, model)?;

                if !is_numeric(&ty) {
                    return Err(Diagnostic::new(
                        format!("cannot apply `-` to {}", model.type_name(ty)),
                        expr.span,
                    ));
                }

                Ok(ty)
            }

            crate::ast::UnaryOp::Not => {
                let ty = type_of_expr_expected(value, bindings, Some(Type::Bool), model)?;

                if ty != Type::Bool {
                    return Err(Diagnostic::new(
                        format!("cannot apply `!` to {}", model.type_name(ty)),
                        expr.span,
                    ));
                }

                Ok(Type::Bool)
            }
        },

        ExprKind::Binary { op, left, right } => {
            let (left_type, right_type) = if is_comparison(*op) {
                comparison_operand_types(left, right, bindings, model)?
            } else {
                // 親から来た期待型を左右両方へ伝える
                (
                    type_of_expr_expected(left, bindings, expected.clone(), model)?,
                    type_of_expr_expected(right, bindings, expected, model)?,
                )
            };

            if left_type != right_type {
                return Err(Diagnostic::new(
                    format!(
                        "cannot apply `{}` to {} and {}",
                        operator_name(*op),
                        model.type_name(left_type),
                        model.type_name(right_type),
                    ),
                    expr.span,
                ));
            }

            if matches!(left_type, Type::Named(_) | Type::Array { .. }) {
                return Err(Diagnostic::new(
                    format!(
                        "cannot apply `{}` to {}",
                        operator_name(*op),
                        model.type_name(left_type)
                    ),
                    expr.span,
                ));
            }

            if is_arithmetic(*op) {
                if !is_numeric(&left_type) {
                    return Err(Diagnostic::new(
                        format!(
                            "cannot apply `{}` to {}",
                            operator_name(*op),
                            model.type_name(left_type),
                        ),
                        expr.span,
                    ));
                }

                Ok(left_type)
            } else if is_ordering(*op) {
                if !is_numeric(&left_type) {
                    return Err(Diagnostic::new(
                        format!(
                            "cannot apply `{}` to {}",
                            operator_name(*op),
                            model.type_name(left_type),
                        ),
                        expr.span,
                    ));
                }

                Ok(Type::Bool)
            } else {
                Ok(Type::Bool)
            }
        }
    }
}

pub(crate) fn resolve_i64_literal(
    literal: &ast::IntegerLiteral,
    span: Span,
) -> SemanticResult<i64> {
    resolve_i64_magnitude(literal, false, span)
}

pub(crate) fn resolve_negated_i64_literal(
    literal: &ast::IntegerLiteral,
    span: Span,
) -> SemanticResult<i64> {
    resolve_i64_magnitude(literal, true, span)
}

fn resolve_i64_magnitude(
    literal: &ast::IntegerLiteral,
    negative: bool,
    span: Span,
) -> SemanticResult<i64> {
    let magnitude = literal
        .digits()
        .parse::<u64>()
        .map_err(|_| Diagnostic::new("integer literal does not fit in i64", span))?;
    let minimum_magnitude = i64::MAX as u64 + 1;

    if negative {
        if magnitude > minimum_magnitude {
            return Err(Diagnostic::new("integer literal does not fit in i64", span));
        }
        if magnitude == minimum_magnitude {
            return Ok(i64::MIN);
        }
        return Ok(-(magnitude as i64));
    }

    i64::try_from(magnitude)
        .map_err(|_| Diagnostic::new("integer literal does not fit in i64", span))
}

fn check_call(
    name: &str,
    name_span: Span,
    arguments: &[Expr],
    bindings: &Bindings,
    model: &SemanticModel,
) -> SemanticResult<ReturnType> {
    let id = model.resolve_function_name(name, name_span)?;
    let function = model.function_definition(id);
    if arguments.len() != function.parameters.len() {
        return Err(Diagnostic::new(
            format!(
                "function `{name}` expects {} arguments, found {}",
                function.parameters.len(),
                arguments.len()
            ),
            name_span,
        ));
    }
    for (argument, parameter) in arguments.iter().zip(&function.parameters) {
        let actual = model.type_of_expr_expected(argument, bindings, Some(parameter.ty.clone()))?;
        if actual != parameter.ty {
            return Err(Diagnostic::new(
                format!(
                    "argument `{}` expects {}, found {}",
                    parameter.name,
                    model.type_name(parameter.ty.clone()),
                    model.type_name(actual)
                ),
                argument.span,
            ));
        }
    }
    Ok(function.return_type.clone())
}

const fn scalar_type(ty: ast::Type) -> Type {
    match ty {
        ast::Type::Bool => Type::Bool,
        ast::Type::Integer(integer) => Type::Integer(integer),
        ast::Type::F32 => Type::F32,
        ast::Type::F64 => Type::F64,
    }
}

pub(crate) fn comparison_operand_types(
    left: &Expr,
    right: &Expr,
    bindings: &Bindings,
    model: &SemanticModel,
) -> SemanticResult<(Type, Type)> {
    let left_type = type_of_expr_expected(left, bindings, None, model)?;
    let right_type = type_of_expr_expected(right, bindings, None, model)?;

    if left_type == right_type {
        return Ok((left_type, right_type));
    }

    let contextual_left = type_of_expr_expected(left, bindings, Some(right_type.clone()), model)?;

    if contextual_left == right_type {
        return Ok((contextual_left, right_type));
    }

    let contextual_right = type_of_expr_expected(right, bindings, Some(left_type.clone()), model)?;

    Ok((left_type, contextual_right))
}

fn operator_name(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::Equal => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::Less => "<",
        BinaryOp::LessEqual => "<=",
        BinaryOp::Greater => ">",
        BinaryOp::GreaterEqual => ">=",
    }
}

const fn is_numeric(ty: &Type) -> bool {
    matches!(ty, Type::Integer(_) | Type::F32 | Type::F64)
}

const fn is_arithmetic(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide
    )
}

const fn is_ordering(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual
    )
}

const fn is_comparison(op: BinaryOp) -> bool {
    !is_arithmetic(op)
}

#[cfg(test)]
mod tests {
    use crate::{lexer::lex, parser::parse, source::Span, types::IntegerType};

    use super::{Type, analyze, check};

    #[test]
    fn explicit_f32_guides_float_literals() {
        let program = parse(lex("x: f32 = 0.1 + 0.2;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(
            bindings.get("x").map(|binding| binding.ty.clone()),
            Some(Type::F32)
        );
    }

    #[test]
    fn explicit_f64_guides_float_literals() {
        let program = parse(lex("x: f64 = 0.1 + 0.2;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(
            bindings.get("x").map(|binding| binding.ty.clone()),
            Some(Type::F64)
        );
    }

    #[test]
    fn infer_defaults_float_to_f64() {
        let program = parse(lex("x: infer = 0.1 + 0.2;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(
            bindings.get("x").map(|binding| binding.ty.clone()),
            Some(Type::F64)
        );
    }

    #[test]
    fn suffix_can_force_f32() {
        let program = parse(lex("x: infer = 0.1f32 + 0.2f32;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(
            bindings.get("x").map(|binding| binding.ty.clone()),
            Some(Type::F32)
        );
    }

    #[test]
    fn rejects_integer_float_mix() {
        let program = parse(lex("x: infer = 1 + 0.1;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "cannot apply `+` to i64 and f64");
        assert_eq!(error.primary_span(), Some(Span::new(11, 18)));
    }

    #[test]
    fn rejects_positive_integer_literal_outside_i64() {
        let program = parse(lex("x: i64 = 9223372036854775808;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "integer literal does not fit in i64");
        assert_eq!(error.primary_span(), Some(Span::new(9, 28)));
    }

    #[test]
    fn accepts_the_minimum_i64_literal() {
        let program = parse(lex("x: i64 = -9223372036854775808;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(
            bindings.get("x").map(|binding| binding.ty.clone()),
            Some(Type::Integer(IntegerType::I64))
        );
    }

    #[test]
    fn rejects_integer_literal_below_i64() {
        let program = parse(lex("x: i64 = -9223372036854775809;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "integer literal does not fit in i64");
        assert_eq!(error.primary_span(), Some(Span::new(9, 29)));
    }

    #[test]
    fn reports_unknown_binding_at_variable() {
        let program = parse(lex("print(missing);").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "unknown binding `missing`");
        assert_eq!(error.primary_span(), Some(Span::new(6, 13)));
    }

    #[test]
    fn reports_duplicate_binding_at_statement() {
        let program = parse(lex("x: i64 = 1; x: i64 = 2;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "duplicate binding `x`");
        assert_eq!(error.primary_span(), Some(Span::new(12, 23)));
    }

    #[test]
    fn reports_binding_type_mismatch_at_value() {
        let program = parse(lex("x: f32 = 1;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(
            error.message(),
            "type mismatch for `x`: expected f32, found i64"
        );
        assert_eq!(error.primary_span(), Some(Span::new(9, 10)));
    }

    #[test]
    fn accepts_assignment_to_mutable_binding() {
        let program = parse(lex("mut x: i64 = 1; x = x + 1;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(
            bindings.get("x").map(|binding| binding.ty.clone()),
            Some(Type::Integer(IntegerType::I64))
        );
        assert_eq!(bindings.get("x").map(|binding| binding.mutable), Some(true));
    }

    #[test]
    fn accepts_assignment_to_nested_array_element() {
        let program =
            parse(lex("mut matrix: [[i64; 2]; 2] = [[1, 2], [3, 4]]; matrix[1][0] = 9;").unwrap())
                .unwrap();

        assert!(check(&program).is_ok());
    }

    #[test]
    fn rejects_array_element_assignment_through_immutable_binding() {
        let program = parse(lex("values: [i64; 2] = [1, 2]; values[0] = 3;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(
            error.message(),
            "cannot assign to immutable binding `values`"
        );
    }

    #[test]
    fn rejects_indexing_non_array_assignment_target() {
        let program = parse(lex("mut value: i64 = 1; value[0] = 2;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(
            error.message(),
            "cannot index assignment target of type i64"
        );
    }

    #[test]
    fn rejects_array_assignment_index_with_non_integer_type() {
        let program =
            parse(lex("mut values: [i64; 2] = [1, 2]; values[true] = 3;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "array index must be i64, found bool");
    }

    #[test]
    fn rejects_array_element_assignment_with_different_type() {
        let program =
            parse(lex("mut values: [i64; 2] = [1, 2]; values[0] = 0.5;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(
            error.message(),
            "type mismatch for assignment to `values`: expected i64, found f64"
        );
    }

    #[test]
    fn rejects_assignment_to_immutable_binding() {
        let program = parse(lex("x: i64 = 1; x = 2;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "cannot assign to immutable binding `x`");
        assert_eq!(error.primary_span(), Some(Span::new(12, 13)));
    }

    #[test]
    fn rejects_assignment_to_unknown_binding() {
        let program = parse(lex("missing = 1;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "unknown binding `missing`");
        assert_eq!(error.primary_span(), Some(Span::new(0, 7)));
    }

    #[test]
    fn rejects_assignment_with_different_type() {
        let program = parse(lex("mut x: i64 = 1; x = 0.5;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(
            error.message(),
            "type mismatch for assignment to `x`: expected i64, found f64"
        );
        assert_eq!(error.primary_span(), Some(Span::new(20, 23)));
    }

    #[test]
    fn checks_boolean_and_numeric_comparisons() {
        let program =
            parse(lex("a: bool = true == !false; b: bool = 1 < 2; c: bool = 0.1 < 0.2;").unwrap())
                .unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(
            bindings.get("a").map(|binding| binding.ty.clone()),
            Some(Type::Bool)
        );
        assert_eq!(
            bindings.get("b").map(|binding| binding.ty.clone()),
            Some(Type::Bool)
        );
        assert_eq!(
            bindings.get("c").map(|binding| binding.ty.clone()),
            Some(Type::Bool)
        );
    }

    #[test]
    fn comparison_uses_float_context_from_either_operand() {
        let program =
            parse(lex("value: f32 = 0.5; a: bool = value < 1.0; b: bool = 0.1 < value;").unwrap())
                .unwrap();

        check(&program).unwrap();
    }

    #[test]
    fn rejects_arithmetic_on_booleans() {
        let program = parse(lex("value: bool = true + false;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "cannot apply `+` to bool");
    }

    #[test]
    fn accepts_shadowing_in_nested_blocks() {
        let program =
            parse(lex("x: i64 = 1; if true { x: bool = false; print(x); } print(x);").unwrap())
                .unwrap();

        check(&program).unwrap();
    }

    #[test]
    fn rejects_block_binding_outside_its_scope() {
        let program = parse(lex("if true { local: i64 = 1; } print(local);").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "unknown binding `local`");
    }

    #[test]
    fn requires_boolean_if_condition() {
        let program = parse(lex("if 1 { print(1); }").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "if condition must be bool, found i64");
    }

    #[test]
    fn requires_boolean_while_condition() {
        let program =
            crate::parser::parse(crate::lexer::lex("while 1 { print(1); }").unwrap()).unwrap();

        let error = check(&program).unwrap_err();
        assert_eq!(error.message(), "while condition must be bool, found i64");
    }

    #[test]
    fn rejects_break_outside_loop() {
        let program = crate::parser::parse(crate::lexer::lex("break;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();
        assert_eq!(error.message(), "break can only be used inside a loop");
    }

    #[test]
    fn rejects_continue_outside_loop() {
        let program = crate::parser::parse(crate::lexer::lex("continue;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();
        assert_eq!(error.message(), "continue can only be used inside a loop");
    }

    #[test]
    fn accepts_loop_control_inside_nested_if() {
        let program = crate::parser::parse(
            crate::lexer::lex(
                "while true {
                    if true { continue; }
                    if false { break; }
                }",
            )
            .unwrap(),
        )
        .unwrap();

        check(&program).unwrap();
    }

    #[test]
    fn resolves_forward_product_type_references_and_fields() {
        let source = "
            type Line { start: Point, end: Point, }
            type Point { x: f64, y: f64 = 0.0, }
            line: Line = Line {
                end: Point { x: 2.0, },
                start: Point { y: 1.0, x: 0.0, },
            };
            print(line.end.x);
        ";
        let program = parse(lex(source).unwrap()).unwrap();
        let model = analyze(&program).unwrap();

        assert_eq!(model.type_definitions[0].id.0, 0);
        assert_eq!(model.type_definitions[0].name, "Line");
        assert_eq!(model.type_definitions[1].id.0, 1);
        assert_eq!(model.type_definitions[1].fields[1].id.0, 1);
        assert_eq!(
            model.bindings.get("line").map(|binding| binding.ty.clone()),
            Some(Type::Named(super::TypeId(0)))
        );
    }

    #[test]
    fn rejects_missing_product_field_without_default() {
        let source = "type Point { x: f64, y: f64, } p: Point = Point { x: 1.0, };";
        let program = parse(lex(source).unwrap()).unwrap();
        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "missing field `y` for type `Point`");
    }

    #[test]
    fn rejects_duplicate_product_fields() {
        let source = "type Point { x: f64, } p: Point = Point { x: 1.0, x: 2.0, };";
        let program = parse(lex(source).unwrap()).unwrap();
        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "field `x` is specified more than once");
    }

    #[test]
    fn rejects_recursive_product_type_with_infinite_size() {
        let source = "type A { b: B, } type B { a: A, }";
        let program = parse(lex(source).unwrap()).unwrap();
        let error = check(&program).unwrap_err();

        assert_eq!(
            error.message(),
            "type `B` has infinite size through field `a`"
        );
    }

    #[test]
    fn accepts_nested_arrays_after_preserving_their_type_structure() {
        let source = "values: [[i64; 2]; 2] = [[1, 2], [3, 4]];";
        let program = parse(lex(source).unwrap()).unwrap();

        check(&program).unwrap();
    }

    #[test]
    fn accepts_fixed_array_fields() {
        let source = "type Row { values: [i64; 2], } row: Row = Row { values: [1, 2], };";
        let program = parse(lex(source).unwrap()).unwrap();

        check(&program).unwrap();
    }

    #[test]
    fn accepts_product_type_array_elements() {
        let source = "
            type Point { x: i64, }
            points: [Point; 2] = [Point { x: 1, }, Point { x: 2, }];
        ";
        let program = parse(lex(source).unwrap()).unwrap();

        check(&program).unwrap();
    }

    #[test]
    fn rejects_infinite_size_through_an_array_field() {
        let source = "type Node { children: [Node; 1], }";
        let program = parse(lex(source).unwrap()).unwrap();
        let error = check(&program).unwrap_err();

        assert_eq!(
            error.message(),
            "type `Node` has infinite size through field `children`"
        );
    }

    #[test]
    fn keeps_named_product_types_distinct() {
        let source = "
            type Point { x: f64, }
            type Velocity { x: f64, }
            point: Point = Velocity { x: 1.0, };
        ";
        let program = parse(lex(source).unwrap()).unwrap();
        let error = check(&program).unwrap_err();

        assert_eq!(
            error.message(),
            "type mismatch for `point`: expected Point, found Velocity"
        );
    }

    #[test]
    fn checks_function_calls_and_returns() {
        check(
            &parse(
                lex(
                    "fn add(left: i64, right: i64) -> i64 { return left + right; }
             answer: i64 = add(20, 22);",
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn rejects_missing_function_return() {
        let error = check(&parse(lex("fn answer() -> i64 { value: i64 = 42; }").unwrap()).unwrap())
            .expect_err("a value function must return");

        assert_eq!(
            error.message(),
            "function `answer` may finish without returning a value"
        );
    }

    #[test]
    fn rejects_direct_and_indirect_recursion_for_now() {
        for source in [
            "fn repeat(value: i64) -> i64 { return repeat(value); }",
            "fn first(value: i64) -> i64 { return second(value); }
             fn second(value: i64) -> i64 { return first(value); }",
        ] {
            let error = check(&parse(lex(source).unwrap()).unwrap())
                .expect_err("recursive calls must be rejected consistently by every backend");

            assert_eq!(
                error.message(),
                "recursive function calls are not supported yet"
            );
        }
    }

    #[test]
    fn rejects_explicit_main_with_top_level_statements() {
        let error = check(
            &parse(
                lex("fn main() -> void { print(1); }
             print(2);")
                .unwrap(),
            )
            .unwrap(),
        )
        .expect_err("main and top-level statements must not be mixed");

        assert_eq!(
            error.message(),
            "an explicit `main` function cannot be combined with top-level statements"
        );
    }
}
