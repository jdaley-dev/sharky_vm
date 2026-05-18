use std::sync::Arc;

use crate::sharky_data_types::*;
use crate::sharky_instruction_set::*;
use crate::sharky_vm::*;

mod sharky_data_types;
mod sharky_memory;
mod sharky_instruction_set;
mod sharky_vm;

fn main() {
    let program_arc: Arc<SharkyProgram> = Arc::new(vec![
        
        // load into the global stack
        SharkyInstruction::StackMode(SharkyStackMode::Indexed),
        SharkyInstruction::PushInt(SharkyParameter::Constant(4)),
        SharkyInstruction::PushInt(SharkyParameter::Constant(7)),

        // copy value into the transition stack
        SharkyInstruction::PushTransition(SharkyParameter::Constant(0)),
        SharkyInstruction::PushTransition(SharkyParameter::Constant(1)),
        
        // clear the transitional stack
        SharkyInstruction::StackMode(SharkyStackMode::Operative),
        SharkyInstruction::CopyTransition(SharkyParameter::Constant(0)),
        SharkyInstruction::CopyTransition(SharkyParameter::Constant(1)),
        SharkyInstruction::Add((SharkyParameter::Constant(0), SharkyParameter::Constant(1))),
        SharkyInstruction::PushTransition(SharkyParameter::Constant(2)),

        SharkyInstruction::StackMode(SharkyStackMode::Indexed),
        SharkyInstruction::CopyTransition(SharkyParameter::Constant(2)),
        SharkyInstruction::Set((SharkyParameter::Constant(0), SharkyParameter::Constant(2))),
        SharkyInstruction::Pop, 
        SharkyInstruction::Pop, 

        // clear the transitional stack for future work
        SharkyInstruction::StackMode(SharkyStackMode::Transitional),
        SharkyInstruction::Clear,
    ]);

    let mut interpreter = SharkyInterpreter::new(program_arc);
    interpreter.run();
    if let Some(stack) = interpreter.get_current_stack() {
        stack.debug_print_stack();
    }
}
