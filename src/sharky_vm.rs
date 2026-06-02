#![allow(dead_code)]
use parking_lot::{RwLock, RwLockReadGuard};
use std::sync::{Arc}; 

use crate::{sharky_app::SharkyTaskPool, sharky_data_types::*, sharky_memory::*, sharky_native::{SharkyFFIFunctionHandle, SharkyFFILibrary}};

#[macro_use] mod sharky_vm_macros;
/*
 * TODO:
 * Switch local stack array to non_empty to prevent the representation of an error state.
 */



pub struct SharkyVM {
    memory: RwLock<SharkyMemoryLayout>,
    program_counter: usize,
    program_memory: Arc<SharkyProgram>,
    heap: SharkyHeap,
    task_pool: SharkyTaskPool,
    ffi_function_table: Arc<Vec<SharkyFFIFunctionHandle>>,
    selected_frame: usize,
    running: bool,
}

pub type SharkySyncedVM = Arc<RwLock<SharkyVM>>;

/// TODO: Sharky code standard: Stop passing references to constructors. Pass clones.
impl SharkyVM {

    pub fn new(program: Arc<SharkyProgram>, heap: &SharkyHeap, task_pool: &SharkyTaskPool, ffi_functions: Arc<Vec<SharkyFFIFunctionHandle>>) -> SharkyVM {
        SharkyVM { 
            memory: RwLock::new(SharkyMemoryLayout::new()),
            program_counter: 0, 
            program_memory: Arc::clone(&program),
            heap: heap.clone(),
            task_pool: task_pool.clone(),
            ffi_function_table: ffi_functions,
            selected_frame: 0,
            running: true,
        }
    }

    pub fn new_subvm(&self, program_counter: usize) -> SharkySyncedVM {
        let mut subvm = Self::new(self.program_memory.clone(), &self.heap, &self.task_pool, self.ffi_function_table.clone());
        let parameter_stack = self.memory.write().get_parameter_stack_mut().clone();
        subvm.program_counter = program_counter;
        subvm.memory.write().set_parameter_stack(&parameter_stack);
        Arc::new(RwLock::new(subvm))
    }

    pub fn has_reference(&self, address: SharkyHeapFrameIndex) -> bool {
        self.memory.read().has_heap_ref(address)
    }

    pub fn new_arc(program: Arc<SharkyProgram>, heap: &SharkyHeap, task_pool: &SharkyTaskPool, ffi_functions: Arc<Vec<SharkyFFIFunctionHandle>>) -> SharkySyncedVM {
        Arc::new(RwLock::new(SharkyVM::new(program, heap, task_pool, ffi_functions)))
    }
 
    fn push_constant(&mut self, value: SharkyDataType) -> Option<()> {
        self.memory.write().get_active_stack_mut()?.push(value);
        Some(())
    }

    pub fn get_program_counter(&self) -> usize {
        self.program_counter
    }

    pub fn set_program_counter(&mut self, index: usize) {
        self.program_counter = index; 
    }

    fn read_parameter<T: SharkyValue>(&mut self, parameter: SharkyParameter<T>) -> Option<T> where SharkyDataType: TryInto<T> + Clone,{
        match parameter {
            SharkyParameter::Constant(val) => {
                Some(val.into())
            }
            SharkyParameter::StackIndex(index) => {
                Some((self.memory.read().get_active_stack()?.read(index)?.clone()).try_into().ok()?)
            }
            SharkyParameter::None => {
                None
            }
        }
    }

    pub fn print_debug(&mut self) -> Option<()> {
        let memory = self.memory.read();
        memory.print_debug();
        Some(())
    }

    pub fn get_memory(&self) -> RwLockReadGuard<'_, SharkyMemoryLayout> {
        self.memory.read()
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

        self.interpret_stackops(&current_instruction);
        self.interpret_constantops(&current_instruction);
        self.interpret_conversionops(&current_instruction);
        self.interpret_operativeops(&current_instruction);
        self.interpret_logicops(&current_instruction);
        self.interpret_heapops(&current_instruction);
        self.interpret_taskops(&current_instruction);
        self.interpret_ffiops(&current_instruction);
        
        if self.program_counter == prev_counter {
            self.program_counter += 1;
        }

        let program_memory_length = self.program_memory.as_ref().len();
        if self.program_counter >= program_memory_length {
            self.running = false;
        }
        Some(())
    }

    fn interpret_stackops(&mut self, op: &SharkyInstruction) -> Option<()> {
        match op {
            SharkyInstruction::SetStackMode(mode) =>
                self.memory.write().set_stack_mode(mode.clone()),
            
            SharkyInstruction::SelectLocalStack(stack) => {
                let parameter = self.read_parameter(stack.clone())?;
                self.memory.write().select_local_stack(parameter) // TODO: raise interrupt upon illegal stack selection
            }
            SharkyInstruction::PushLocalStack =>
                self.memory.write().new_local_stack(),
            
            SharkyInstruction::PopLocalStack =>
                self.memory.write().pop_local_stack(),
            
            SharkyInstruction::PushTransition(a) => {
                let param = self.read_parameter(a.clone())?;
                let mut memory = self.memory.write();
                let stack = memory.get_active_stack()?;
                let value = stack.read(param)?.clone();
                memory.get_transitional_stack().push(value);
            }
            
            SharkyInstruction::CopyTransition(a) => {
                let param = self.read_parameter(a.clone())?;
                let mut memory = self.memory.write();
                let value = memory.get_transitional_stack().read(param)?.clone();
                memory.get_active_stack_mut()?.push(value);
            }

            SharkyInstruction::Copy(a) => {
                let param_a = self.read_parameter(a.clone())?;
                let mut memory = self.memory.write();
                let stack = memory.get_active_stack_mut()?;
                let data = stack.read(param_a)?.clone(); // TODO: interrupt upon non-existent index
                stack.push(data);
            }

            SharkyInstruction::Nilify(a) => {
                let param_a = self.read_parameter(a.clone())?;
                let mut memory = self.memory.write();
                let stack = memory.get_active_stack_mut()?;
                stack.set(param_a, SharkyDataType::Nil);// TODO: interrupt upon non-existent index
            }

            SharkyInstruction::Set((a, b)) => {
                let param_a = self.read_parameter(a.clone())?;
                let param_b = self.read_parameter(b.clone())?;
                let mut memory = self.memory.write();
                let stack = memory.get_active_stack_mut()?;
                stack.set(param_a, stack.read(param_b)?.clone());
            }

            SharkyInstruction::Pop => {
                self.memory.write().get_active_stack_mut()?.pop();
            }

            SharkyInstruction::Clear => {
                self.memory.write().get_active_stack_mut()?.clear();
            }
            _ => {}
        }
        Some(())
    }
    
    fn interpret_constantops(&mut self, op: &SharkyInstruction) -> Option<()> {
        match op {
            SharkyInstruction::PushNil => {self.push_constant(SharkyDataType::Nil)?}
            SharkyInstruction::PushReal(v) => {push_constant!(self, v.clone(), Real);}
            SharkyInstruction::PushMax(v) => {push_constant!(self, v.clone(), Max);}
            SharkyInstruction::PushInt(v) => {push_constant!(self, v.clone(), Int);}
            SharkyInstruction::PushByte(v) => {push_constant!(self, v.clone(), Byte);}
            SharkyInstruction::PushBool(v) => {push_constant!(self, v.clone(), Bool);}
            SharkyInstruction::PushHeapReference(v) =>{push_constant!(self, v.clone(), HeapReference);}
            _ => {}
        }
        Some(())
    }
    
    fn interpret_conversionops(&mut self, op: &SharkyInstruction) -> Option<()> {
        match op {
            SharkyInstruction::ToInt(a) => {
                convert_match_impl!(self, a.clone(), stack, 
                    SharkyDataType::Max(v) => stack.push(SharkyDataType::Int(*v as SharkyInt)),
                    SharkyDataType::Byte(v) => stack.push(SharkyDataType::Int(*v as SharkyInt)),
                );
            }

            SharkyInstruction::ToMax(a) => {
                convert_match_impl!(self, a.clone(), stack, 
                    SharkyDataType::Int(v) => stack.push(SharkyDataType::Max(*v as SharkyMax)),
                    SharkyDataType::Byte(v) => stack.push(SharkyDataType::Max(*v as SharkyMax)),
                );
            }
            
            SharkyInstruction::ToByte(a) => {
                convert_match_impl!(self, a.clone(), stack, 
                    SharkyDataType::Max(v) => stack.push(SharkyDataType::Byte(*v as SharkyByte)),
                    SharkyDataType::Int(v) => stack.push(SharkyDataType::Byte(*v as SharkyByte)),
                );
            }
            _ => {}
        }
        Some(())
    }

    fn interpret_operativeops(&mut self, op: &SharkyInstruction) -> Option<()> {
        match op {
            SharkyInstruction::Add((a, b)) => {operational_binary_impl!(self, a.clone(), b.clone(), +, real);}
            SharkyInstruction::Subtract((a, b)) => {operational_binary_impl!(self, a.clone(), b.clone(), -, real);}
            SharkyInstruction::Multiply((a, b)) => {operational_binary_impl!(self, a.clone(), b.clone(), *, real);}
            SharkyInstruction::Divide((a, b)) => {operational_binary_impl!(self, a.clone(), b.clone(), /, real);}
            SharkyInstruction::Modulus((a, b)) => {operational_binary_impl!(self, a.clone(), b.clone(), %, real);}

            SharkyInstruction::BitLeftShift((a, b)) => {operational_binary_impl!(self, a.clone(), b.clone(), <<);}
            SharkyInstruction::BitRightShift((a, b)) => {operational_binary_impl!(self, a.clone(), b.clone(), >>);}
            SharkyInstruction::BitAnd((a, b)) => {operational_binary_impl!(self, a.clone(), b.clone(), &);}
            SharkyInstruction::BitXor((a, b)) => {operational_binary_impl!(self, a.clone(), b.clone(), ^);}
            SharkyInstruction::BitOr((a, b)) => {operational_binary_impl!(self, a.clone(), b.clone(), |);}
            SharkyInstruction::BitNot(a) => {operational_unary_impl!(self, a.clone(), !);}

            SharkyInstruction::Not(a) => {
                let param_a = self.read_parameter(a.clone())?;
                let mut memory = self.memory.write();
                let opstack = memory.get_operational_stack();
                let val = opstack.read(param_a);
                let result = match val? {
                    SharkyDataType::Bool(a) => {!a}
                    _ => {false}// TODO: type mismatch interrupt
                };
                opstack.push(SharkyDataType::Bool(result));
            }

            SharkyInstruction::And((a, b)) => {operational_binary_boolean_impl!(self, a.clone(), b.clone(), &&);}
            SharkyInstruction::Or((a, b)) => {operational_binary_boolean_impl!(self, a.clone(), b.clone(), ||);}
            SharkyInstruction::Equals((a, b)) => {operational_binary_comparison_impl!(self, a.clone(), b.clone(), ==);}
            SharkyInstruction::NotEquals((a, b)) => {operational_binary_comparison_impl!(self, a.clone(), b.clone(), !=);}
            SharkyInstruction::GreaterThan((a, b)) => {operational_binary_comparison_impl!(self, a.clone(), b.clone(), >);}
            SharkyInstruction::LesserThan((a, b)) => {operational_binary_comparison_impl!(self, a.clone(), b.clone(), <);}
            SharkyInstruction::GreaterThanOrEquals((a, b)) => {operational_binary_comparison_impl!(self, a.clone(), b.clone(), >=);}
            SharkyInstruction::LesserThanOrEquals((a, b)) => {operational_binary_comparison_impl!(self, a.clone(), b.clone(), <=);}

            _ => {}
        }
        Some(())
    }

    fn interpret_logicops(&mut self, op: &SharkyInstruction) -> Option<()> {
        match op {
            SharkyInstruction::Jump(a) => {
                self.program_counter = self.read_parameter(a.clone())?;
            }            

            SharkyInstruction::JumpIfNot((a, b)) => {
                let param_a = self.read_parameter(a.clone())?;
                let param_b = self.read_parameter(b.clone())?;
                let mut memory = self.memory.write();
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
                }
            }
            _ => {}
        }
        Some(())
    }

    fn interpret_heapops(&mut self, op: &SharkyInstruction) -> Option<()> {
        match op {
                SharkyInstruction::CreateHeap => {
                    let address = self.heap.allocate();
                    self.memory.write().get_active_stack_mut()?.push(SharkyDataType::HeapReference(address));
                }

                SharkyInstruction::SelectHeap(a) => {
                    let select = self.read_parameter(a.clone())?;
                    self.selected_frame = select; 
                }
                
                SharkyInstruction::PushHeap => {
                    self.heap.push(self.selected_frame, &SharkyDataType::Nil);
                }

                SharkyInstruction::WriteHeap((a, b)) => {
                    let dest = self.read_parameter(a.clone())?;
                    let src = self.read_parameter(b.clone())?;
                    let data = self.memory.read().get_active_stack()?.read(src)?.clone();
                    self.heap.set(self.selected_frame, dest, &data);
                }

                SharkyInstruction::ReadHeap((a, b)) => {
                    let dest = self.read_parameter(a.clone())?;
                    let src = self.read_parameter(b.clone())?;
                    let data = self.heap.read(self.selected_frame, src)?;
                    self.memory.write().get_active_stack_mut()?.set(dest, data);
                }

                SharkyInstruction::CloneHeap => {
                    let address = self.heap.clone_frame(self.selected_frame)?;
                    self.memory.write().get_active_stack_mut()?.push(SharkyDataType::HeapReference(address));
                }

                SharkyInstruction::SizeHeap => {
                    let size = self.heap.size_frame(self.selected_frame)?;
                    self.memory.write().get_active_stack_mut()?.push(SharkyDataType::Max(size));
                }
                _ => {}
        } 
        Some(())
    }

    fn interpret_taskops(&mut self, op: &SharkyInstruction) -> Option<()> {
        match op {
            SharkyInstruction::SpawnSubtask(a) => {
                let param_a = self.read_parameter(a.clone())?;
                let subtask = self.new_subvm(param_a);
                self.task_pool.spawn_subtask(subtask);
            }
            SharkyInstruction::KillSelf => {
                self.running = false;
            }
            _ => {}
        }
        
        Some(())
    }

    fn interpret_ffiops(&mut self, op: &SharkyInstruction) -> Option<()> {
        match op {
            SharkyInstruction::FFICall(a) => {
                let param_a = self.read_parameter(a.clone())?;
                let function = self.ffi_function_table.get(param_a)?;
                SharkyFFILibrary::call_function(function, self.memory.read().get_parameter_stack().get_vec());
            }
            _ => {}
        }
        Some(())
    }
}

