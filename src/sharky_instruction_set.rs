use crate::sharky_memory;
#[derive(Default, Debug, Clone)]
pub enum SharkyStackMode {
    #[default]
    Indexed,
    Addressed,
    Operative,
    Native,
}

#[derive(Default, Debug, Clone)]
pub enum SharkyInstruction {
    StackMode(SharkyStackMode),
    SelectStack(usize),
    PushStack,
    PopStack,

    // push a constant value to the top of the stack
    ConstantPushMax(sharky_memory::SharkyMax),
    ConstantPushInt(sharky_memory::SharkyInt),
    ConstantPushReal(sharky_memory::SharkyReal),
    ConstantPushByte(sharky_memory::SharkyByte),
    ConstantPushBool(sharky_memory::SharkyBool),
    ConstantPushString(sharky_memory::SharkyString),
    ConstantPushHeapReference(sharky_memory::SharkyHeapAddress),
    ConstantPushNil,

    // stack operations
    Copy(usize),
    Nilify(usize),
    CopyTo((usize, usize)),
    Pop,
    Clear,

    // operative operations    
    CopyOperativeToStack, // Copies the top of the operative stack to the selected indexed stack.

    Add((usize, usize)),
    Subtract((usize, usize)),
    Multiply((usize, usize)),
    Divide((usize, usize)),
    Modulus((usize, usize)),
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
    PopJumpIfNot(usize),
    #[default]
    NoOperation,
    
    // Native operation
    CopyNativeResultToStack,
    CallNative(usize),
}

pub type SharkyProgram = Vec<SharkyInstruction>;