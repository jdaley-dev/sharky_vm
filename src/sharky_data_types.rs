use derive_more::From;
use derive_more::TryInto;

#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[repr(C)]
pub struct SharkyHeapAddress {
    pub frame: usize,
    pub index: usize,
}

pub trait SharkyValue {}

pub type SharkyMax = usize;
pub type SharkyInt = i64;
pub type SharkyReal = f64;
pub type SharkyByte = u8;
pub type SharkyBool = bool;
pub type SharkyHeapReference = SharkyHeapAddress;

impl SharkyValue for SharkyMax {}
impl SharkyValue for SharkyInt {}
impl SharkyValue for SharkyReal {}
impl SharkyValue for SharkyByte {}
impl SharkyValue for SharkyBool {}
impl SharkyValue for SharkyHeapReference {}

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
    HeapReference(SharkyHeapReference),
}

impl std::fmt::Display for SharkyDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SharkyDataType::Max(v)            => write!(f, "Max({})", v),
            SharkyDataType::Int(v)            => write!(f, "Int({})", v),
            SharkyDataType::Real(v)           => write!(f, "Real({})", v),
            SharkyDataType::Byte(v)           => write!(f, "Byte({})", v),
            SharkyDataType::Bool(v)           => write!(f, "Bool({})", v),
            SharkyDataType::HeapReference(v)  => write!(f, "<ref {}:{}>", v.frame, v.index),
            SharkyDataType::Nil               => write!(f, "nil"),
        }
    }
}