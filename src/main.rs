use std::sync::Arc;

use sharky_env::data_types::*;
use sharky_env::ffi::*;
use sharky_env::ffi_collections::*;

use crate::app::*;
use crate::instructions::*;

mod app;
mod bytecode_reader;
mod instructions;
mod vm;

fn main() {
    let mut library_pool = SharkyFFIPool::new();
    let library = library_pool
        .load_library("C:\\Users\\jdale\\Working\\_CC\\test_lib\\x64\\Release\\test_lib.dll")
        .unwrap();
    library_pool.load_function(library, "print").unwrap(); // id 0. sequential.

    let mut sharky_string = CVec::<SharkyByte>::new();
    let rust_string = String::from(
        "Sharky! FROM A BYTESTRING\nThis is a true FFI interaction between sharky and a C library.",
    );
    sharky_string.operate(|vec| {
        vec.resize(rust_string.len(), '\0' as u8);
        vec.clone_from_slice(rust_string.as_bytes());
    });

    let program_arc: Arc<SharkyProgram> = Arc::new(vec![
        // no op
        SharkyInstruction::NoOperation,
        // stack ops
        SharkyInstruction::SelectStack(
            OpParameter::Constant(SharkyDataType::Max(0)),
            SelectStackMode::Indexed,
        ), // select indexed stack 0
        SharkyInstruction::Push(SharkyDataType::Real(2.4)),
        SharkyInstruction::Push(SharkyDataType::Int(SharkyInt::MIN)),
        SharkyInstruction::Push(SharkyDataType::Max(SharkyMax::MAX)),
        SharkyInstruction::Push(SharkyDataType::ByteString(sharky_string)),
    ]);

    SharkyApp::init(program_arc, vec![], library_pool);
}
