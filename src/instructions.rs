use derive_more::derive::TryInto;
use num_enum::TryFromPrimitive;
use sharky_env::data_types::*;

#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum ArithmeticMode {
    Add = 0,
    Subtract = 1,
    Multiply = 2,
    Divide = 3,
    Mod = 4,
}

#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum BitwiseMode {
    ShiftLeft = 0,
    ShiftRight = 1,
    And = 2,
    Or = 3,
    Xor = 4,
    Not = 5,
}

#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum ComparisonMode {
    And = 0,
    Or = 1,
    Equals = 2,
    NotEquals = 3,
    GreaterThan = 4,
    LessThan = 5,
    GreaterThanOrEquals = 6,
    LessThanOrEquals = 7,
}

#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum ConversionMode {
    Max = 0,
    Int = 1,
    Real = 2,
    Byte = 3,
    Bool = 4,
    HeapReference = 5,
    ByteString = 6,
    Nil = 7,
}

#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum LogicMode {
    If = 0,
    IfNot = 1,
}

#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum SelectStackMode {
    Fixed = 0,
    Indexed = 1,
}

#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum FixedStackMode {
    Operative = 0,
    Transitional = 1,
    Parameter = 2,
}

#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum HeapOpMode {
    Create = 0,
    Clone = 1,
    NewItem = 2,
    Size,
}

#[allow(unused)]
#[derive(Default, Debug, Clone)]
pub enum OpParameter {
    #[default]
    None,
    Constant(SharkyDataType), // A constant value
    Index(usize),             // The value at a stack index
    Pointer(usize),           // The value of an index at a stack index
}

#[allow(unused)]
#[derive(Default, Debug, Clone)]
pub enum SharkyInstruction {
    #[default]
    NoOperation,

    // stack_index, mode
    SelectStack(OpParameter, SelectStackMode),
    // add stack frame
    PushStack,
    // remove stack frame
    PopStack,

    // copy value from [selected_stack] and push to the transition stack
    CopyToTransition(OpParameter),
    // copy value from the transition stack and push to [selected_stack]
    CopyFromTransition(OpParameter),
    // push value (constant or index into [selected_stack]) to [selected_stack]
    Push(SharkyDataType),
    // convert value in [selected_stack] to different type
    Convert(OpParameter, ConversionMode),
    // copy value from [selected_stack] and push to [selected_stack]
    Copy(OpParameter),
    // set value in [selected_stack] to nil
    Nilify(OpParameter),
    // set value[0] in [selected_stack] to value[1] in [selected_stack]
    Set(OpParameter, OpParameter),
    // pop value from [selected_stack]
    Pop,
    // clear [selected_stack]
    Clear,

    // perform a heap op
    Heap(HeapOpMode),
    SelectHeap(OpParameter),
    CopyToHeapItem(OpParameter, OpParameter),
    CopyFromHeapItem(OpParameter, OpParameter),

    // dest, value
    FillByteStringWithValue(OpParameter, OpParameter),
    // dest, length
    ExtendByteString(OpParameter, OpParameter),
    // dest, src
    CopyToByteString(OpParameter, OpParameter),
    // dest, src
    CopyFromByteString(OpParameter, OpParameter),
    // dest, src
    AppendByteString(OpParameter, OpParameter),
    // input
    ClearByteString(OpParameter),
    // input
    ByteStringSize(OpParameter),
    // src, begin, end
    SliceByteString(OpParameter, OpParameter, OpParameter),

    ArithmeticOp(OpParameter, OpParameter, ArithmeticMode),
    BitwiseOp(OpParameter, OpParameter, BitwiseMode),
    ComparisonOp(OpParameter, OpParameter, ComparisonMode),
    NotBool(OpParameter),

    Goto(OpParameter),
    LogicalGoto(OpParameter, OpParameter, LogicMode),

    SpawnSubtask(OpParameter),
    EndTask,

    // ffi ops
    FFICall(OpParameter),
}

pub type SharkyProgram = Vec<SharkyInstruction>;
