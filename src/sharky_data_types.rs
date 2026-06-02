use derive_more::From;
use derive_more::TryInto;

use std::sync::Arc;
use parking_lot::RwLock;

pub type SharkySynced<T> = Arc<RwLock<T>>;

pub trait SharkyValue {}

pub type SharkyHeapFrameIndex = usize;
pub type SharkyHeapCellIndex = usize;
pub type SharkyBytePoolIndex = usize;
pub type SharkyMax = usize;
pub type SharkyInt = i64;
pub type SharkyReal = f64;
pub type SharkyByte = u8;
pub type SharkyBool = bool;

impl SharkyValue for SharkyMax {}
impl SharkyValue for SharkyInt {}
impl SharkyValue for SharkyReal {}
impl SharkyValue for SharkyByte {}
impl SharkyValue for SharkyBool {}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, From, TryInto)]
#[repr(C, u8)]
pub enum SharkyDataType {
    #[default]
    Nil,
    Max(SharkyMax),
    Int(SharkyInt),
    Real(SharkyReal),
    Byte(SharkyByte),
    Bool(SharkyBool),
    #[from(ignore)]
    #[try_into(ignore)]
    HeapReference(SharkyHeapFrameIndex),
}

impl std::fmt::Display for SharkyDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SharkyDataType::Max(v)            => write!(f, "Max({})", v),
            SharkyDataType::Int(v)            => write!(f, "Int({})", v),
            SharkyDataType::Real(v)           => write!(f, "Real({})", v),
            SharkyDataType::Byte(v)           => write!(f, "Byte({})", v),
            SharkyDataType::Bool(v)           => write!(f, "Bool({})", v),
            SharkyDataType::HeapReference(v)  => write!(f, "Ref({})", v),
            SharkyDataType::Nil               => write!(f, "nil"),
        }
    }
}

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
    KillSelf,

    // ffi ops
    FFICall(SharkyIndexParameter),
}

pub type SharkyProgram = Vec<SharkyInstruction>;