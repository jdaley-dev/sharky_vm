use std::sync::Arc;

use crate::sharky_instruction_set::*;
use crate::sharky_vm::*;

mod sharky_memory;
mod sharky_instruction_set;
mod sharky_vm;

fn main() {
    let program_arc: Arc<SharkyProgram> = Arc::new(vec![
        // set operative mode
        SharkyInstruction::StackMode(SharkyStackMode::Operative),

        // base values we'll reuse
        SharkyInstruction::ConstantPushInt(10),     // slot 0
        SharkyInstruction::ConstantPushInt(3),      // slot 1

        // arithmetic
        SharkyInstruction::Add((0, 1)),             // slot 2  = 13
        SharkyInstruction::Subtract((0, 1)),        // slot 3  = 7
        SharkyInstruction::Multiply((0, 1)),        // slot 4  = 30
        SharkyInstruction::Divide((0, 1)),          // slot 5  = 3
        SharkyInstruction::Modulus((0, 1)),         // slot 6  = 1

        // bitwise
        SharkyInstruction::BitAnd((0, 1)),          // slot 7  = 0b00000010 = 2
        SharkyInstruction::BitOr((0, 1)),           // slot 8  = 0b00001011 = 11
        SharkyInstruction::BitXor((0, 1)),          // slot 9  = 0b00001001 = 9
        SharkyInstruction::BitLeftShift((0, 1)),    // slot 10 = 10 << 3 = 80
        SharkyInstruction::BitRightShift((0, 1)),   // slot 11 = 10 >> 3 = 1
        SharkyInstruction::BitNot(0),               // slot 12 = !10 = -11

        // comparisons — all use original slot 0 (10) and slot 1 (3)
        SharkyInstruction::Equals((0, 1)),          // slot 13 = false
        SharkyInstruction::NotEquals((0, 1)),       // slot 14 = true
        SharkyInstruction::GreaterThan((0, 1)),     // slot 15 = true
        SharkyInstruction::LesserThan((0, 1)),      // slot 16 = false
        SharkyInstruction::GreaterThanOrEquals((0, 1)), // slot 17 = true
        SharkyInstruction::LesserThanOrEquals((0, 1)),  // slot 18 = false

        // bool logic — push fresh bools
        SharkyInstruction::ConstantPushBool(true),  // slot 19
        SharkyInstruction::ConstantPushBool(false), // slot 20
        SharkyInstruction::And((19, 20)),           // slot 21 = false
        SharkyInstruction::Or((19, 20)),            // slot 22 = true
        SharkyInstruction::Not(19),                 // slot 23 = false
        SharkyInstruction::Not(20),                 // slot 24 = true

        // stack ops
        SharkyInstruction::Copy(0),                 // slot 25 = 10 (copy of slot 0)
        SharkyInstruction::CopyTo((25, 1)),         // slot 25 = 3  (overwrite with slot 1)
        SharkyInstruction::Nilify(25),              // slot 25 = Nil
        SharkyInstruction::Pop                     // removes slot 25
        ]);

    let mut interpreter = sharky_vm::SharkyInterpreter::new(program_arc);
    interpreter.run();
    interpreter.get_operational_stack().debug_print_stack();
}
