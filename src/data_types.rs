use derive_more::From;
use derive_more::TryInto;

use std::sync::Arc;
use parking_lot::RwLock;

use crate::ffi_collections::CVec;

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
pub type SharkyByteString = CVec<SharkyByte>;

impl SharkyValue for SharkyMax {}
impl SharkyValue for SharkyInt {}
impl SharkyValue for SharkyReal {}
impl SharkyValue for SharkyByte {}
impl SharkyValue for SharkyBool {}
impl SharkyValue for SharkyByteString {}

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
    ByteString(SharkyByteString),
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
            SharkyDataType::ByteString(v) => {
                let mut formatted = String::new();
                let vec = v.get_operator();
                for val in vec.iter() {
                    formatted.push(*val as char);
                }
                write!(f, "")
            },
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



