#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Bool,
    I64,
    Float,
    Double,
    Named(usize),
    Array {
        element: ArrayElementType,
        length: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayElementType {
    Bool,
    I64,
    Float,
    Double,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub array_types: Vec<Type>,
    pub type_definitions: Vec<TypeDefinition>,
    pub functions: Vec<Function>,
    pub explicit_main: Option<usize>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub id: usize,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDefinition {
    pub id: usize,
    pub name: String,
    pub fields: Vec<FieldDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDefinition {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Binding {
        name: String,
        ty: Type,
        value: Expr,
    },
    Assignment {
        name: String,
        value: Expr,
    },
    Print {
        format: PrintFormat,
        value: Expr,
    },
    Call {
        function_id: usize,
        function_name: String,
        arguments: Vec<Expr>,
    },
    Return(Option<Expr>),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintFormat {
    Bool,
    I64,
    F32,
    F64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub ty: Type,
    pub kind: ExprKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    Boolean(bool),
    Integer(i64),
    Float {
        text: String,
        suffix_f32: bool,
    },
    Variable(String),
    Construct {
        type_id: usize,
        fields: Vec<FieldValue>,
    },
    FieldAccess {
        field_name: String,
        base: Box<Expr>,
    },
    Array(Vec<Expr>),
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    Call {
        function_id: usize,
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
    pub name: String,
    pub value: Expr,
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
