use std::sync::{Arc};
 
use crate::{sharky_memory::*, sharky_instruction_set::*};

macro_rules! operational_binary_impl {
    ($self:ident, $a:expr, $b:expr, $op:tt) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let index_a = $self.operational_stack.read($a);
                let index_b = $self.operational_stack.read($b);
                let result = match (index_a, index_b) {
                    (SharkyDataType::Int(a), SharkyDataType::Int(b)) => {SharkyDataType::Int(a $op b)}
                    (SharkyDataType::Max(a), SharkyDataType::Max(b)) => {SharkyDataType::Max(a $op b)}
                    (SharkyDataType::Byte(a), SharkyDataType::Byte(b)) => {SharkyDataType::Byte(a $op b)}
                    (_,_) => {SharkyDataType::Nil} // TODO: return type mismatch interrupt
                };
                $self.operational_stack.push(result);
    };
    ($self:ident, $a:expr, $b:expr, $op:tt, real) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let index_a = $self.operational_stack.read($a);
                let index_b = $self.operational_stack.read($b);
                let result = match (index_a, index_b) {
                    (SharkyDataType::Int(a), SharkyDataType::Int(b)) => {SharkyDataType::Int(a $op b)}
                    (SharkyDataType::Max(a), SharkyDataType::Max(b)) => {SharkyDataType::Max(a $op b)}
                    (SharkyDataType::Real(a), SharkyDataType::Real(b)) => {SharkyDataType::Real(a $op b)}
                    (SharkyDataType::Byte(a), SharkyDataType::Byte(b)) => {SharkyDataType::Byte(a $op b)}
                    (_,_) => {SharkyDataType::Nil} // TODO: return type mismatch interrupt
                };
                $self.operational_stack.push(result);
    };
    // TODO: add b being zero variant. that raises an interrupt
}

macro_rules! operational_unary_impl {
    
    ($self:ident, $a:expr, $op:tt) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let index = $self.operational_stack.read($a);
                let result = match index {
                    SharkyDataType::Int(a) => {SharkyDataType::Int($op a)}
                    (SharkyDataType::Max(a)) => {SharkyDataType::Max($op a)}
                    (SharkyDataType::Byte(a)) => {SharkyDataType::Byte($op a)}
                    (_) => {SharkyDataType::Nil} // TODO: return type mismatch interrupt
                };
                $self.operational_stack.push(result);
    };
}

macro_rules! operational_binary_boolean_impl {
    ($self:ident, $a:expr, $b:expr, $op:tt) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let index_a = $self.operational_stack.read($a);
                let index_b = $self.operational_stack.read($b);
                let result = match (index_a, index_b) {
                    (SharkyDataType::Bool(a), SharkyDataType::Bool(b)) => {SharkyDataType::Bool(a $op b)}
                    (_,_) => {SharkyDataType::Bool(false)} // TODO: return type mismatch interrupt
                };
                $self.operational_stack.push(result);
    };
}

macro_rules! operational_binary_comparison_impl {
    ($self:ident, $a:expr, $b:expr, $op:tt) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let index_a = $self.operational_stack.read($a);
                let index_b = $self.operational_stack.read($b);
                let result = match (index_a, index_b) {
                    (SharkyDataType::Int(a), SharkyDataType::Int(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::Max(a), SharkyDataType::Max(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::Real(a), SharkyDataType::Real(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::Byte(a), SharkyDataType::Byte(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::Bool(a), SharkyDataType::Bool(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::String(a), SharkyDataType::String(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::HeapReference(a), SharkyDataType::HeapReference(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::Nil, SharkyDataType::Nil) => {SharkyDataType::Bool(SharkyDataType::Nil $op SharkyDataType::Nil)}
                    (_,_) => {SharkyDataType::Nil} // TODO: return type mismatch interrupt
                };
                $self.operational_stack.push(result);
    };
}
/*
 * TODO:
 * Switch local stack array to non_empty to prevent the representation of an error state.
 */

pub struct SharkyInterpreter {
    program_counter: usize,
    operational_stack: SharkyStack,
    local_stacks: SharkyStackVec,
    selected_local_stack: usize,
    program_memory: Arc<SharkyProgram>,
    stack_mode: SharkyStackMode,
}

impl SharkyInterpreter {

    pub fn new(program: Arc<SharkyProgram>) -> SharkyInterpreter {
        SharkyInterpreter { 
            program_counter: 0, 
            operational_stack: SharkyStack::default(), 
            local_stacks: vec![SharkyStack::default()], 
            selected_local_stack: 0, 
            program_memory: Arc::clone(&program),
            stack_mode: SharkyStackMode::Indexed,
        }
    }

    pub fn get_operational_stack(&mut self) -> &SharkyStack {
        &self.operational_stack
    }

    pub fn get_current_stack(&mut self) -> Option<&mut SharkyStack> {
        if let Some(frame) = self.local_stacks.get_mut(self.selected_local_stack) {
            Some(frame)
        } else {
            None
        }
    }

    fn get_active_stack(&mut self) -> Option<&mut SharkyStack> {
        match self.stack_mode {
            SharkyStackMode::Indexed => {
                Some(self.get_current_stack()?)
            }
            SharkyStackMode::Operative => {
                Some(&mut self.operational_stack)
            }
            _ => { None }
        }
    }

    fn push_constant(&mut self, value: SharkyDataType) -> Option<()> {
        self.get_active_stack()?.push(value);
        Some(())
    }

    fn interpret(&mut self) -> Option<()> {

        let current_instruction = self.program_memory
            .as_ref()
            .get(self.program_counter)?
            .clone();
        // TODO: interrupts
        match current_instruction {

            // stack select ops
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
                self.push_constant(SharkyDataType::Nil)?,
            SharkyInstruction::ConstantPushReal(v) => 
                self.push_constant(SharkyDataType::Real(v))?,
            SharkyInstruction::ConstantPushMax(v) => 
                self.push_constant(SharkyDataType::Max(v))?,
            SharkyInstruction::ConstantPushInt(v) => 
                self.push_constant(SharkyDataType::Int(v))?,
            SharkyInstruction::ConstantPushByte(v) => 
                self.push_constant(SharkyDataType::Byte(v))?,
            SharkyInstruction::ConstantPushBool(v) => 
                self.push_constant(SharkyDataType::Bool(v))?,
            SharkyInstruction::ConstantPushString(v) => 
                self.push_constant(SharkyDataType::String(v))?,
            SharkyInstruction::ConstantPushHeapReference(v) => 
                self.push_constant(SharkyDataType::HeapReference(v))?,

            // stack ops
            SharkyInstruction::Copy(index) => {
                if let Some(stack) = self.get_active_stack() {
                    let data = stack.read(index); // TODO: interrupt upon non-existent index
                    stack.push(data);
                }
            }

            SharkyInstruction::Nilify(index) => {
                if let Some(stack) = self.get_active_stack() {
                    stack.set(index, SharkyDataType::Nil);// TODO: interrupt upon non-existent index
                }
            }

            SharkyInstruction::CopyTo((a, b)) => {
                if let Some(stack) = self.get_active_stack() {
                    stack.set(a, stack.read(b));
                }
            }

            SharkyInstruction::Pop => {
                if let Some(stack) = self.get_active_stack() {
                    stack.pop();
                }
            }

            SharkyInstruction::Clear => {
                if let Some(stack) = self.get_active_stack() {
                    stack.clear();
                }
            }

            // operative ops
            SharkyInstruction::Add((a, b)) => {operational_binary_impl!(self, a, b, +, real);}
            SharkyInstruction::Subtract((a, b)) => {operational_binary_impl!(self, a, b, -, real);}
            SharkyInstruction::Multiply((a, b)) => {operational_binary_impl!(self, a, b, *, real);}
            SharkyInstruction::Divide((a, b)) => {operational_binary_impl!(self, a, b, /, real);}
            SharkyInstruction::Modulus((a, b)) => {operational_binary_impl!(self, a, b, %, real);}

            SharkyInstruction::BitLeftShift((a, b)) => {operational_binary_impl!(self, a, b, <<);}
            SharkyInstruction::BitRightShift((a, b)) => {operational_binary_impl!(self, a, b, >>);}
            SharkyInstruction::BitAnd((a, b)) => {operational_binary_impl!(self, a, b, &);}
            SharkyInstruction::BitXor((a, b)) => {operational_binary_impl!(self, a, b, ^);}
            SharkyInstruction::BitOr((a, b)) => {operational_binary_impl!(self, a, b, |);}
            SharkyInstruction::BitNot((a)) => {operational_unary_impl!(self, a, !);}

            SharkyInstruction::Not(a) => {
                let val = self.operational_stack.read(a);
                let result = match val {
                    SharkyDataType::Bool(a) => {!a}
                    _ => {false}// TODO: type mismatch interrupt
                };
                self.operational_stack.push(SharkyDataType::Bool(result));
            }

            SharkyInstruction::And((a, b)) => {operational_binary_boolean_impl!(self, a, b, &&);}
            SharkyInstruction::Or((a, b)) => {operational_binary_boolean_impl!(self, a, b, ||);}
            SharkyInstruction::Equals((a, b)) => {operational_binary_comparison_impl!(self, a, b, ==);}
            SharkyInstruction::NotEquals((a, b)) => {operational_binary_comparison_impl!(self, a, b, !=);}
            SharkyInstruction::GreaterThan((a, b)) => {operational_binary_comparison_impl!(self, a, b, >);}
            SharkyInstruction::LesserThan((a, b)) => {operational_binary_comparison_impl!(self, a, b, <);}
            SharkyInstruction::GreaterThanOrEquals((a, b)) => {operational_binary_comparison_impl!(self, a, b, >=);}
            SharkyInstruction::LesserThanOrEquals((a, b)) => {operational_binary_comparison_impl!(self, a, b, <=);}

            

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