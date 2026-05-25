#![allow(dead_code)]

use crate::{sharky_data_types::*};
#[derive(Default, Debug, Clone)]
pub enum SharkyStackMode {
    #[default]
    Indexed,
    Addressed,
    Operative,
    Native,
    Parameter,
    String,
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

    // heap operations
    CreateDynamicHeap,                                       /// creates a dynamic heap (any type, memory not flat.) address is pushed to the top of the stack.
    CreateByteHeap,                                          /// creates a byte heap (bytes only, memory flat and contiguous) address is pushed to the top of the stack.
    CreateIntHeap,                                           /// creates an int heap (ints only, memory flat and contiguous) address is pushed to the top of the stack.
    CreateMaxHeap,                                           /// creates a max heap (max's only, memory flat and contiguous) address is pushed to the top of the stack.
    CreateRealHeap,                                          /// creates a real heap (reals only, memory flat and contiguous) address is pushed to the top of the stack.
    ReadHeap((SharkyIndexParameter, SharkyIndexParameter)),  /// Reads heap address (param_b) to the selected stack address (param_a)
    WriteHeap((SharkyIndexParameter, SharkyIndexParameter)), /// Writes to heap address (param_a) from the selected stack address (param_b)
    PushHeap((SharkyIndexParameter, SharkyIndexParameter)),  /// Pushes to heap (param_a) from the selected stack address (param_b)
    DeleteHeap(SharkyIndexParameter),                        /// Deletes heap (param_a)
    CloneHeap(SharkyIndexParameter),                         /// Clones heap (param_a) address is pushed to the top of the stack.
    SliceHeap((SharkyIndexParameter, SharkyIndexParameter)), /// Clones heap from (param_a) to (param_b) to a new heap. address is pushed to the top of the stack.
    SizeHeap(SharkyIndexParameter),                          /// Reads the size of the heap (param_a) to the top of the stack

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
}

pub type SharkyProgram = Vec<SharkyInstruction>;