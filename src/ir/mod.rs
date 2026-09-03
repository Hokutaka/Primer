pub mod builder;
pub mod text;

use crate::source::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Bool,
    I64,
    F32,
    F64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub statements: Vec<Statement>,
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
        id: BindingId,
        name: String,
        ty: Type,
        value: Expr,
    },
    Print {
        value: Expr,
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
