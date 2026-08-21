#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    I64,
    Float,
    Double,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub slots: Vec<Slot>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slot {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Temp(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    Integer(i64),
    Float32(u32),
    Float64(u64),
    Temp(Temp),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    SDiv,
    FAdd,
    FSub,
    FMul,
    FDiv,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintFormat {
    I64,
    F32,
    F64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Alloca {
        slot: SlotId,
    },
    Store {
        ty: Type,
        value: Operand,
        slot: SlotId,
    },
    Load {
        dest: Temp,
        ty: Type,
        slot: SlotId,
    },
    Binary {
        dest: Temp,
        op: BinaryOp,
        ty: Type,
        left: Operand,
        right: Operand,
    },
    FNeg {
        dest: Temp,
        ty: Type,
        value: Operand,
    },
    FPExt {
        dest: Temp,
        value: Operand,
    },
    CallPrintf {
        format: PrintFormat,
        arg_ty: Type,
        value: Operand,
    },
}
