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
    // All copy operations are (dest, src)
    CopyTo((usize, usize)),
    CopySlotTo((usize, usize)),
    CopySlotToSlot((usize, usize)),
    CopyToSlot((usize, usize)),
    Pop,
    Clear,

    // operative operations    
    CopyOperativeToStack, // Copies the top of the operative stack to the selected indexed stack.


    // All operations are a OP b
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

    // Logic operations
    Jump(usize),
    // conditional jumps are (to, condition)
    JumpIfNot((usize, usize)),
    // popjumpifnot checks if the top of the stack is false then if false it pops it, and jumps to the location
    PopJumpIfNot(usize),
    #[default]
    NoOperation,
    
    // functional operations
    Call(usize),
    Return,

    // thread operations
    SpawnThread(usize),
    Await(usize),



    // Native operation
    CopyNativeResultToStack,
    CallNative(usize),
}

pub type SharkyProgram = Vec<SharkyInstruction>;