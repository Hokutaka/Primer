use std::collections::HashMap;

use crate::{
    ast::{self, BinaryOp, Expr, ExprKind, Item, Program, Stmt, StmtKind, TypeSpec},
    diagnostic::Diagnostic,
    source::Span,
};

pub type Bindings = HashMap<String, BindingInfo>;
type SemanticResult<T> = Result<T, Diagnostic>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Bool,
    I64,
    F32,
    F64,
    Named(TypeId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub struct SemanticModel {
    pub bindings: Bindings,
    pub type_definitions: Vec<TypeDefinition>,
    type_names: HashMap<String, TypeId>,
}

impl SemanticModel {
    pub fn resolve_type_ref(&self, type_ref: &ast::TypeRef) -> SemanticResult<Type> {
        resolve_type_name(&type_ref.name, type_ref.span, &self.type_names)
    }

    pub fn type_definition(&self, id: TypeId) -> &TypeDefinition {
        &self.type_definitions[id.0]
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
            Type::I64 => "i64".into(),
            Type::F32 => "f32".into(),
            Type::F64 => "f64".into(),
            Type::Named(id) => self.type_definition(id).name.clone(),
        }
    }
}

pub fn check(program: &Program) -> SemanticResult<Bindings> {
    Ok(analyze(program)?.bindings)
}

pub fn analyze(program: &Program) -> SemanticResult<SemanticModel> {
    let type_names = register_type_names(program)?;
    let type_definitions = resolve_type_definitions(program, &type_names)?;
    let mut model = SemanticModel {
        bindings: HashMap::new(),
        type_definitions,
        type_names,
    };

    reject_infinite_types(&model)?;
    check_defaults(&model)?;

    let mut scopes = vec![HashMap::new()];
    for item in &program.items {
        match item {
            Item::TypeDefinition(_) => {}
            Item::Statement(statement) => {
                check_statements(std::slice::from_ref(statement), &mut scopes, 0, &model)?;
            }
        }
    }
    model.bindings = scopes.pop().expect("top-level scope must exist");
    Ok(model)
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
                ty: resolve_type_name(&field.type_ref.name, field.type_ref.span, type_names)?,
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
        "i64" => Ok(Type::I64),
        "f32" => Ok(Type::F32),
        "f64" => Ok(Type::F64),
        _ => type_names
            .get(name)
            .copied()
            .map(Type::Named)
            .ok_or_else(|| Diagnostic::new(format!("unknown type `{name}`"), span)),
    }
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
            let Type::Named(next) = field.ty else {
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
            let actual = model.type_of_expr_expected(default, &bindings, Some(field.ty))?;
            if actual != field.ty {
                return Err(Diagnostic::new(
                    format!(
                        "default for field `{}` expects {}, found {}",
                        field.name,
                        model.type_name(field.ty),
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
                        let actual =
                            model.type_of_expr_expected(value, &bindings, Some(expected))?;

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

            StmtKind::Assignment {
                name,
                name_span,
                value,
            } => {
                let binding = bindings.get(name).copied().ok_or_else(|| {
                    Diagnostic::new(format!("unknown binding `{name}`"), *name_span)
                })?;

                if !binding.mutable {
                    return Err(Diagnostic::new(
                        format!("cannot assign to immutable binding `{name}`"),
                        *name_span,
                    ));
                }

                let actual = model.type_of_expr_expected(value, &bindings, Some(binding.ty))?;

                if actual != binding.ty {
                    return Err(Diagnostic::new(
                        format!(
                            "type mismatch for assignment to `{name}`: expected {}, found {}",
                            model.type_name(binding.ty),
                            model.type_name(actual),
                        ),
                        value.span,
                    ));
                }
            }

            StmtKind::Print { value } => {
                let ty = model.type_of_expr(value, &bindings)?;
                if matches!(ty, Type::Named(_)) {
                    return Err(Diagnostic::new(
                        format!("cannot print value of type {}", model.type_name(ty)),
                        value.span,
                    ));
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
                check_statements(then_body, scopes, loop_depth, model)?;
                scopes.pop();

                scopes.push(HashMap::new());
                check_statements(else_body, scopes, loop_depth, model)?;
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
                check_statements(body, scopes, loop_depth + 1, model)?;
                scopes.pop();
            }

            StmtKind::For {
                initializer,
                condition,
                update,
                body,
            } => {
                scopes.push(HashMap::new());
                check_statements(std::slice::from_ref(initializer), scopes, loop_depth, model)?;

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

                check_statements(std::slice::from_ref(update), scopes, loop_depth, model)?;

                scopes.push(HashMap::new());
                check_statements(body, scopes, loop_depth + 1, model)?;
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
        visible.extend(scope.iter().map(|(name, binding)| (name.clone(), *binding)));
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

        ExprKind::Integer(_) => Ok(Type::I64),

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
            .map(|binding| binding.ty)
            .ok_or_else(|| Diagnostic::new(format!("unknown binding `{name}`"), expr.span)),

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

                let actual =
                    model.type_of_expr_expected(&field_value.value, bindings, Some(field.ty))?;
                if actual != field.ty {
                    return Err(Diagnostic::new(
                        format!(
                            "field `{}` expects {}, found {}",
                            field.name,
                            model.type_name(field.ty),
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
                .map(|field| field.ty)
                .ok_or_else(|| {
                    Diagnostic::new(
                        format!("type `{}` has no field `{field_name}`", definition.name),
                        *field_name_span,
                    )
                })
        }

        ExprKind::Unary { op, value } => match op {
            crate::ast::UnaryOp::Negate => {
                let ty = type_of_expr_expected(value, bindings, expected, model)?;

                if !is_numeric(ty) {
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
                    type_of_expr_expected(left, bindings, expected, model)?,
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

            if matches!(left_type, Type::Named(_)) {
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
                if !is_numeric(left_type) {
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
                if !is_numeric(left_type) {
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

const fn scalar_type(ty: ast::Type) -> Type {
    match ty {
        ast::Type::Bool => Type::Bool,
        ast::Type::I64 => Type::I64,
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

    let contextual_left = type_of_expr_expected(left, bindings, Some(right_type), model)?;

    if contextual_left == right_type {
        return Ok((contextual_left, right_type));
    }

    let contextual_right = type_of_expr_expected(right, bindings, Some(left_type), model)?;

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

const fn is_numeric(ty: Type) -> bool {
    matches!(ty, Type::I64 | Type::F32 | Type::F64)
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
    use crate::{lexer::lex, parser::parse, source::Span};

    use super::{Type, analyze, check};

    #[test]
    fn explicit_f32_guides_float_literals() {
        let program = parse(lex("x: f32 = 0.1 + 0.2;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(bindings.get("x").map(|binding| binding.ty), Some(Type::F32));
    }

    #[test]
    fn explicit_f64_guides_float_literals() {
        let program = parse(lex("x: f64 = 0.1 + 0.2;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(bindings.get("x").map(|binding| binding.ty), Some(Type::F64));
    }

    #[test]
    fn infer_defaults_float_to_f64() {
        let program = parse(lex("x: infer = 0.1 + 0.2;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(bindings.get("x").map(|binding| binding.ty), Some(Type::F64));
    }

    #[test]
    fn suffix_can_force_f32() {
        let program = parse(lex("x: infer = 0.1f32 + 0.2f32;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(bindings.get("x").map(|binding| binding.ty), Some(Type::F32));
    }

    #[test]
    fn rejects_integer_float_mix() {
        let program = parse(lex("x: infer = 1 + 0.1;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "cannot apply `+` to i64 and f64");
        assert_eq!(error.primary_span(), Some(Span::new(11, 18)));
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

        assert_eq!(bindings.get("x").map(|binding| binding.ty), Some(Type::I64));
        assert_eq!(bindings.get("x").map(|binding| binding.mutable), Some(true));
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
            bindings.get("a").map(|binding| binding.ty),
            Some(Type::Bool)
        );
        assert_eq!(
            bindings.get("b").map(|binding| binding.ty),
            Some(Type::Bool)
        );
        assert_eq!(
            bindings.get("c").map(|binding| binding.ty),
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
            model.bindings.get("line").map(|binding| binding.ty),
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
}
