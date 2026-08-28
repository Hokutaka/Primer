#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    I64,
    F32,
    F64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub frame_size: usize,
    pub float_constants: Vec<FloatConstant>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FloatConstant {
    F32 { id: usize, bits: u32 },
    F64 { id: usize, bits: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    MovI64ImmediateToRax(i64),

    LoadI64FromStack(isize),
    StoreI64ToStack(isize),

    LoadF32FromStack(isize),
    StoreF32ToStack(isize),

    LoadF64FromStack(isize),
    StoreF64ToStack(isize),

    LoadF32Constant(usize),
    LoadF64Constant(usize),

    NegI64,
    NegF32,
    NegF64,

    MoveRaxToRcx,
    LoadI64ScratchToRax(isize),
    I64Binary(BinaryOp),
    SignExtendRax,
    DivideRaxByRcx,

    CopyXmm0ToXmm1F32,
    CopyXmm0ToXmm1F64,
    LoadF32ScratchToXmm0(isize),
    LoadF64ScratchToXmm0(isize),
    F32Binary(BinaryOp),
    F64Binary(BinaryOp),

    MoveRaxToRdx,
    LoadFormatI64ToRcx,

    ConvertF32ToF64Argument,
    MoveXmm1ToRdx,
    LoadFormatF32ToRcx,

    CopyXmm0ToXmm1F64Scalar,
    LoadFormatF64ToRcx,

    CallPrintf,
}
