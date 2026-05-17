use std::sync::Arc;

use crate::sharky_instruction_set::*;
use crate::sharky_vm::*;

mod sharky_memory;
mod sharky_instruction_set;
mod sharky_vm;

fn main() {
    let program_arc: Arc<SharkyProgram> = Arc::new(vec![
        // 0
        SharkyInstruction::StackMode(SharkyStackMode::Operative), 

        // 1
        SharkyInstruction::ConstantPushInt(0), // counter (0)
        // 2
        SharkyInstruction::ConstantPushInt(1), // increment by (1)
        // 3
        SharkyInstruction::ConstantPushInt(10), // goal (2)
        // 4
        SharkyInstruction::ConstantPushBool(false), // checker (3)

        // 5
        SharkyInstruction::Add((0, 1)), // addition (4)
        // 6
        SharkyInstruction::CopyTo((0, 4)),
        // 7
        SharkyInstruction::Pop, // POPS ADD

        // 8
        SharkyInstruction::Equals((0, 2)), // comparison (4)
        // 9
        SharkyInstruction::CopyTo((3, 4)),
        // 10
        SharkyInstruction::Pop, // POPS NOT EQUALS
        // 11
        SharkyInstruction::JumpIfNot((5, 3)),

        ]);

    let mut interpreter = SharkyInterpreter::new(program_arc);
    interpreter.run();
    interpreter.get_operational_stack().debug_print_stack();
}
