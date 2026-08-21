#[derive(Debug, Clone)]
pub struct Module {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    I64,
    Single,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Temp(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    Integer(i64),
    Float32(String),
    Float64(String),
    Binding(String),
    Temp(Temp),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintFormat {
    I64,
    F32,
    F64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Copy {
        name: String,
        ty: Type,
        value: Operand,
    },
    Negate {
        dest: Temp,
        ty: Type,
        value: Operand,
    },
    Binary {
        dest: Temp,
        op: BinaryOp,
        ty: Type,
        left: Operand,
        right: Operand,
    },
    ExtendSingleToDouble {
        dest: Temp,
        value: Operand,
    },
    CallPrintf {
        dest: Temp,
        format: PrintFormat,
        arg_ty: Type,
        value: Operand,
    },
}
