use std::sync::{Arc};
 
use crate::{sharky_data_types::*, sharky_instruction_set::*, sharky_memory::*, sharky_string::SharkyStringPool};

macro_rules! operational_binary_impl {
    ($self:ident, $a:expr, $b:expr, $op:tt) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let param_a = $self.read_parameter($a)?;
                let param_b = $self.read_parameter($b)?; 
                let index_a = $self.operational_stack.read(param_a);
                let index_b = $self.operational_stack.read(param_b);
                let result = match (index_a?, index_b?) {
                    (SharkyDataType::Int(a), SharkyDataType::Int(b)) => {SharkyDataType::Int(a $op b)}
                    (SharkyDataType::Max(a), SharkyDataType::Max(b)) => {SharkyDataType::Max(a $op b)}
                    (SharkyDataType::Byte(a), SharkyDataType::Byte(b)) => {SharkyDataType::Byte(a $op b)}
                    (_,_) => {SharkyDataType::Nil} // TODO: return type mismatch interrupt
                };
                $self.operational_stack.push(result);
    };
    ($self:ident, $a:expr, $b:expr, $op:tt, real) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let param_a = $self.read_parameter($a)?;
                let param_b = $self.read_parameter($b)?; 
                let index_a = $self.operational_stack.read(param_a);
                let index_b = $self.operational_stack.read(param_b);
                let result = match (index_a?, index_b?) {
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
                let param_a = $self.read_parameter($a)?;
                let index = $self.operational_stack.read(param_a);
                let result = match index? {
                    SharkyDataType::Int(a) => {SharkyDataType::Int($op a)}
                    SharkyDataType::Max(a) => {SharkyDataType::Max($op a)}
                    SharkyDataType::Byte(a) => {SharkyDataType::Byte($op a)}
                    _ => {SharkyDataType::Nil} // TODO: return type mismatch interrupt
                };
                $self.operational_stack.push(result);
    };
}

macro_rules! operational_binary_boolean_impl {
    ($self:ident, $a:expr, $b:expr, $op:tt) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let param_a = $self.read_parameter($a)?;
                let param_b = $self.read_parameter($b)?; 
                let index_a = $self.operational_stack.read(param_a);
                let index_b = $self.operational_stack.read(param_b);
                let result = match (index_a?, index_b?) {
                    (SharkyDataType::Bool(a), SharkyDataType::Bool(b)) => {SharkyDataType::Bool(*a $op *b)}
                    (_,_) => {SharkyDataType::Bool(false)} // TODO: return type mismatch interrupt
                };
                $self.operational_stack.push(result);
    };
}

macro_rules! operational_binary_comparison_impl {
    ($self:ident, $a:expr, $b:expr, $op:tt) => {
                // TODO: raise interrupt upon adding between non-existent stack elements
                let param_a = $self.read_parameter($a)?;
                let param_b = $self.read_parameter($b)?; 
                let index_a = $self.operational_stack.read(param_a);
                let index_b = $self.operational_stack.read(param_b);
                let result = match (index_a?, index_b?) {
                    (SharkyDataType::Int(a), SharkyDataType::Int(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::Max(a), SharkyDataType::Max(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::Real(a), SharkyDataType::Real(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::Byte(a), SharkyDataType::Byte(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::Bool(a), SharkyDataType::Bool(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::HeapReference(a), SharkyDataType::HeapReference(b)) => {SharkyDataType::Bool(a $op b)}
                    (SharkyDataType::Nil, SharkyDataType::Nil) => {SharkyDataType::Bool(SharkyDataType::Nil $op SharkyDataType::Nil)}
                    (_,_) => {SharkyDataType::Nil} // TODO: return type mismatch interrupt
                };
                $self.operational_stack.push(result);
    };
}

macro_rules! push_constant {
    ($self:ident, $val:expr, $data_type:ident) => {
            {
                let parameter = $self.read_parameter($val)?;
                $self.push_constant(SharkyDataType::$data_type(parameter))?;
            }
    };
}

macro_rules! convert_match_impl {
    ($self:ident, $a:expr, $stack:ident, $($pattern:pat => $body:expr),* $(,)?) => {
        let param_a = $self.read_parameter($a)?;
        let $stack = $self.get_active_stack()?;
        let data = $stack.read(param_a)?;
        match data {
            $($pattern => $body,)*
            _ => {} // TODO: failed conversion interrupt
        }
    }
}
/*
 * TODO:
 * Switch local stack array to non_empty to prevent the representation of an error state.
 */

pub struct SharkyInterpreter {
    program_counter: usize,
    operational_stack: SharkyDataStack,

    local_stacks: Vec<SharkyDataStack>,
    selected_local_stack: usize,
    
    program_memory: Arc<SharkyProgram>,
    string_memory: Arc<SharkyStringPool>,

    stack_mode: SharkyStackMode,
    parameter_stack: SharkyDataStack,
    transitional_stack: SharkyDataStack,
    string_stack: SharkyDataStack,
}

impl SharkyInterpreter {

    pub fn new(program: Arc<SharkyProgram>, string_pool: Arc<SharkyStringPool>) -> SharkyInterpreter {
        SharkyInterpreter { 
            program_counter: 0, 
            operational_stack: SharkyDataStack::default(), 
            local_stacks: vec![SharkyDataStack::default()], 
            selected_local_stack: 0, 
            program_memory: Arc::clone(&program),
            string_memory: Arc::clone(&string_pool),
            stack_mode: SharkyStackMode::Indexed,
            parameter_stack: SharkyDataStack::default(),
            transitional_stack: SharkyDataStack::default(),
            string_stack: SharkyDataStack::default(),
        }
    }

    pub fn get_current_stack(&mut self) -> Option<&mut SharkyDataStack> {
        if let Some(frame) = self.local_stacks.get_mut(self.selected_local_stack) {
            Some(frame)
        } else {
            None
        }
    }

    fn get_active_stack(&mut self) -> Option<&mut SharkyDataStack> {
        match self.stack_mode {
            SharkyStackMode::Indexed => {
                Some(self.get_current_stack()?)
            }
            SharkyStackMode::Operative => {
                Some(&mut self.operational_stack)
            }
            SharkyStackMode::Parameter => {
                Some(&mut self.parameter_stack)
            }
            SharkyStackMode::Transitional => {
                Some(&mut self.transitional_stack)
            }
            SharkyStackMode::String => Some(&mut self.string_stack),
            _ => { None }
        }
    }

    fn push_constant(&mut self, value: SharkyDataType) -> Option<()> {
        self.get_active_stack()?.push(value);
        Some(())
    }

    fn read_size(&mut self, index: usize) -> Option<usize> {
        let data = self.get_active_stack()?.read(index);
        match data? {
            SharkyDataType::Max(a) => Some(*a),
            SharkyDataType::Int(a) => Some(*a as usize),
            SharkyDataType::Byte(a) => Some(*a as usize),
            _ => None 
        }
    }

    fn read_parameter<T: SharkyValue>(&mut self, parameter: SharkyParameter<T>) -> Option<T> where SharkyDataType: TryInto<T> + Clone,{
        match parameter {
            SharkyParameter::Constant(val) => {
                Some(val.into())
            }
            SharkyParameter::StackIndex(index) => {
                Some((self.get_active_stack()?.read(index)?.clone()).try_into().ok()?)
            }
            SharkyParameter::None => {
                None
            }
        }
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
                self.selected_local_stack = self.read_parameter(stack)?, // TODO: raise interrupt upon illegal stack selection
            SharkyInstruction::PushStack =>
                self.local_stacks.push(SharkyStack::default()),
            SharkyInstruction::PopStack =>
                { let _ = self.local_stacks.pop(); } // TODO: raise interrupt upon trying to drop the first stack
            
            // constant push ops
            SharkyInstruction::PushNil => {self.push_constant(SharkyDataType::Nil)?}
            SharkyInstruction::PushReal(v) => {push_constant!(self, v, Real);}
            SharkyInstruction::PushMax(v) => {push_constant!(self, v, Max);}
            SharkyInstruction::PushInt(v) => {push_constant!(self, v, Int);}
            SharkyInstruction::PushByte(v) => {push_constant!(self, v, Byte);}
            SharkyInstruction::PushBool(v) => {push_constant!(self, v, Bool);}
            SharkyInstruction::PushHeapReference(v) =>{push_constant!(self, v, HeapReference);}



            // conversion ops
            SharkyInstruction::ToInt(a) => {
                convert_match_impl!(self, a, stack, 
                    SharkyDataType::Max(v) => stack.push(SharkyDataType::Int(*v as SharkyInt)),
                    SharkyDataType::Byte(v) => stack.push(SharkyDataType::Int(*v as SharkyInt)),
                );
            }

            SharkyInstruction::ToMax(a) => {
                convert_match_impl!(self, a, stack, 
                    SharkyDataType::Int(v) => stack.push(SharkyDataType::Max(*v as SharkyMax)),
                    SharkyDataType::Byte(v) => stack.push(SharkyDataType::Max(*v as SharkyMax)),
                );
            }
            
            SharkyInstruction::ToByte(a) => {
                convert_match_impl!(self, a, stack, 
                    SharkyDataType::Max(v) => stack.push(SharkyDataType::Byte(*v as SharkyByte)),
                    SharkyDataType::Int(v) => stack.push(SharkyDataType::Byte(*v as SharkyByte)),
                );
            }

            // stack ops
            SharkyInstruction::PushTransition(a) => {
                let param = self.read_parameter(a)?;
                let stack = self.get_active_stack()?;
                let value = stack.read(param)?.clone();
                self.transitional_stack.push(value);
            }
            
            SharkyInstruction::CopyTransition(a) => {
                let param = self.read_parameter(a)?;
                let value = self.transitional_stack.read(param)?.clone();
                self.get_active_stack()?.push(value);
            }

            SharkyInstruction::Copy(a) => {
                let param_a = self.read_parameter(a)?;
                if let Some(stack) = self.get_active_stack() {
                    let data = stack.read(param_a)?.clone(); // TODO: interrupt upon non-existent index
                    stack.push(data);
                }
            }

            SharkyInstruction::Nilify(a) => {
                let param_a = self.read_parameter(a)?;
                if let Some(stack) = self.get_active_stack() {
                    stack.set(param_a, SharkyDataType::Nil);// TODO: interrupt upon non-existent index
                }
            }

            SharkyInstruction::Set((a, b)) => {
                let param_a = self.read_parameter(a)?;
                let param_b = self.read_parameter(b)?;
                if let Some(stack) = self.get_active_stack() {
                    stack.set(param_a, stack.read(param_b)?.clone());
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
            SharkyInstruction::BitNot(a) => {operational_unary_impl!(self, a, !);}

            SharkyInstruction::Not(a) => {
                let param_a = self.read_parameter(a)?;
                let val = self.operational_stack.read(param_a);
                let result = match val? {
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

            SharkyInstruction::Jump(a) => {
                self.program_counter = self.read_parameter(a)?;
                return Some(());
            }            

            SharkyInstruction::JumpIfNot((a, b)) => {
                let param_a = self.read_parameter(a)?;
                let param_b = self.read_parameter(b)?;
                let read = self.get_active_stack()?.read(param_b);
                let mut jump = false;
                match read? {
                    SharkyDataType::Bool(a) => {
                        jump = !a;
                    }
                    _ => {}
                }
                if jump {
                    self.program_counter = param_a;
                    return Some(());
                }
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