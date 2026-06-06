use sharky_env::{data_types::*, ffi_collections::*};

#[derive(Debug, Clone)]
pub enum ArithmeticMode {
    Add = 0,
    Subtract = 1,
    Multiply = 2,
    Divide = 3,
    Mod = 4,
}

#[derive(Debug, Clone)]
pub enum BitwiseMode {
    ShiftLeft = 0,
    ShiftRight = 1,
    And = 2,
    Or = 3,
    Xor = 4,
    Not = 5,
}

#[derive(Debug, Clone)]
pub enum ComparisonMode {
    And = 0,
    Or = 1,
    Equals = 2,
    NotEquals = 3,
    GreaterThan = 4,
    LesserThan = 5,
    GreaterThanOrEquals = 6,
    LesserThanOrEquals = 7,
}

#[derive(Debug, Clone)]
pub enum TypeMode {
    Max = 0,
    Int = 1,
    Real = 2,
    Byte = 3,
    Bool = 4,
    HeapReference = 5,
    ByteString = 6,
    Nil = 7,
}

#[derive(Debug, Clone)]
pub enum LogicMode {
    If = 0,
    IfNot = 1,
    PopIf = 2,
}

#[derive(Debug, Clone)]
pub enum SelectStackMode {
    Fixed = 0,
    Indexed = 1,
}

#[derive(Debug, Clone)]
pub enum FixedStackMode {
    Operative = 0,
    Transitional = 1,
    Parameter = 2,
}

#[derive(Debug, Clone)]
pub enum HeapOpMode {
    New = 0,
    Clone = 1,
}

#[allow(unused)]
#[derive(Default, Debug, Clone)]
pub enum OpParameter {
    #[default]
    None,
    Constant(SharkyDataType),
    StackIndex(usize),
}

#[allow(unused)]
#[derive(Default, Debug, Clone)]
pub enum SharkyInstruction {
    #[default]
    NoOperation,

    SelectStack(OpParameter, SelectStackMode),
    PushStack,
    PopStack,

    Push(OpParameter, TypeMode),

    Convert(OpParameter, TypeMode),

    // stack operations
    CopyFromTransition(OpParameter),
    CopyToTransition(OpParameter),

    Copy(OpParameter),
    Nilify(OpParameter),
    Set((OpParameter, OpParameter)),
    Pop,
    Clear,

    // heap operations
    Heap(HeapOpMode),
    GetHeapSize,
    NewHeapItem,
    SelectHeap(OpParameter),
    CopyToHeapItem(OpParameter, OpParameter),
    CopyFromHeapItem(OpParameter, OpParameter),

    // bytestring operations
    PushBytes(OpParameter, OpParameter, OpParameter),
    SetByte((OpParameter, OpParameter, OpParameter)),
    ReadByteAs((OpParameter, OpParameter, TypeMode)),

    BinaryOp(OpParameter, OpParameter, ArithmeticMode),

    Goto(OpParameter),
    LogicalGoto((OpParameter, OpParameter, LogicMode)),

    // Thread operations
    SpawnSubtask(OpParameter),
    EndTask,

    // ffi ops
    FFICall(OpParameter),
}

pub type SharkyProgram = Vec<SharkyInstruction>;
