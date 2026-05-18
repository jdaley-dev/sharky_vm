use crate::{sharky_data_types::*};
#[derive(Default, Debug, Clone)]
pub enum SharkyStackMode {
    #[default]
    Indexed,
    Addressed,
    Operative,
    Native,
    Parameter,
    Transitional,
}

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
    StackMode(SharkyStackMode),
    SelectStack(SharkyIndexParameter),
    PushStack,
    PopStack,

    // push a constant value to the top of the stack
    PushMax(SharkyParameter<SharkyMax>),
    PushInt(SharkyParameter<SharkyInt>),
    PushReal(SharkyParameter<SharkyReal>),
    PushByte(SharkyParameter<SharkyByte>),
    PushBool(SharkyParameter<SharkyBool>),
    PushHeapReference(SharkyParameter<SharkyHeapAddress>),
    PushNil,

    ToMax(SharkyIndexParameter),
    ToByte(SharkyIndexParameter),
    ToInt(SharkyIndexParameter),

    // stack operations
    PushTransition(SharkyIndexParameter),
    CopyTransition(SharkyIndexParameter),
    Copy(SharkyIndexParameter),
    Nilify(SharkyIndexParameter),
    // All copy operations are (dest, src)
    Set((SharkyIndexParameter, SharkyIndexParameter)),
    Pop,
    Clear,

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
    #[default]
    NoOperation,

    // thread operations
    SpawnThread(SharkyIndexParameter),
    Await(SharkyIndexParameter),

    // Native operation
    CopyNativeResultToStack,
    CallNative(SharkyIndexParameter),
}

pub type SharkyProgram = Vec<SharkyInstruction>;