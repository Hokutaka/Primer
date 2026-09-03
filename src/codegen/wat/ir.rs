#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Bool,
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
    I32Const(i32),
    I64Const(i64),
    F32Const(String),
    F64Const(String),

    LocalGet(String),
    LocalSet(String),

    If {
        then_instructions: Vec<Instruction>,
        else_instructions: Vec<Instruction>,
    },
    While {
        condition_instructions: Vec<Instruction>,
        body_instructions: Vec<Instruction>,
    },

    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LeS,
    I64GtS,
    I64GeS,

    I32Eq,
    I32Ne,
    I32Eqz,

    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Neg,
    F32Eq,
    F32Ne,
    F32Lt,
    F32Le,
    F32Gt,
    F32Ge,

    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Neg,
    F64Eq,
    F64Ne,
    F64Lt,
    F64Le,
    F64Gt,
    F64Ge,

    CallPrint(Type),
}
