#[derive(Debug, Clone)]
pub struct Module {
    pub functions: Vec<Function>,
    pub explicit_main: Option<usize>,
    pub slots: Vec<Slot>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub id: usize,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub slots: Vec<Slot>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
    pub slot: usize,
}

#[derive(Debug, Clone)]
pub struct Slot {
    pub name: String,
    pub size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Bool,
    I64,
    Single,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Temp(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    Boolean(bool),
    Integer(i64),
    Float32(String),
    Float64(String),
    Temp(Temp),
    Slot(usize),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintFormat {
    I64,
    F32,
    F64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Label {
        id: usize,
        name: &'static str,
    },
    Branch {
        condition: Operand,
        then_label: usize,
        else_label: usize,
    },
    Jump(usize),
    Store {
        address: Operand,
        ty: Type,
        value: Operand,
    },
    Load {
        dest: Temp,
        address: Operand,
        ty: Type,
    },
    Address {
        dest: Temp,
        base: Operand,
        offset: usize,
    },
    Blit {
        source: Operand,
        destination: Operand,
        size: usize,
    },
    Abort,
    Call {
        dest: Option<Temp>,
        function_id: usize,
        return_type: Option<Type>,
        arguments: Vec<(Type, Operand)>,
    },
    Return {
        value: Option<(Type, Operand)>,
    },
    Negate {
        dest: Temp,
        ty: Type,
        value: Operand,
    },
    Not {
        dest: Temp,
        value: Operand,
    },
    Binary {
        dest: Temp,
        op: BinaryOp,
        ty: Type,
        left: Operand,
        right: Operand,
    },
    Compare {
        dest: Temp,
        op: CompareOp,
        operand_ty: Type,
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
    CallPrintBool {
        offset: Temp,
        scaled_offset: Temp,
        address: Temp,
        text: Temp,
        result: Temp,
        value: Operand,
    },
}
