use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use crate::sharky_app::SharkyApp;
use crate::sharky_instruction_set::*;
use crate::sharky_string::SharkyStringPool;
use crate::sharky_vm::*;
use crate::sharky_native::*;

mod sharky_native;
mod sharky_string;
mod sharky_data_types;
mod sharky_memory;
mod sharky_instruction_set;
mod sharky_vm;
mod sharky_app;

fn main() {

    if let Some(mut natives) = SharkyNativeLibrary::load_library("test_lib", Path::new("C:\\Users\\jdale\\Working\\_CC\\test_lib\\x64\\Release\\test_lib.dll")) {
        if let Some(index) = natives.load_symbol("add") {
            if let Some(result) = natives.call(index, vec![1920i64.into(), 1080i64.into()]) {
                println!("Result: {result}");
            }
        }
    }

    let mut test_strings = SharkyStringPool::new();
    let index = test_strings.create_string("hello".as_bytes());
    let string = std::str::from_utf8(test_strings.get_slice(index).unwrap()).unwrap();
    println!("Saved String: {string}");

    let program_arc: Arc<SharkyProgram> = Arc::new(vec![
        SharkyInstruction::CreateDynamicHeap, // 0: Heap reference to top of stack.
    ]);

    SharkyApp::init(program_arc);
}
