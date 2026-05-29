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
        
        // load into the global stack
        SharkyInstruction::SetStackMode(SharkyStackMode::Indexed),
        SharkyInstruction::PushInt(SharkyParameter::Constant(123)),
        SharkyInstruction::PushInt(SharkyParameter::Constant(414)),

        // copy value into the transition stack
        SharkyInstruction::PushTransition(SharkyParameter::Constant(0)),
        SharkyInstruction::PushTransition(SharkyParameter::Constant(1)),
        
        SharkyInstruction::SetStackMode(SharkyStackMode::Operative),
        SharkyInstruction::CopyTransition(SharkyParameter::Constant(0)),
        SharkyInstruction::CopyTransition(SharkyParameter::Constant(1)),
        SharkyInstruction::Add((SharkyParameter::Constant(0), SharkyParameter::Constant(1))),
        SharkyInstruction::PushTransition(SharkyParameter::Constant(2)),

        SharkyInstruction::SetStackMode(SharkyStackMode::Indexed),
        SharkyInstruction::CopyTransition(SharkyParameter::Constant(2)),
        SharkyInstruction::Set((SharkyParameter::Constant(0), SharkyParameter::Constant(2))),
        SharkyInstruction::Pop, 
        SharkyInstruction::Pop, 
        
        // clear the transitional stack for future work.
        SharkyInstruction::SetStackMode(SharkyStackMode::Transitional),
        SharkyInstruction::Clear,
    ]);


    let sharky_app = SharkyApp::new_arc();
    SharkyApp::start_garbage_collector(sharky_app.clone());
    for _ in 0..100 {
        sharky_app.write().unwrap().spawn_interpreter(Arc::clone(&program_arc));
    }
    SharkyApp::await_processes(sharky_app);
}
