use std::sync::Arc;

use crate::sharky_instruction_set::*;
use crate::sharky_vm::*;

mod sharky_memory;
mod sharky_instruction_set;
mod sharky_vm;

fn main() {
    let program_arc: Arc<SharkyProgram> = Arc::new(vec![
        // instruction 0
    SharkyInstruction::StackMode(SharkyStackMode::Operative),

    // slot 0 = a
    // slot 1 = b
    // slot 2 = counter (starts at 5, destination index)
    // slot 3 = limit (20)
    // slot 4 = bool scratch
    // slots 5-19 = results

    // instruction 1
    SharkyInstruction::ConstantPushInt(0),       // slot 0 = a
    // instruction 2
    SharkyInstruction::ConstantPushInt(1),       // slot 1 = b
    // instruction 3
    SharkyInstruction::ConstantPushInt(5),       // slot 2 = counter
    // instruction 4
    SharkyInstruction::ConstantPushInt(20),      // slot 3 = limit
    // instruction 5
    SharkyInstruction::ConstantPushBool(false),  // slot 4 = bool scratch

    // pre-allocate result slots 5-19
    // instruction 6
    SharkyInstruction::ConstantPushInt(0),       // slot 5
    // instruction 7
    SharkyInstruction::ConstantPushInt(0),       // slot 6
    // instruction 8
    SharkyInstruction::ConstantPushInt(0),       // slot 7
    // instruction 9
    SharkyInstruction::ConstantPushInt(0),       // slot 8
    // instruction 10
    SharkyInstruction::ConstantPushInt(0),       // slot 9
    // instruction 11
    SharkyInstruction::ConstantPushInt(0),       // slot 10
    // instruction 12
    SharkyInstruction::ConstantPushInt(0),       // slot 11
    // instruction 13
    SharkyInstruction::ConstantPushInt(0),       // slot 12
    // instruction 14
    SharkyInstruction::ConstantPushInt(0),       // slot 13
    // instruction 15
    SharkyInstruction::ConstantPushInt(0),       // slot 14
    // instruction 16
    SharkyInstruction::ConstantPushInt(0),       // slot 15
    // instruction 17
    SharkyInstruction::ConstantPushInt(0),       // slot 16
    // instruction 18
    SharkyInstruction::ConstantPushInt(0),       // slot 17
    // instruction 19
    SharkyInstruction::ConstantPushInt(0),       // slot 18
    // instruction 20
    SharkyInstruction::ConstantPushInt(0),       // slot 19

    // --- LOOP START = instruction 21 ---

    // result[counter] = a
    // instruction 21
    SharkyInstruction::CopyToSlot((2, 0)),       // slot[counter] = a

    // temp = a + b
    // instruction 22
    SharkyInstruction::Add((0, 1)),              // slot 20 = temp

    // a = b
    // instruction 23
    SharkyInstruction::CopyTo((0, 1)),           // a = b

    // b = temp
    // instruction 24
    SharkyInstruction::CopyTo((1, 20)),          // b = temp

    // pop temp
    // instruction 25
    SharkyInstruction::Pop,                      // pop slot 20

    // counter = counter + 1
    // instruction 26
    SharkyInstruction::ConstantPushInt(1),       // slot 20 = 1
    // instruction 27
    SharkyInstruction::Add((2, 20)),             // slot 21 = counter + 1
    // instruction 28
    SharkyInstruction::CopyTo((2, 21)),          // counter = result
    // instruction 29
    SharkyInstruction::Pop,                      // pop slot 21
    // instruction 30
    SharkyInstruction::Pop,                      // pop slot 20

    // check counter >= limit (RISC style — jump when false)
    // instruction 31
    SharkyInstruction::GreaterThanOrEquals((2, 3)), // slot 20 = bool
    // instruction 32
    SharkyInstruction::CopyTo((4, 20)),          // bool scratch = result
    // instruction 33
    SharkyInstruction::Pop,                      // pop slot 20

    // instruction 34
    SharkyInstruction::JumpIfNot((21, 4)),       // jump to 21 if counter < limit

    ]);

    let mut interpreter = SharkyInterpreter::new(program_arc);
    interpreter.run();
    interpreter.get_operational_stack().debug_print_stack();
}
