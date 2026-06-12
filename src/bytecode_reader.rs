use std::{
    fs::File,
    io::{BufReader, Seek},
    path::Path,
};

use sharky_env::simplex_file_read::{SimplexReadable, simplex_le_read};

#[repr(u16)]
pub enum SharkyOpCode {
    NoOperation = 0,
    SelectStack = 1,
    PushStack = 2,
    PopStack = 3,
    CopyToTransition = 4,
    CopyFromTransition = 5,
    Push = 6,
    Convert = 7,
    Copy = 8,
    Nilify = 9,
    Set = 10,
    Pop = 11,
    Clear = 12,
    Heap = 13,
    SelectHeap = 14,
    CopyToHeapItem = 15,
    CopyFromHeapItem = 16,
    FillByteStringWithValue = 17,
    ExtendByteString = 18,
    CopyToByteString = 19,
    CopyFromByteString = 20,
    AppendByteString = 21,
    ClearByteString = 22,
    ByteStringSize = 23,
    SliceByteString = 24,
    ArithmeticOp = 25,
    BitwiseOp = 26,
    ComparisonOp = 27,
    NotBool = 28,
    Goto = 29,
    LogicalGoto = 30,
    SpawnSubtask = 31,
    EndTask = 32,
    FFICall = 33,
}

#[repr(u8)]
pub enum SharkyParamType {
    Constant,
    Index,
    Pointer,
}

#[repr(u8)]
pub enum SharkyTypeCode {
    Nil,
    Max,
    Int,
    Byte,
    Real,
    Bool,
    HeapReferencePtr,
    ByteStringPtr,
}

pub type SharkyByteSlice = [u8; 128];

pub struct SharkyLibrarySymbol(String, String);

pub struct SharkyConstant(SharkyTypeCode, SharkyByteSlice);

pub struct SharkyParameter(SharkyOpCode, SharkyParamType, SharkyByteSlice);

pub struct SharkyBytecodeHeader {
    magic: u32,
    version: u32,
    symbol_offset: u64,
    string_offset: u64,
    globals_offset: u64,
    program_offset: u64,
    end_offset: u64,
}

impl SharkyBytecodeHeader {
    fn from<T: Seek, Read>(reader: BufReader<T>) -> Self {}
}

pub struct SharkyBytecodeConstants {
    symbols: Vec<SharkyLibrarySymbol>,
    strings: Vec<String>,
    constants: Vec<SharkyConstant>,
}

pub struct SharkyBytecodeInstruction {
    code: SharkyOpCode,
    parameters: Vec<SharkyParameter>,
}

pub struct SharkyBytecodeProgram {
    header: SharkyBytecodeHeader,
    constants: SharkyBytecodeConstants,
    instructions: Vec<SharkyBytecodeInstruction>,
}

impl SharkyBytecodeProgram {
    fn load_file(path: &Path) -> Option<Self> {
        let file = File::open(path).ok()?;
        let mut reader = BufReader::new(file);

        let magic: u32 = simplex_le_read::<u32, File>(&mut reader)?;
        let header = SharkyBytecodeHeader {
            magic,
            version: todo!(),
            symbol_offset: todo!(),
            string_offset: todo!(),
            globals_offset: todo!(),
            program_offset: todo!(),
            end_offset: todo!(),
        };
        None
    }
}
