#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    I64,
    F32,
    F64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub locals: Vec<Local>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Local {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    I64Const(i64),
    F32Const(String),
    F64Const(String),

    LocalGet(String),
    LocalSet(String),

    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,

    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Neg,

    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Neg,

    CallPrint(Type),
}
