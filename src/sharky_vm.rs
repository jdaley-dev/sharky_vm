use std::sync::Arc;

use crate::sharky_memory; 

enum SharkyStackMode {
    Indexed,
    Addressed,
    Operative,
    Native,
}

enum SharkyInstructionSet {
    StackMode(SharkyStackMode),
    SelectStack(usize),
    BottomStack,

    // push a constant value to the top of the stack
    ConstantPushMax(sharky_memory::SharkyMax),
    ConstantPushInt(sharky_memory::SharkyInt),
    ConstantPushReal(sharky_memory::SharkyReal),
    ConstantPushByte(sharky_memory::SharkyByte),
    ConstantPushBool(sharky_memory::SharkyBool),
    ConstantPushString(sharky_memory::SharkyString),
    ConstantPushHeapReference(sharky_memory::SharkyHeapAddress),
    ConstantPushNil,

    // memory operations
    CopyIndex(usize),
    NilIndex(usize),
    CopyToIndex((usize, usize)),
    Pop,

    // operative operations    
    PushOperative(usize),
    ClearOperative,
    PopOperative,
    Add((usize, usize)),
    Sub((usize, usize)),
    Multiply((usize, usize)),
    Divide((usize, usize)),
    BitLeftShift((usize, usize)),
    BitRightShift((usize, usize)),
    BitAnd((usize, usize)),
    BitXor((usize, usize)),
    BitOr((usize, usize)),
    BitNot(usize),
    Not(usize),
    And((usize, usize)),
    Or((usize, usize)),
    Equals((usize, usize)),
    NotEquals((usize, usize)),
    GreaterThan((usize, usize)),
    LesserThan((usize, usize)),
    GreaterThanOrEquals((usize, usize)),
    LesserThanOrEquals((usize, usize)),

    // functional operations
    Call(usize),
    Return,

    // thread operations
    SpawnThread(usize),
    Await(usize),

    // Logic operations
    Jump(usize),
    JumpIfNot((usize, usize)),
    PopJumpIfNot((usize, usize)),
    NoOperation,

    // Native operations
    PushNative(usize),
    ClaimNative(usize), // pushes a native index into the selected stack.
    ClearNative,
    CallNative(usize),
}

struct SharkyTask {
    stack_indices: Vec<usize>,
    return_task: usize,
}

struct SharkyVM {
    memory: Arc<sharky_memory::SharkyMemory>,
}