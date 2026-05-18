use derive_more::From;
use derive_more::TryInto;

#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
pub struct SharkyHeapAddress {
    frame: usize,
    index: usize,
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

#[derive(Debug, Clone, PartialEq, PartialOrd, From, TryInto)]
pub enum SharkyDataType {
    Max(SharkyMax),
    Int(SharkyInt),
    Real(SharkyReal),
    Byte(SharkyByte),
    Bool(SharkyBool),
    HeapReference(SharkyHeapReference),
    Nil
}

impl std::fmt::Display for SharkyDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SharkyDataType::Max(v)            => write!(f, "{}", v),
            SharkyDataType::Int(v)            => write!(f, "{}", v),
            SharkyDataType::Real(v)           => write!(f, "{}", v),
            SharkyDataType::Byte(v)           => write!(f, "{}", v),
            SharkyDataType::Bool(v)           => write!(f, "{}", v),
            SharkyDataType::HeapReference(v)  => write!(f, "<ref {}:{}>", v.frame, v.index),
            SharkyDataType::Nil               => write!(f, "nil"),
        }
    }
}