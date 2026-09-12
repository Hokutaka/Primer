use super::*;

#[derive(Clone)]
struct Symbol {
    name: String,
    public: bool,
}
#[derive(Default)]
struct Names {
    functions: HashMap<String, Symbol>,
    types: HashMap<String, Symbol>,
}

pub(super) fn resolve(units: &[Unit]) -> Result<Program, Diagnostic> {
    let mut names = Vec::new();
    for (index, unit) in units.iter().enumerate() {
        let mut scope = Names::default();
        for item in &unit.module.program.items {
            let (name, span, functions) = match item {
                Item::FunctionDefinition(d) => (&d.name, d.name_span, true),
                Item::TypeDefinition(d) => (&d.name, d.name_span, false),
                _ => continue,
            };
            if Type::from_name(name).is_some() || (functions && name == "byte_len") {
                return Err(Diagnostic::new(
                    format!("definition name `{name}` is reserved"),
                    span,
                ));
            }
            if unit.imports.contains_key(name) {
                return Err(Diagnostic::new(
                    format!("definition `{name}` conflicts with an import alias"),
                    span,
                ));
            }
            let table = if functions {
                &mut scope.functions
            } else {
                &mut scope.types
            };
            let symbol = Symbol {
                name: if index == 0 && functions && name == "main" {
                    name.clone()
                } else {
                    format!("module_{}_{name}", unit.id.index())
                },
                public: unit
                    .module
                    .exports
                    .iter()
                    .any(|(_, exported)| *exported == span),
            };
            if table.insert(name.clone(), symbol).is_some() {
                return Err(Diagnostic::new(
                    format!(
                        "duplicate {} `{name}`",
                        if functions { "function" } else { "type" }
                    ),
                    span,
                ));
            }
        }
        names.push(scope);
    }
    let mut program = Program { items: Vec::new() };
    for (index, unit) in units.iter().enumerate() {
        let resolver = Resolver {
            units,
            names: &names,
            current: index,
        };
        for mut item in unit.module.program.items.clone() {
            match &mut item {
                Item::TypeDefinition(d) => {
                    let public = names[index].types[&d.name].public;
                    d.name = names[index].types[&d.name].name.clone();
                    for field in &mut d.fields {
                        resolver.ty(&mut field.type_ref, public)?;
                        if let Some(default) = &mut field.default {
                            resolver.expr(default)?;
                        }
                    }
                }
                Item::FunctionDefinition(d) => {
                    let public = names[index].functions[&d.name].public;
                    d.name = names[index].functions[&d.name].name.clone();
                    for parameter in &mut d.parameters {
                        resolver.binding(&parameter.name, parameter.name_span)?;
                        resolver.ty(&mut parameter.type_ref, public)?;
                    }
                    if let ReturnTypeRef::Value(ty) = &mut d.return_type {
                        resolver.ty(ty, public)?;
                    }
                    resolver.statements(&mut d.body)?;
                }
                Item::Statement(statement) => resolver.statement(statement)?,
            }
            program.items.push(item);
        }
    }
    Ok(program)
}

struct Resolver<'a> {
    units: &'a [Unit],
    names: &'a [Names],
    current: usize,
}
impl Resolver<'_> {
    fn name(
        &self,
        name: &str,
        span: Span,
        functions: bool,
        public: bool,
    ) -> Result<String, Diagnostic> {
        let (target, member, qualified) = if let Some((alias, member)) = name.split_once("::") {
            (
                *self.units[self.current].imports.get(alias).ok_or_else(|| {
                    Diagnostic::new(format!("unknown import alias `{alias}`"), span)
                })?,
                member,
                true,
            )
        } else {
            (self.current, name, false)
        };
        let table = if functions {
            &self.names[target].functions
        } else {
            &self.names[target].types
        };
        let kind = if functions { "function" } else { "type" };
        let symbol = table
            .get(member)
            .ok_or_else(|| Diagnostic::new(format!("unknown {kind} `{name}`"), span))?;
        if (qualified || public) && !symbol.public {
            return Err(Diagnostic::new(
                format!(
                    "private {kind} `{name}` cannot be used {}",
                    if qualified {
                        "through an import"
                    } else {
                        "in a public signature"
                    }
                ),
                span,
            ));
        }
        Ok(symbol.name.clone())
    }

    fn ty(&self, ty: &mut TypeRef, public: bool) -> Result<(), Diagnostic> {
        match &mut ty.kind {
            TypeRefKind::Named(name) if Type::from_name(name).is_none() && name != "infer" => {
                *name = self.name(name, ty.span, false, public)?
            }
            TypeRefKind::Array { element, .. } => self.ty(element, public)?,
            _ => {}
        }
        Ok(())
    }

    fn binding(&self, name: &str, span: Span) -> Result<(), Diagnostic> {
        if self.units[self.current].imports.contains_key(name) {
            return Err(Diagnostic::new(
                format!("binding `{name}` conflicts with an import alias"),
                span,
            ));
        }
        Ok(())
    }

    fn statements(&self, body: &mut [Stmt]) -> Result<(), Diagnostic> {
        for statement in body {
            self.statement(statement)?;
        }
        Ok(())
    }

    fn statement(&self, statement: &mut Stmt) -> Result<(), Diagnostic> {
        match &mut statement.kind {
            StmtKind::Binding {
                name,
                type_spec,
                value,
                ..
            } => {
                self.binding(name, statement.span)?;
                if let TypeSpec::Explicit(ty) = type_spec {
                    self.ty(ty, false)?;
                }
                self.expr(value)?;
            }
            StmtKind::Assignment { target, value } => {
                for AssignmentProjection::Index { index, .. } in &mut target.projections {
                    self.expr(index)?;
                }
                self.expr(value)?;
            }
            StmtKind::Print { value } | StmtKind::Call { value } => self.expr(value)?,
            StmtKind::Return { value } => {
                if let Some(value) = value {
                    self.expr(value)?;
                }
            }
            StmtKind::If {
                condition,
                then_body,
                else_body,
            } => {
                self.expr(condition)?;
                self.statements(then_body)?;
                self.statements(else_body)?;
            }
            StmtKind::While { condition, body } => {
                self.expr(condition)?;
                self.statements(body)?;
            }
            StmtKind::For {
                initializer,
                condition,
                update,
                body,
            } => {
                self.statement(initializer)?;
                self.expr(condition)?;
                self.statement(update)?;
                self.statements(body)?;
            }
            StmtKind::Break | StmtKind::Continue => {}
        }
        Ok(())
    }

    fn expr(&self, expr: &mut Expr) -> Result<(), Diagnostic> {
        match &mut expr.kind {
            ExprKind::Call {
                name,
                name_span,
                arguments,
            } => {
                if name != "byte_len" {
                    *name = self.name(name, *name_span, true, false)?;
                }
                for argument in arguments {
                    self.expr(argument)?;
                }
            }
            ExprKind::Construct {
                type_name,
                type_name_span,
                fields,
            } => {
                *type_name = self.name(type_name, *type_name_span, false, false)?;
                for field in fields {
                    self.expr(&mut field.value)?;
                }
            }
            ExprKind::Convert { target, value, .. } => {
                self.ty(target, false)?;
                self.expr(value)?;
            }
            ExprKind::Array(values) => {
                for value in values {
                    self.expr(value)?;
                }
            }
            ExprKind::Index { base, index } => {
                self.expr(base)?;
                self.expr(index)?;
            }
            ExprKind::FieldAccess { base, .. } | ExprKind::Unary { value: base, .. } => {
                self.expr(base)?
            }
            ExprKind::Logical { left, right, .. } | ExprKind::Binary { left, right, .. } => {
                self.expr(left)?;
                self.expr(right)?;
            }
            ExprKind::Variable(name) if name.contains("::") => {
                return Err(Diagnostic::new(
                    "modules expose functions and types, not variable values",
                    expr.span,
                ));
            }
            _ => {}
        }
        Ok(())
    }
}
