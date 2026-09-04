use crate::source::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Bool,
    I64,
    F32,
    F64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeSpec {
    Explicit(TypeRef),
    Infer,
}

/// ソースに書かれた、まだ意味解析で解決していない型名です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeRef {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub items: Vec<Item>,
}

impl Program {
    pub fn statement(&self, index: usize) -> &Stmt {
        self.items
            .iter()
            .filter_map(|item| match item {
                Item::TypeDefinition(_) | Item::FunctionDefinition(_) => None,
                Item::Statement(statement) => Some(statement),
            })
            .nth(index)
            .expect("statement index must exist")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    TypeDefinition(TypeDefinition),
    FunctionDefinition(FunctionDefinition),
    Statement(Stmt),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDefinition {
    pub name: String,
    pub name_span: Span,
    pub parameters: Vec<Parameter>,
    pub return_type: ReturnTypeRef,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub name_span: Span,
    pub type_ref: TypeRef,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnTypeRef {
    Void(Span),
    Value(TypeRef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDefinition {
    pub name: String,
    pub name_span: Span,
    pub fields: Vec<FieldDefinition>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDefinition {
    pub name: String,
    pub name_span: Span,
    pub type_ref: TypeRef,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StmtKind {
    Binding {
        mutable: bool,
        name: String,
        type_spec: TypeSpec,
        value: Expr,
    },
    Assignment {
        name: String,
        name_span: Span,
        value: Expr,
    },
    Print {
        value: Expr,
    },
    Call {
        value: Expr,
    },
    Return {
        value: Option<Expr>,
    },
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        initializer: Box<Stmt>,
        condition: Expr,
        update: Box<Stmt>,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    Boolean(bool),
    Integer(i64),
    Float {
        text: String,
        explicit_type: Option<Type>,
    },
    Variable(String),
    Construct {
        type_name: String,
        type_name_span: Span,
        fields: Vec<FieldValue>,
    },
    FieldAccess {
        base: Box<Expr>,
        field_name: String,
        field_name_span: Span,
    },
    Call {
        name: String,
        name_span: Span,
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
    pub name: String,
    pub name_span: Span,
    pub value: Expr,
    pub span: Span,
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
