use std::sync::{Arc, Mutex};

use crate::sharky_memory::{self, SharkyDataType, SharkyStack}; 

#[derive(Default, Debug, Clone)]
pub enum SharkyStackMode {
    #[default]
    Indexed,
    Addressed,
    Operative,
    Native,
}

#[derive(Default, Debug, Clone)]
pub enum SharkyInstruction {
    StackMode(SharkyStackMode),
    SelectStack(usize),
    PushStack,
    PopStack,

    // push a constant value to the top of the stack
    ConstantPushMax(sharky_memory::SharkyMax),
    ConstantPushInt(sharky_memory::SharkyInt),
    ConstantPushReal(sharky_memory::SharkyReal),
    ConstantPushByte(sharky_memory::SharkyByte),
    ConstantPushBool(sharky_memory::SharkyBool),
    ConstantPushString(sharky_memory::SharkyString),
    ConstantPushHeapReference(sharky_memory::SharkyHeapAddress),
    ConstantPushNil,

    // memory operations
    Copy(usize),
    Nilify(usize),
    CopyTo((usize, usize)),
    Pop,

    // operative operations    
    CopyOperativeToStack, // Copies the top of the operative stack to the selected indexed stack.

    Add((usize, usize)),
    Subtract((usize, usize)),
    Multiply((usize, usize)),
    Divide((usize, usize)),
    BitLeftShift((usize, usize)),
    BitRightShift((usize, usize)),
    BitAnd((usize, usize)),
    BitXor((usize, usize)),
    BitOr((usize, usize)),
    BitNot(usize),
    Not(usize),
    And((usize, usize)),
    Or((usize, usize)),
    Equals((usize, usize)),
    NotEquals((usize, usize)),
    GreaterThan((usize, usize)),
    LesserThan((usize, usize)),
    GreaterThanOrEquals((usize, usize)),
    LesserThanOrEquals((usize, usize)),

    // functional operations
    Call(usize),
    Return,

    // thread operations
    SpawnThread(usize),
    Await(usize),

    // Logic operations
    Jump(usize),
    JumpIfNot((usize, usize)),
    PopJumpIfNot((usize, usize)),
    #[default]
    NoOperation,

    // Native operations
    PushNative(usize),
    ClaimNative(usize), // pushes a native index into the selected stack.
    ClearNative,
    CallNative(usize),
}

pub type SharkyProgram = Vec<SharkyInstruction>;

pub struct SharkyProcedure {
    return_address: usize,
    return_stack: usize,
}

pub struct SharkyInterpreter {
    program_counter: usize,
    operational_stack: sharky_memory::SharkyStack,
    procedure_frame: Vec<SharkyProcedure>,
    local_stacks: sharky_memory::SharkyStackVec,
    selected_local_stack: usize,
    program_memory: Arc<SharkyProgram>,
    stack_mode: SharkyStackMode,
}

impl SharkyInterpreter {

    pub fn new(program: Arc<SharkyProgram>) -> SharkyInterpreter {
        SharkyInterpreter { 
            program_counter: 0, 
            operational_stack: sharky_memory::SharkyStack::default(), 
            procedure_frame: vec![], 
            local_stacks: vec![sharky_memory::SharkyStack::default()], 
            selected_local_stack: 0, 
            program_memory: Arc::clone(&program),
            stack_mode: SharkyStackMode::Indexed,
        }
    }

    pub fn get_operational_stack(&mut self) -> &sharky_memory::SharkyStack {
        &self.operational_stack
    }

    pub fn get_current_stack(&mut self) -> Option<&mut sharky_memory::SharkyStack> {
        if let Some(frame) = self.local_stacks.get_mut(self.selected_local_stack) {
            Some(frame)
        } else {
            None
        }
    }

    fn push_constant(&mut self, value: sharky_memory::SharkyDataType) -> Option<()> {
        // TODO: support other stack modes
        match self.stack_mode {
            SharkyStackMode::Indexed => {
                let stack_index = self.selected_local_stack;
                self.local_stacks.get_mut(stack_index)?.push(value);
            }
            SharkyStackMode::Operative => {
                self.operational_stack.push(value);
            }
            _ => {}
        }
        Some(())
    }

    fn interpret(&mut self) -> Option<()> {

        let current_instruction = self.program_memory
            .as_ref()
            .get(self.program_counter)?
            .clone();
        // TODO: interrupts
        match current_instruction {

            // stack ops
            SharkyInstruction::StackMode(mode) =>
                self.stack_mode = mode,
            SharkyInstruction::SelectStack(stack) =>
                self.selected_local_stack = stack, // TODO: raise interrupt upon illegal stack selection
            SharkyInstruction::PushStack =>
                self.local_stacks.push(SharkyStack::default()),
            SharkyInstruction::PopStack =>
                { let _ = self.local_stacks.pop(); } // TODO: raise interrupt upon trying to drop the first stack
            
            // constant push ops
            SharkyInstruction::ConstantPushNil => 
                self.push_constant(sharky_memory::SharkyDataType::Nil)?,
            SharkyInstruction::ConstantPushReal(v) => 
                self.push_constant(sharky_memory::SharkyDataType::Real(v))?,
            SharkyInstruction::ConstantPushMax(v) => 
                self.push_constant(sharky_memory::SharkyDataType::Max(v))?,
            SharkyInstruction::ConstantPushInt(v) => 
                self.push_constant(sharky_memory::SharkyDataType::Int(v))?,
            SharkyInstruction::ConstantPushByte(v) => 
                self.push_constant(sharky_memory::SharkyDataType::Byte(v))?,
            SharkyInstruction::ConstantPushBool(v) => 
                self.push_constant(sharky_memory::SharkyDataType::Bool(v))?,
            SharkyInstruction::ConstantPushString(v) => 
                self.push_constant(sharky_memory::SharkyDataType::String(v))?,
            SharkyInstruction::ConstantPushHeapReference(v) => 
                self.push_constant(sharky_memory::SharkyDataType::HeapReference(v))?,

            // operative ops
            SharkyInstruction::Add((a, b)) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let index_a = self.operational_stack.read(a);
                let index_b = self.operational_stack.read(b);
                let result = match (index_a, index_b) {
                    (SharkyDataType::Int(a), SharkyDataType::Int(b)) => {SharkyDataType::Int(a + b)}
                    (SharkyDataType::Max(a), SharkyDataType::Max(b)) => {SharkyDataType::Max(a + b)}
                    (SharkyDataType::Real(a), SharkyDataType::Real(b)) => {SharkyDataType::Real(a + b)}
                    (SharkyDataType::Byte(a), SharkyDataType::Byte(b)) => {SharkyDataType::Byte(a + b)}
                    (l,r) => {SharkyDataType::Nil} // TODO: return type mismatch interrupt
                };
                self.operational_stack.push(result);
            }

            SharkyInstruction::Subtract((a, b)) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let index_a = self.operational_stack.read(a);
                let index_b = self.operational_stack.read(b);
                let result = match (index_a, index_b) {
                    (SharkyDataType::Int(a), SharkyDataType::Int(b)) => {SharkyDataType::Int(a - b)}
                    (SharkyDataType::Max(a), SharkyDataType::Max(b)) => {SharkyDataType::Max(a - b)}
                    (SharkyDataType::Real(a), SharkyDataType::Real(b)) => {SharkyDataType::Real(a - b)}
                    (SharkyDataType::Byte(a), SharkyDataType::Byte(b)) => {SharkyDataType::Byte(a - b)}
                    (l,r) => {SharkyDataType::Nil} // TODO: return type mismatch interrupt
                };
                self.operational_stack.push(result);
            }

            SharkyInstruction::Multiply((a, b)) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let index_a = self.operational_stack.read(a);
                let index_b = self.operational_stack.read(b);
                let result = match (index_a, index_b) {
                    (SharkyDataType::Int(a), SharkyDataType::Int(b)) => {SharkyDataType::Int(a * b)}
                    (SharkyDataType::Max(a), SharkyDataType::Max(b)) => {SharkyDataType::Max(a * b)}
                    (SharkyDataType::Real(a), SharkyDataType::Real(b)) => {SharkyDataType::Real(a * b)}
                    (SharkyDataType::Byte(a), SharkyDataType::Byte(b)) => {SharkyDataType::Byte(a * b)}
                    (l,r) => {SharkyDataType::Nil} // TODO: return type mismatch interrupt
                };
                self.operational_stack.push(result);
            }

            SharkyInstruction::Divide((a, b)) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let index_a = self.operational_stack.read(a);
                let index_b = self.operational_stack.read(b);
                // TODO: add interrupt for divide by zero
                let result = match (index_a, index_b) {
                    (SharkyDataType::Int(a), SharkyDataType::Int(b)) => {SharkyDataType::Int(a / b)}
                    (SharkyDataType::Max(a), SharkyDataType::Max(b)) => {SharkyDataType::Max(a / b)}
                    (SharkyDataType::Real(a), SharkyDataType::Real(b)) => {SharkyDataType::Real(a / b)}
                    (SharkyDataType::Byte(a), SharkyDataType::Byte(b)) => {SharkyDataType::Byte(a / b)}
                    (l,r) => {SharkyDataType::Nil} // TODO: return type mismatch interrupt
                };
                self.operational_stack.push(result);
            }
            
            _ => {}
        }

        self.program_counter += 1;
        Some(())
    }

    pub fn run(&mut self) -> Option<()> {
        let program_memory_length = self.program_memory.as_ref().len();
        while self.program_counter < program_memory_length {
            self.interpret();
        }
        Some(())
    }
}