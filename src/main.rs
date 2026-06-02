use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::sharky_app::SharkyApp;
use crate::sharky_memory::SharkyDataStack;
use crate::sharky_string::SharkyStringPool;
use crate::sharky_native::*;
use crate::sharky_data_types::*;

mod sharky_native;
mod sharky_string;
mod sharky_data_types;
mod sharky_memory;
mod sharky_vm;
mod sharky_app;

use parking_lot::deadlock;

fn main() {

    let mut library_pool = SharkyFFIPool::new();
    let library = library_pool.load_library("C:\\Users\\jdale\\Working\\_CC\\test_lib\\x64\\Release\\test_lib.dll").unwrap();
    let function_id = library_pool.load_function(library, "print").unwrap();

    let program_arc: Arc<SharkyProgram> = Arc::new(vec![
        SharkyInstruction::SetStackMode(SharkyStackMode::Parameter),
        SharkyInstruction::PushMax(SharkyParameter::Constant(6)),
        SharkyInstruction::PushByte(SharkyParameter::Constant('S' as u8)),
        SharkyInstruction::PushByte(SharkyParameter::Constant('h' as u8)),
        SharkyInstruction::PushByte(SharkyParameter::Constant('a' as u8)),
        SharkyInstruction::PushByte(SharkyParameter::Constant('r' as u8)),
        SharkyInstruction::PushByte(SharkyParameter::Constant('k' as u8)),
        SharkyInstruction::PushByte(SharkyParameter::Constant('!' as u8)),
        SharkyInstruction::FFICall(SharkyParameter::Constant(0)),
        SharkyInstruction::KillSelf,
    ]);
    
    let mut test_frame = SharkyDataStack::default();
    test_frame.push(SharkyDataType::Byte('H' as u8));
    test_frame.push(SharkyDataType::Byte('e' as u8));
    test_frame.push(SharkyDataType::Byte('l' as u8));
    test_frame.push(SharkyDataType::Byte('l' as u8));
    test_frame.push(SharkyDataType::Byte('o' as u8));
    test_frame.push(SharkyDataType::Byte(' ' as u8));
    test_frame.push(SharkyDataType::Byte('S' as u8));
    test_frame.push(SharkyDataType::Byte('h' as u8));
    test_frame.push(SharkyDataType::Byte('a' as u8));
    test_frame.push(SharkyDataType::Byte('r' as u8));
    test_frame.push(SharkyDataType::Byte('k' as u8));
    test_frame.push(SharkyDataType::Byte('y' as u8));
    test_frame.push(SharkyDataType::Byte('!' as u8));

    SharkyApp::init(program_arc, vec![test_frame], library_pool);
}
