use crate::{
    source::{ConversionSyntax, Span},
    types::IntegerType,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Bool,
    String,
    Integer(IntegerType),
    F32,
    F64,
}

impl Type {
    /// 組み込み型の名前を解決します。関数やユーザー定義型で上書きできません。
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "bool" => Some(Self::Bool),
            "string" => Some(Self::String),
            "f32" => Some(Self::F32),
            "f64" => Some(Self::F64),
            name => IntegerType::from_name(name).map(Self::Integer),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeSpec {
    Explicit(TypeRef),
    Infer,
}

/// ソースに書かれた、まだ意味解析で解決していない型です。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeRef {
    pub kind: TypeRefKind,
    pub span: Span,
}

impl TypeRef {
    pub fn is_named(&self, expected: &str) -> bool {
        matches!(&self.kind, TypeRefKind::Named(name) if name == expected)
    }
}

/// ソースに書かれた型の形を、入れ子を失わずに保持します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeRefKind {
    Named(String),
    Array {
        element: Box<TypeRef>,
        length: usize,
    },
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
        target: AssignmentTarget,
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
pub struct AssignmentTarget {
    pub name: String,
    pub name_span: Span,
    pub projections: Vec<AssignmentProjection>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignmentProjection {
    Index { index: Expr, span: Span },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

/// 型が決まる前の10進整数リテラルを保持します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegerLiteral {
    digits: String,
    explicit_type: Option<IntegerType>,
}

impl IntegerLiteral {
    /// 符号を含まない10進数字から整数リテラルを作ります。
    pub fn decimal(digits: impl Into<String>) -> Self {
        Self {
            digits: digits.into(),
            explicit_type: None,
        }
    }

    pub fn with_type(digits: impl Into<String>, ty: IntegerType) -> Self {
        Self {
            digits: digits.into(),
            explicit_type: Some(ty),
        }
    }

    pub const fn explicit_type(&self) -> Option<IntegerType> {
        self.explicit_type
    }

    /// 符号や型接尾辞を含まない10進数字を返します。
    pub fn digits(&self) -> &str {
        &self.digits
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    Logical {
        op: LogicalOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Convert {
        target: TypeRef,
        value: Box<Expr>,
        syntax: ConversionSyntax,
    },
    Boolean(bool),
    String(String),
    Integer(IntegerLiteral),
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
    Array(Vec<Expr>),
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
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
    BitNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOp {
    And,
    Or,
}
