use sharky_env::{data_types::*, ffi_collections::*};

#[derive(Default, Debug, Clone)]
pub enum SharkyParameter<T> {
    #[default]
    None,
    Constant(T),
    StackIndex(usize),
}

type SharkyIndexParameter = SharkyParameter<usize>;

#[derive(Default, Debug, Clone)]
pub enum SharkyInstruction {
    #[default]
    NoOperation,

    SetStackMode(SharkyStackMode),
    SelectLocalStack(SharkyIndexParameter),
    PushLocalStack,
    PopLocalStack,

    // push a constant value to the top of the stack
    PushMax(SharkyParameter<SharkyMax>),
    PushInt(SharkyParameter<SharkyInt>),
    PushReal(SharkyParameter<SharkyReal>),
    PushByte(SharkyParameter<SharkyByte>),
    PushBool(SharkyParameter<SharkyBool>),
    PushHeapReference(SharkyParameter<SharkyHeapFrameIndex>),
    PushByteString(SharkyParameter<CVec<SharkyByte>>),
    PushNil,

    ToMax(SharkyIndexParameter),
    ToByte(SharkyIndexParameter),
    ToInt(SharkyIndexParameter),

    // stack operations
    PushTransition(SharkyIndexParameter),
    CopyTransition(SharkyIndexParameter),
    Copy(SharkyIndexParameter),
    Nilify(SharkyIndexParameter),
    Set((SharkyIndexParameter, SharkyIndexParameter)),
    Pop,
    Clear,

    // heap operations
    CreateHeap,
    CloneHeap,
    SizeHeap,
    PushHeap,
    SelectHeap(SharkyIndexParameter),
    WriteHeap((SharkyIndexParameter, SharkyIndexParameter)),
    ReadHeap((SharkyIndexParameter, SharkyIndexParameter)),

    // All operations are a OP b
    Add((SharkyIndexParameter, SharkyIndexParameter)),
    Subtract((SharkyIndexParameter, SharkyIndexParameter)),
    Multiply((SharkyIndexParameter, SharkyIndexParameter)),
    Divide((SharkyIndexParameter, SharkyIndexParameter)),
    Modulus((SharkyIndexParameter, SharkyIndexParameter)),
    BitLeftShift((SharkyIndexParameter, SharkyIndexParameter)),
    BitRightShift((SharkyIndexParameter, SharkyIndexParameter)),
    BitAnd((SharkyIndexParameter, SharkyIndexParameter)),
    BitXor((SharkyIndexParameter, SharkyIndexParameter)),
    BitOr((SharkyIndexParameter, SharkyIndexParameter)),
    BitNot(SharkyIndexParameter),
    Not(SharkyIndexParameter),
    And((SharkyIndexParameter, SharkyIndexParameter)),
    Or((SharkyIndexParameter, SharkyIndexParameter)),
    Equals((SharkyIndexParameter, SharkyIndexParameter)),
    NotEquals((SharkyIndexParameter, SharkyIndexParameter)),
    GreaterThan((SharkyIndexParameter, SharkyIndexParameter)),
    LesserThan((SharkyIndexParameter, SharkyIndexParameter)),
    GreaterThanOrEquals((SharkyIndexParameter, SharkyIndexParameter)),
    LesserThanOrEquals((SharkyIndexParameter, SharkyIndexParameter)),

    // Logic operations
    Jump(SharkyIndexParameter),
    // conditional jumps are (to, condition)
    JumpIfNot((SharkyIndexParameter, SharkyIndexParameter)),
    // popjumpifnot checks if the top of the stack is false then if false it pops it, and jumps to the location
    PopJumpIfNot(SharkyIndexParameter),

    // Thread operations
    SpawnSubtask(SharkyIndexParameter),
    EndTask,

    // ffi ops
    FFICall(SharkyIndexParameter),
}

pub type SharkyProgram = Vec<SharkyInstruction>;