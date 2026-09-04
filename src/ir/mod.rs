pub mod builder;
pub mod text;

use crate::source::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Bool,
    I64,
    F32,
    F64,
    Named(TypeId),
    Array { element: Box<Type>, length: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub type_definitions: Vec<TypeDefinition>,
    pub function_definitions: Vec<FunctionDefinition>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnType {
    Void,
    Value(Type),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDefinition {
    pub id: FunctionId,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: ReturnType,
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub id: BindingId,
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDefinition {
    pub id: TypeId,
    pub name: String,
    pub fields: Vec<FieldDefinition>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDefinition {
    pub id: FieldId,
    pub name: String,
    pub ty: Type,
    pub default: Option<Expr>,
    pub span: Span,
}

/// ソース上の名前がどの束縛を指すかを一意に識別します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BindingId(pub usize);

/// Primer IRの文と、その文が由来するソース範囲を表します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

/// Primer IRの文の種類を表します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementKind {
    Binding {
        id: BindingId,
        mutable: bool,
        name: String,
        ty: Type,
        value: Expr,
    },
    Assignment {
        target: AssignmentTarget,
        value: Expr,
    },
    Print {
        value: Expr,
    },
    Call {
        function_id: FunctionId,
        function_name: String,
        arguments: Vec<Expr>,
    },
    Return {
        value: Option<Expr>,
    },
    If {
        condition: Expr,
        then_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
    While {
        condition: Expr,
        body: Vec<Statement>,
    },
    For {
        initializer: Box<Statement>,
        condition: Expr,
        update: Box<Statement>,
        body: Vec<Statement>,
    },
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentTarget {
    pub id: BindingId,
    pub name: String,
    pub root_ty: Type,
    pub projections: Vec<AssignmentProjection>,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignmentProjection {
    Index {
        index: Expr,
        element: Type,
        length: usize,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub ty: Type,
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    Boolean(bool),
    Integer(i64),
    Float {
        text: String,
    },
    Variable {
        id: BindingId,
        name: String,
    },
    Construct {
        type_id: TypeId,
        type_name: String,
        fields: Vec<FieldValue>,
    },
    FieldAccess {
        type_id: TypeId,
        field_id: FieldId,
        field_name: String,
        base: Box<Expr>,
    },
    Array(Vec<Expr>),
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    Call {
        function_id: FunctionId,
        function_name: String,
        arguments: Vec<Expr>,
    },
    Unary {
        op: UnaryOp,
        value: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldValue {
    pub id: FieldId,
    pub name: String,
    pub value: Expr,
    pub origin: FieldValueOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldValueOrigin {
    Explicit { span: Span },
    Default { definition_span: Span },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}
