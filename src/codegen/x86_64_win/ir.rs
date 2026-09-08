#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    /// 不変な長さ付き静的バイト列への参照です。
    String,
    Bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub uses_strings: bool,
    pub strings: Vec<String>,
    pub functions: Vec<Function>,
    pub explicit_main: Option<usize>,
    pub frame_size: usize,
    pub float_constants: Vec<FloatConstant>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub id: usize,
    pub name: String,
    pub frame_size: usize,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Argument {
    Scalar {
        ty: Type,
        offset: isize,
    },
    /// 呼び出し先が自身のstackへコピーする値のaddressです。
    Aggregate {
        offset: isize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FloatConstant {
    F32 { id: usize, bits: u32 },
    F64 { id: usize, bits: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    LoadStringLength,
    LoadStringConstant(usize),
    CompareString {
        left_offset: isize,
        equal: bool,
    },
    PrintString,
    ConvertNumeric {
        conversion: crate::codegen::NumericConversion,
        label: usize,
    },
    BitNot {
        mask: i64,
    },
    IntegerBinary {
        op: crate::codegen::IntegerBinaryOp,
        ty: crate::types::IntegerType,
        label: usize,
    },
    CheckIntegerRange {
        ty: crate::types::IntegerType,
        label: usize,
    },
    Label {
        id: usize,
        name: &'static str,
    },
    JumpIfZero(usize),
    Jump(usize),

    MovI64ImmediateToRax(i64),

    LoadI64FromStack(isize),
    StoreI64ToStack(isize),

    LoadF32FromStack(isize),
    StoreF32ToStack(isize),

    LoadF64FromStack(isize),
    StoreF64ToStack(isize),

    CheckedArrayLoad {
        ty: Type,
        base_offset: isize,
        length: usize,
        label: usize,
    },
    CheckedArrayCopy {
        base_offset: isize,
        length: usize,
        element_slots: usize,
        destination_offset: isize,
        label: usize,
    },
    CheckedArrayAddress {
        base_offset: isize,
        base_is_pointer: bool,
        length: usize,
        element_slots: usize,
        destination_offset: isize,
        label: usize,
    },
    StoreI64ToPointer(isize),
    StoreF32ToPointer(isize),
    StoreF64ToPointer(isize),
    CopyToPointer {
        source_offset: isize,
        slots: usize,
        pointer_offset: isize,
    },

    StoreParameter {
        index: usize,
        ty: Type,
        offset: isize,
    },
    StoreAggregateParameter {
        index: usize,
        slots: usize,
        destination_offset: isize,
    },
    /// 内部呼び出し規約で`RAX`に渡された集約戻り値の保存先を退避します。
    StoreAggregateReturnPointer {
        offset: isize,
    },
    CopyToAggregateReturn {
        source_offset: isize,
        slots: usize,
        pointer_offset: isize,
    },
    Call {
        function_id: usize,
        arguments: Vec<Argument>,
        aggregate_result_offset: Option<isize>,
    },
    Return,

    LoadF32Constant(usize),
    LoadF64Constant(usize),

    NegI64,
    TrapIfOverflow(usize),
    NotBool,
    NegF32,
    NegF64,

    MoveRaxToRcx,
    LoadI64ScratchToRax(isize),
    I64Binary(BinaryOp),
    CompareI64(CompareOp),
    SignExtendRax,
    TrapIfInvalidI64Division(usize),
    DivideRaxByRcx,

    CopyXmm0ToXmm1F32,
    CopyXmm0ToXmm1F64,
    LoadF32ScratchToXmm0(isize),
    LoadF64ScratchToXmm0(isize),
    F32Binary(BinaryOp),
    F64Binary(BinaryOp),
    CompareF32(CompareOp),
    CompareF64(CompareOp),

    MoveRaxToRdx,
    LoadFormatI64ToRcx,

    ConvertF32ToF64Argument,
    MoveXmm1ToRdx,
    LoadFormatF32ToRcx,

    CopyXmm0ToXmm1F64Scalar,
    LoadFormatF64ToRcx,

    CallPrintf,
    CallPrintBool,
}
