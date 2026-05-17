use std::sync::Arc;

use crate::sharky_vm::{SharkyInstruction};

mod sharky_memory;
mod sharky_vm;

fn main() {
    let program_arc: Arc<sharky_vm::SharkyProgram> = Arc::new(vec![
        SharkyInstruction::StackMode(sharky_vm::SharkyStackMode::Operative),
        SharkyInstruction::ConstantPushReal(2.0), 
        SharkyInstruction::ConstantPushReal(4.0), 
        SharkyInstruction::Add((0, 1)), 
        SharkyInstruction::Subtract((0, 1)),
        SharkyInstruction::Multiply((0, 1)), 
        SharkyInstruction::Divide((0, 1)),
        ]);

    let mut interpreter = sharky_vm::SharkyInterpreter::new(program_arc);
    interpreter.run();
    interpreter.get_operational_stack().debug_print_stack();
}
