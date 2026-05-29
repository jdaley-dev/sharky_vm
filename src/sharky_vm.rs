#![allow(dead_code)]
use std::sync::{Arc, RwLock, RwLockReadGuard}; 

use crate::{sharky_data_types::*, sharky_instruction_set::*, sharky_memory::*, sharky_string::SharkyStringPool};

#[macro_use] mod sharky_vm_macros;
/*
 * TODO:
 * Switch local stack array to non_empty to prevent the representation of an error state.
 */

#[derive(Default)]
pub struct SharkyMemoryLayout {
    transitional_stack: SharkyDataStack,
    local_stacks: Vec<SharkyDataStack>,
    operational_stack: SharkyDataStack,
    parameter_stack: SharkyDataStack,
    heap_reference_stack: SharkyDataStack,
    string_stack: SharkyDataStack,
    selected_local_stack: usize,
    stack_mode: SharkyStackMode,
}

impl SharkyMemoryLayout {
    pub fn new() -> Self { 
        let mut result = Self::default();
        result.local_stacks.push(SharkyDataStack::default()); // initialize the local stacks with a minimum of one stack.
        result 
    }

    pub fn check_for_address_in_stacks(&self, address: SharkyHeapFrameIndex) -> bool {
        let heap_search = |data: &SharkyDataType| {
            match data {
                SharkyDataType::HeapReference(href) => {
                    href.frame == address
                }
                _ => false
            }
        };
        // local stacks search
        for stack in self.local_stacks.iter() {
            if stack.search(heap_search) { return true; }
        }
        if self.transitional_stack.search(heap_search) { return true; }
        if self.operational_stack.search(heap_search) { return true; }
        if self.parameter_stack.search(heap_search) { return true; }

        false
    }

    pub fn set_stack_mode(&mut self, mode: SharkyStackMode) {
        self.stack_mode = mode;
    }

    pub fn new_local_stack(&mut self) {
        self.local_stacks.push(SharkyDataStack::default());
    }

    pub fn pop_local_stack(&mut self) {
        self.local_stacks.pop();
    }

    pub fn select_local_stack(&mut self, index: usize) {
        self.selected_local_stack = index; 
    }

    pub fn get_transitional_stack(&mut self) -> &mut SharkyDataStack {
        &mut self.transitional_stack
    }

    pub fn get_operational_stack(&mut self) -> &mut SharkyDataStack {
        &mut self.operational_stack
    }

    pub fn print_debug(&self) -> Option<()> {
        let mut count = 0;
        for i in self.local_stacks.iter() {
            println!("STACK {count}");
            i.debug_print_stack();
            count += 1;
        }
        Some(())
    }

    pub fn get_active_stack_mut(&mut self) -> Option<&mut SharkyDataStack> {
        match self.stack_mode {
            SharkyStackMode::Indexed => {
                let selected = self.selected_local_stack;
                self.local_stacks.get_mut(selected)
            }
            SharkyStackMode::Transitional => {
                Some(&mut self.transitional_stack)
            }
            SharkyStackMode::Operative => {
                Some(&mut self.operational_stack)
            }

            _ => None,
        }
    }

    pub fn get_active_stack(&self) -> Option<&SharkyDataStack> {
                match self.stack_mode {
            SharkyStackMode::Indexed => {
                let selected = self.selected_local_stack;
                self.local_stacks.get(selected)
            }
            SharkyStackMode::Transitional => {
                Some(&self.operational_stack)
            }
            SharkyStackMode::Operative => {
                Some(&self.operational_stack)
            }
            _ => None,
        }
    }
}

pub struct SharkyInterpreter {
    memory: RwLock<SharkyMemoryLayout>,
    program_counter: usize,
    program_memory: Arc<SharkyProgram>,
    running: bool,
}

pub type SharkySyncedInterpreter = Arc<RwLock<SharkyInterpreter>>;

impl SharkyInterpreter {

    pub fn new(program: Arc<SharkyProgram>) -> SharkyInterpreter {
        SharkyInterpreter { 
            memory: RwLock::new(SharkyMemoryLayout::new()),
            program_counter: 0, 
            program_memory: Arc::clone(&program),
            running: true,
        }
    }

    pub fn has_reference(&self, address: SharkyHeapFrameIndex) -> bool {
        self.memory.read().unwrap().check_for_address_in_stacks(address)
    }

    pub fn new_arc(program: Arc<SharkyProgram>) -> SharkySyncedInterpreter {
        Arc::new(RwLock::new(SharkyInterpreter::new(program)))
    }
 
    fn push_constant(&mut self, value: SharkyDataType) -> Option<()> {
        self.memory.write().ok()?.get_active_stack_mut()?.push(value);
        Some(())
    }

    fn read_parameter<T: SharkyValue>(&mut self, parameter: SharkyParameter<T>) -> Option<T> where SharkyDataType: TryInto<T> + Clone,{
        match parameter {
            SharkyParameter::Constant(val) => {
                Some(val.into())
            }
            SharkyParameter::StackIndex(index) => {
                Some((self.memory.read().ok()?.get_active_stack()?.read(index)?.clone()).try_into().ok()?)
            }
            SharkyParameter::None => {
                None
            }
        }
    }

    pub fn print_debug(&mut self) -> Option<()> {
        let memory = self.memory.read().ok()?;
        memory.print_debug();
        Some(())
    }

    pub fn get_memory(&self) -> Option<RwLockReadGuard<'_, SharkyMemoryLayout>> {
        self.memory.read().ok()
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn interpret(&mut self) -> Option<()> {
        if !self.is_running() {
            return Some(())
        }
        let current_instruction = self.program_memory
            .as_ref()
            .get(self.program_counter)?
            .clone();
        
        let prev_counter = self.program_counter;

        self.interpret_stackops(current_instruction.clone())?;
        self.interpret_constantops(current_instruction.clone())?;
        self.interpret_conversionops(current_instruction.clone())?;
        self.interpret_operativeops(current_instruction.clone())?;
        self.interpret_logicops(current_instruction.clone())?;
        self.interpret_heapops(current_instruction.clone())?;
        
        if self.program_counter == prev_counter {
            self.program_counter += 1;
        }

        let program_memory_length = self.program_memory.as_ref().len();
        if self.program_counter >= program_memory_length {
            self.running = false;
        }
        Some(())
    }

    fn interpret_stackops(&mut self, op: SharkyInstruction) -> Option<()> {
        match op {
            SharkyInstruction::SetStackMode(mode) =>
                self.memory.write().ok()?.set_stack_mode(mode),
            
            SharkyInstruction::SelectLocalStack(stack) => {
                let parameter = self.read_parameter(stack)?;
                self.memory.write().ok()?.select_local_stack(parameter) // TODO: raise interrupt upon illegal stack selection
            }
            SharkyInstruction::PushLocalStack =>
                self.memory.write().ok()?.new_local_stack(),
            
            SharkyInstruction::PopLocalStack =>
                self.memory.write().ok()?.pop_local_stack(),
            
            SharkyInstruction::PushTransition(a) => {
                let param = self.read_parameter(a)?;
                let mut memory = self.memory.write().ok()?;
                let stack = memory.get_active_stack()?;
                let value = stack.read(param)?.clone();
                memory.get_transitional_stack().push(value);
            }
            
            SharkyInstruction::CopyTransition(a) => {
                let param = self.read_parameter(a)?;
                let mut memory = self.memory.write().ok()?;
                let value = memory.get_transitional_stack().read(param)?.clone();
                memory.get_active_stack_mut()?.push(value);
            }

            SharkyInstruction::Copy(a) => {
                let param_a = self.read_parameter(a)?;
                let mut memory = self.memory.write().ok()?;
                let stack = memory.get_active_stack_mut()?;
                let data = stack.read(param_a)?.clone(); // TODO: interrupt upon non-existent index
                stack.push(data);
            }

            SharkyInstruction::Nilify(a) => {
                let param_a = self.read_parameter(a)?;
                let mut memory = self.memory.write().ok()?;
                let stack = memory.get_active_stack_mut()?;
                stack.set(param_a, SharkyDataType::Nil);// TODO: interrupt upon non-existent index
            }

            SharkyInstruction::Set((a, b)) => {
                let param_a = self.read_parameter(a)?;
                let param_b = self.read_parameter(b)?;
                let mut memory = self.memory.write().ok()?;
                let stack = memory.get_active_stack_mut()?;
                stack.set(param_a, stack.read(param_b)?.clone());
            }

            SharkyInstruction::Pop => {
                self.memory.write().ok()?.get_active_stack_mut()?.pop();
            }

            SharkyInstruction::Clear => {
                self.memory.write().ok()?.get_active_stack_mut()?.clear();
            }
            _ => {}
        }
        Some(())
    }
    
    fn interpret_constantops(&mut self, op: SharkyInstruction) -> Option<()> {
        match op {
            SharkyInstruction::PushNil => {self.push_constant(SharkyDataType::Nil)?}
            SharkyInstruction::PushReal(v) => {push_constant!(self, v, Real);}
            SharkyInstruction::PushMax(v) => {push_constant!(self, v, Max);}
            SharkyInstruction::PushInt(v) => {push_constant!(self, v, Int);}
            SharkyInstruction::PushByte(v) => {push_constant!(self, v, Byte);}
            SharkyInstruction::PushBool(v) => {push_constant!(self, v, Bool);}
            SharkyInstruction::PushHeapReference(v) =>{push_constant!(self, v, HeapReference);}
            _ => {}
        }
        Some(())
    }
    
    fn interpret_conversionops(&mut self, op: SharkyInstruction) -> Option<()> {
        match op {
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
            _ => {}
        }
        Some(())
    }

    fn interpret_operativeops(&mut self, op: SharkyInstruction) -> Option<()> {
        match op {
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
                let mut memory = self.memory.write().ok()?;
                let opstack = memory.get_operational_stack();
                let val = opstack.read(param_a);
                let result = match val? {
                    SharkyDataType::Bool(a) => {!a}
                    _ => {false}// TODO: type mismatch interrupt
                };
                opstack.push(SharkyDataType::Bool(result));
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
        Some(())
    }

    fn interpret_logicops(&mut self, op: SharkyInstruction) -> Option<()> {
        match op {
            SharkyInstruction::Jump(a) => {
                self.program_counter = self.read_parameter(a)?;
                return None;
            }            

            SharkyInstruction::JumpIfNot((a, b)) => {
                let param_a = self.read_parameter(a)?;
                let param_b = self.read_parameter(b)?;
                let mut memory = self.memory.write().ok()?;
                let read = memory.get_active_stack_mut()?.read(param_b);
                let mut jump = false;
                match read? {
                    SharkyDataType::Bool(a) => {
                        jump = !a;
                    }
                    _ => {}
                }
                if jump {
                    self.program_counter = param_a;
                    return None;
                }
            }
            _ => {}
        }
        Some(())
    }

    #[allow(unused)]
    fn interpret_heapops(&mut self, op: SharkyInstruction) -> Option<()> {
        match op {
            SharkyInstruction::CreateDynamicHeap => {}                         
            SharkyInstruction::CreateByteHeap => {}                            
            SharkyInstruction::CreateIntHeap => {}                              
            SharkyInstruction::CreateMaxHeap => {}                                
            SharkyInstruction::CreateRealHeap => {}                                 
            SharkyInstruction::ReadHeap((a, b))  => {}  
            SharkyInstruction::WriteHeap((a, b))  => {}
            SharkyInstruction::PushHeap((a, b))   => {}
            SharkyInstruction::DeleteHeap(a) => {}
            SharkyInstruction::CloneHeap(a) => {}
            SharkyInstruction::SliceHeap((a, b)) => {} 
            SharkyInstruction::SizeHeap(a) => {}
            _ => {}
        }
        Some(())
    }
}

