use std::sync::Arc;

use crate::sharky_instruction_set::*;
use crate::sharky_vm::*;

mod sharky_memory;
mod sharky_instruction_set;
mod sharky_vm;

fn main() {
    let program_arc: Arc<SharkyProgram> = Arc::new(vec![
        SharkyInstruction::StackMode(SharkyStackMode::Operative),

        SharkyInstruction::Call(6),
        SharkyInstruction::Call(6),
        SharkyInstruction::Call(6),
        SharkyInstruction::Call(6),
        SharkyInstruction::Jump(100),

        SharkyInstruction::ConstantPushInt(2),
        SharkyInstruction::Return,
    ]);

    let mut interpreter = SharkyInterpreter::new(program_arc);
    interpreter.run();
    interpreter.get_operational_stack().debug_print_stack();
}
