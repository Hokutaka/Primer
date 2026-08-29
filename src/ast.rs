use crate::source::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    I64,
    F32,
    F64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeSpec {
    Explicit(Type),
    Infer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StmtKind {
    Binding {
        name: String,
        type_spec: TypeSpec,
        value: Expr,
    },
    Print {
        value: Expr,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    Integer(i64),
    Float {
        text: String,
        explicit_type: Option<Type>,
    },
    Variable(String),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}
