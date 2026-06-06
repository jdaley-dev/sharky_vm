#![allow(dead_code)]
use parking_lot::{RwLock, RwLockReadGuard};
use std::{fmt::Display, sync::Arc};

use sharky_env::{collections::*, data_types::*, ffi::*};

use crate::{app::SharkyTaskPool, instructions::*};

#[macro_use]
mod vm_macros;

pub enum SharkyInterrupt {
    OutOfOps = 0,
    InvalidParameter = 1,
    InvalidStackIndex = 2,
    InvalidStackMode = 3,
    InvalidStackObjectIndex = 4,
    InvalidHeapIndex = 5,
    TypeMismatch = 6,
    InvalidConversion = 7,
    InvalidFFICall = 8,
    DivideByZero = 9,
}

impl Display for SharkyInterrupt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SharkyInterrupt::OutOfOps => write!(f, "OutOfOps"),
            SharkyInterrupt::InvalidParameter => write!(f, "InvalidParameter"),
            SharkyInterrupt::InvalidStackIndex => write!(f, "InvalidStackIndex"),
            SharkyInterrupt::InvalidStackMode => write!(f, "InvalidStackMode"),
            SharkyInterrupt::InvalidStackObjectIndex => write!(f, "InvalidStackObjectIndex"),
            SharkyInterrupt::InvalidHeapIndex => write!(f, "InvalidHeapIndex"),
            SharkyInterrupt::TypeMismatch => write!(f, "TypeMismatch"),
            SharkyInterrupt::InvalidConversion => write!(f, "InvalidConversion"),
            SharkyInterrupt::InvalidFFICall => write!(f, "InvalidFFICall"),
            SharkyInterrupt::DivideByZero => write!(f, "DivideByZero"),
        }
    }
}

pub type SharkyInterpreterStatus = Result<(), SharkyInterrupt>;

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
    pub fn new(
        program: Arc<SharkyProgram>,
        heap: SharkyHeap,
        task_pool: SharkyTaskPool,
        ffi_functions: Arc<Vec<SharkyFFIFunctionHandle>>,
    ) -> SharkyVM {
        SharkyVM {
            memory: RwLock::new(SharkyMemoryLayout::new()),
            program_counter: 0,
            program_memory: Arc::clone(&program),
            heap: heap,
            task_pool: task_pool,
            ffi_function_table: ffi_functions,
            selected_frame: 0,
            running: true,
        }
    }

    pub fn new_subvm(&self, program_counter: usize) -> SharkySyncedVM {
        let mut subvm = Self::new(
            self.program_memory.clone(),
            self.heap.clone(),
            self.task_pool.clone(),
            self.ffi_function_table.clone(),
        );
        let parameter_stack = self.memory.write().get_parameter_stack_mut().clone();
        subvm.program_counter = program_counter;
        subvm.memory.write().set_parameter_stack(&parameter_stack);
        Arc::new(RwLock::new(subvm))
    }

    pub fn has_reference(&self, address: SharkyHeapFrameIndex) -> bool {
        self.memory.read().has_heap_ref(address)
    }

    pub fn new_arc(
        program: Arc<SharkyProgram>,
        heap: SharkyHeap,
        task_pool: SharkyTaskPool,
        ffi_functions: Arc<Vec<SharkyFFIFunctionHandle>>,
    ) -> SharkySyncedVM {
        Arc::new(RwLock::new(SharkyVM::new(
            program,
            heap,
            task_pool,
            ffi_functions,
        )))
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

    pub fn stop(&mut self) {
        self.running = false;
    }

    fn read_parameter<T: SharkyValue>(&mut self, parameter: SharkyParameter<T>) -> Option<T>
    where
        SharkyDataType: TryInto<T> + Clone,
    {
        match parameter {
            SharkyParameter::Constant(val) => Some(val.into()),
            SharkyParameter::StackIndex(index) => Some(
                (self.memory.read().get_active_stack()?.read(index)?.clone())
                    .try_into()
                    .ok()?,
            ),
            SharkyParameter::None => None,
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

    pub fn interpret(&mut self) -> SharkyInterpreterStatus {
        if !self.is_running() {
            return Ok(());
        }

        let current_instruction = self
            .program_memory
            .as_ref()
            .get(self.program_counter)
            .ok_or(SharkyInterrupt::OutOfOps)?
            .clone();

        let prev_counter = self.program_counter;

        match current_instruction {
            // -----------------------------------------------------------------------
            // -----------------------------Begin Stack Ops---------------------------
            // -----------------------------------------------------------------------
            SharkyInstruction::SetStackMode(mode) => self.memory.write().set_stack_mode(mode),

            SharkyInstruction::SelectLocalStack(stack) => {
                let parameter = self
                    .read_parameter(stack)
                    .ok_or(SharkyInterrupt::InvalidParameter)?;
                self.memory.write().select_local_stack(parameter)
            }
            SharkyInstruction::PushLocalStack => self.memory.write().new_local_stack(),

            SharkyInstruction::PopLocalStack => self.memory.write().pop_local_stack(),

            SharkyInstruction::PushTransition(a) => {
                let param = self
                    .read_parameter(a)
                    .ok_or(SharkyInterrupt::InvalidParameter)?;
                let mut memory = self.memory.write();
                let stack = memory
                    .get_active_stack()
                    .ok_or(SharkyInterrupt::InvalidStackIndex)?;
                let value = stack
                    .read(param)
                    .ok_or(SharkyInterrupt::InvalidStackObjectIndex)?
                    .clone();
                memory.get_transitional_stack().push(value);
            }

            SharkyInstruction::CopyTransition(a) => {
                let param = self
                    .read_parameter(a)
                    .ok_or(SharkyInterrupt::InvalidParameter)?;
                let mut memory = self.memory.write();
                let value = memory
                    .get_transitional_stack()
                    .read(param)
                    .ok_or(SharkyInterrupt::InvalidStackObjectIndex)?
                    .clone();
                memory
                    .get_active_stack_mut()
                    .ok_or(SharkyInterrupt::InvalidStackIndex)?
                    .push(value);
            }

            SharkyInstruction::Copy(a) => {
                let param_a = self
                    .read_parameter(a)
                    .ok_or(SharkyInterrupt::InvalidParameter)?;
                let mut memory = self.memory.write();
                let stack = memory
                    .get_active_stack_mut()
                    .ok_or(SharkyInterrupt::InvalidStackIndex)?;
                let data = stack
                    .read(param_a)
                    .ok_or(SharkyInterrupt::InvalidStackObjectIndex)?
                    .clone();
                stack.push(data);
            }

            SharkyInstruction::Nilify(a) => {
                let param_a = self
                    .read_parameter(a)
                    .ok_or(SharkyInterrupt::InvalidParameter)?;
                let mut memory = self.memory.write();
                let stack = memory
                    .get_active_stack_mut()
                    .ok_or(SharkyInterrupt::InvalidStackIndex)?;
                stack
                    .set(param_a, SharkyDataType::Nil)
                    .ok_or(SharkyInterrupt::InvalidStackObjectIndex)?;
            }

            SharkyInstruction::Set((a, b)) => {
                let param_a = self
                    .read_parameter(a)
                    .ok_or(SharkyInterrupt::InvalidParameter)?; // INVALID PARAMETER
                let param_b = self
                    .read_parameter(b)
                    .ok_or(SharkyInterrupt::InvalidParameter)?; // INVALID PARAMETER
                let mut memory = self.memory.write();
                let stack = memory
                    .get_active_stack_mut()
                    .ok_or(SharkyInterrupt::InvalidParameter)?; // INVALID STACK INDEX
                stack.set(
                    param_a,
                    stack
                        .read(param_b)
                        .ok_or(SharkyInterrupt::InvalidStackIndex)?
                        .clone(),
                ); // INVALID STACK INDEX
            }

            SharkyInstruction::Pop => {
                self.memory
                    .write()
                    .get_active_stack_mut()
                    .ok_or(SharkyInterrupt::InvalidStackIndex)?
                    .pop(); // INVALID STACK INDEX
            }

            SharkyInstruction::Clear => {
                self.memory
                    .write()
                    .get_active_stack_mut()
                    .ok_or(SharkyInterrupt::InvalidStackIndex)?
                    .clear(); // INVALID STACK INDEX
            }
            // -----------------------------------------------------------------------
            // ----------------------------End Stack Ops------------------------------
            // -----------------------------------------------------------------------

            // -----------------------------------------------------------------------
            // ----------------------------Begin Push Ops-----------------------------
            // -----------------------------------------------------------------------
            SharkyInstruction::PushNil => self
                .push_constant(SharkyDataType::Nil)
                .ok_or(SharkyInterrupt::InvalidStackIndex)?, // INVALID STACK INDEX

            SharkyInstruction::PushReal(v) => {
                push_constant!(self, v.clone(), Real);
            }
            SharkyInstruction::PushMax(v) => {
                push_constant!(self, v.clone(), Max);
            }
            SharkyInstruction::PushInt(v) => {
                push_constant!(self, v.clone(), Int);
            }
            SharkyInstruction::PushByte(v) => {
                push_constant!(self, v.clone(), Byte);
            }
            SharkyInstruction::PushBool(v) => {
                push_constant!(self, v.clone(), Bool);
            }
            SharkyInstruction::PushHeapReference(v) => {
                push_constant!(self, v.clone(), HeapReference);
            }
            SharkyInstruction::PushByteString(v) => {
                push_constant!(self, v.clone(), ByteString)
            }
            // -----------------------------------------------------------------------
            // ----------------------------End Push Ops-------------------------------
            // -----------------------------------------------------------------------

            // -----------------------------------------------------------------------
            // ----------------------------Begin Conversion Ops-----------------------
            // -----------------------------------------------------------------------
            SharkyInstruction::ToInt(a) => {
                convert_match_impl!(self, a, stack,
                    SharkyDataType::Max(v) => stack.push(SharkyDataType::Int(*v as SharkyInt)),
                    SharkyDataType::Byte(v) => stack.push(SharkyDataType::Int(*v as SharkyInt)),
                ); // INVALID CONVERSION
            }

            SharkyInstruction::ToMax(a) => {
                convert_match_impl!(self, a, stack,
                    SharkyDataType::Int(v) => stack.push(SharkyDataType::Max(*v as SharkyMax)),
                    SharkyDataType::Byte(v) => stack.push(SharkyDataType::Max(*v as SharkyMax)),
                ); // INVALID CONVERSION
            }

            SharkyInstruction::ToByte(a) => {
                convert_match_impl!(self, a, stack,
                    SharkyDataType::Max(v) => stack.push(SharkyDataType::Byte(*v as SharkyByte)),
                    SharkyDataType::Int(v) => stack.push(SharkyDataType::Byte(*v as SharkyByte)),
                ); // INVALID CONVERSION
            }
            // -----------------------------------------------------------------------
            // ----------------------------End Conversion Ops-------------------------
            // -----------------------------------------------------------------------

            // -----------------------------------------------------------------------
            // -------------------------Begin Operative Ops---------------------------
            // -----------------------------------------------------------------------
            // TODO: INCOMPATIBLE COMPARISON
            SharkyInstruction::Add((a, b)) => {
                operational_binary_impl!(self, a, b, +, real);
            }
            SharkyInstruction::Subtract((a, b)) => {
                operational_binary_impl!(self, a, b, -, real);
            }
            SharkyInstruction::Multiply((a, b)) => {
                operational_binary_impl!(self, a, b, *, real);
            }
            SharkyInstruction::Divide((a, b)) => {
                if self
                    .read_parameter(b.clone())
                    .ok_or(SharkyInterrupt::InvalidParameter)?
                    == 0
                {
                    return Err(SharkyInterrupt::DivideByZero);
                }
                operational_binary_impl!(self, a, b, /, real);
            }
            SharkyInstruction::Modulus((a, b)) => {
                if self
                    .read_parameter(b.clone())
                    .ok_or(SharkyInterrupt::InvalidParameter)?
                    == 0
                {
                    return Err(SharkyInterrupt::DivideByZero);
                }
                operational_binary_impl!(self, a, b, %, real);
            }

            SharkyInstruction::BitLeftShift((a, b)) => {
                operational_binary_impl!(self, a, b, <<);
            }
            SharkyInstruction::BitRightShift((a, b)) => {
                operational_binary_impl!(self, a, b, >>);
            }
            SharkyInstruction::BitAnd((a, b)) => {
                operational_binary_impl!(self, a, b, &);
            }
            SharkyInstruction::BitXor((a, b)) => {
                operational_binary_impl!(self, a, b, ^);
            }
            SharkyInstruction::BitOr((a, b)) => {
                operational_binary_impl!(self, a, b, |);
            }
            SharkyInstruction::BitNot(a) => {
                operational_unary_impl!(self, a, !);
            }

            SharkyInstruction::Not(a) => {
                let param_a = self
                    .read_parameter(a)
                    .ok_or(SharkyInterrupt::InvalidParameter)?;
                let mut memory = self.memory.write();
                let opstack = memory.get_operational_stack();
                let val = opstack
                    .read(param_a)
                    .ok_or(SharkyInterrupt::InvalidStackObjectIndex)?;
                let result = match val {
                    SharkyDataType::Bool(a) => !a,
                    _ => false, // TODO: type mismatch interrupt
                };
                opstack.push(SharkyDataType::Bool(result));
            }

            SharkyInstruction::And((a, b)) => {
                operational_binary_boolean_impl!(self, a, b, &&);
            }
            SharkyInstruction::Or((a, b)) => {
                operational_binary_boolean_impl!(self, a, b, ||);
            }
            SharkyInstruction::Equals((a, b)) => {
                operational_binary_comparison_impl!(self, a, b, ==);
            }
            SharkyInstruction::NotEquals((a, b)) => {
                operational_binary_comparison_impl!(self, a, b, !=);
            }
            SharkyInstruction::GreaterThan((a, b)) => {
                operational_binary_comparison_impl!(self, a, b, >);
            }
            SharkyInstruction::LesserThan((a, b)) => {
                operational_binary_comparison_impl!(self, a, b, <);
            }
            SharkyInstruction::GreaterThanOrEquals((a, b)) => {
                operational_binary_comparison_impl!(self, a, b, >=);
            }
            SharkyInstruction::LesserThanOrEquals((a, b)) => {
                operational_binary_comparison_impl!(self, a, b, <=);
            }
            // -----------------------------------------------------------------------
            // ---------------------------End Operative Ops---------------------------
            // -----------------------------------------------------------------------

            // -----------------------------------------------------------------------
            // ----------------------------Begin Logic Ops----------------------------
            // -----------------------------------------------------------------------
            SharkyInstruction::Jump(a) => {
                self.program_counter = self
                    .read_parameter(a)
                    .ok_or(SharkyInterrupt::InvalidParameter)?;
            }

            SharkyInstruction::JumpIfNot((a, b)) => {
                let param_a = self
                    .read_parameter(a)
                    .ok_or(SharkyInterrupt::InvalidParameter)?; // INVALID PARAM
                let param_b = self
                    .read_parameter(b)
                    .ok_or(SharkyInterrupt::InvalidParameter)?; // INVALID PARAM
                let mut memory = self.memory.write();
                let read = memory
                    .get_active_stack_mut()
                    .ok_or(SharkyInterrupt::InvalidStackIndex)?
                    .read(param_b)
                    .ok_or(SharkyInterrupt::InvalidStackObjectIndex)?;
                let mut jump = false;
                match read {
                    SharkyDataType::Bool(a) => {
                        jump = !a;
                    }
                    _ => {}
                }
                if jump {
                    self.program_counter = param_a;
                }
            }
            // -----------------------------------------------------------------------
            // ---------------------------End Logic Ops-------------------------------
            // -----------------------------------------------------------------------

            // -----------------------------------------------------------------------
            // ---------------------------Begin Heap Ops------------------------------
            // -----------------------------------------------------------------------
            SharkyInstruction::CreateHeap => {
                let address = self.heap.allocate();
                self.memory
                    .write()
                    .get_active_stack_mut()
                    .ok_or(SharkyInterrupt::InvalidStackIndex)?
                    .push(SharkyDataType::HeapReference(address));
            }

            SharkyInstruction::SelectHeap(a) => {
                let select = self
                    .read_parameter(a)
                    .ok_or(SharkyInterrupt::InvalidParameter)?;
                self.selected_frame = select;
            }

            SharkyInstruction::PushHeap => {
                self.heap.push(self.selected_frame, &SharkyDataType::Nil);
            }

            SharkyInstruction::WriteHeap((a, b)) => {
                let dest = self
                    .read_parameter(a)
                    .ok_or(SharkyInterrupt::InvalidParameter)?;
                let src = self
                    .read_parameter(b)
                    .ok_or(SharkyInterrupt::InvalidParameter)?;
                let data = self
                    .memory
                    .read()
                    .get_active_stack()
                    .ok_or(SharkyInterrupt::InvalidStackIndex)?
                    .read(src)
                    .ok_or(SharkyInterrupt::InvalidStackObjectIndex)?
                    .clone();
                self.heap
                    .set(self.selected_frame, dest, &data)
                    .ok_or(SharkyInterrupt::InvalidHeapIndex)?;
            }

            SharkyInstruction::ReadHeap((a, b)) => {
                let dest = self
                    .read_parameter(a)
                    .ok_or(SharkyInterrupt::InvalidParameter)?; // INVALID PARAM
                let src = self
                    .read_parameter(b)
                    .ok_or(SharkyInterrupt::InvalidParameter)?; // INVALID PARAM
                let data = self
                    .heap
                    .read(self.selected_frame, src)
                    .ok_or(SharkyInterrupt::InvalidHeapIndex)?; // INVALID HEAP INDEX
                self.memory
                    .write()
                    .get_active_stack_mut()
                    .ok_or(SharkyInterrupt::InvalidStackIndex)?
                    .set(dest, data)
                    .ok_or(SharkyInterrupt::InvalidStackObjectIndex)?; // INVALID STACK INDEX / INVALID STACK OBJECT INDEX
            }

            SharkyInstruction::CloneHeap => {
                let address = self
                    .heap
                    .clone_frame(self.selected_frame)
                    .ok_or(SharkyInterrupt::InvalidHeapIndex)?;
                self.memory
                    .write()
                    .get_active_stack_mut()
                    .ok_or(SharkyInterrupt::InvalidStackIndex)?
                    .push(SharkyDataType::HeapReference(address));
            }

            SharkyInstruction::SizeHeap => {
                let size = self
                    .heap
                    .size_frame(self.selected_frame)
                    .ok_or(SharkyInterrupt::InvalidHeapIndex)?;
                self.memory
                    .write()
                    .get_active_stack_mut()
                    .ok_or(SharkyInterrupt::InvalidHeapIndex)?
                    .push(SharkyDataType::Max(size));
            }
            // -----------------------------------------------------------------------
            // ----------------------------End Heap Ops-------------------------------
            // -----------------------------------------------------------------------

            // -----------------------------------------------------------------------
            // ----------------------------Begin Task Ops-----------------------------
            // -----------------------------------------------------------------------
            SharkyInstruction::SpawnSubtask(a) => {
                let param_a = self
                    .read_parameter(a)
                    .ok_or(SharkyInterrupt::InvalidParameter)?; // INVALID PARAM
                let subtask = self.new_subvm(param_a);
                self.task_pool.spawn_subtask(subtask);
            }
            SharkyInstruction::EndTask => {
                self.running = false;
            }
            // -----------------------------------------------------------------------
            // -----------------------------End Task Ops------------------------------
            // -----------------------------------------------------------------------

            // -----------------------------------------------------------------------
            // -----------------------------Begin FFI Ops-----------------------------
            // -----------------------------------------------------------------------
            SharkyInstruction::FFICall(a) => {
                let param_a = self
                    .read_parameter(a)
                    .ok_or(SharkyInterrupt::InvalidParameter)?; // INVALID PARAM
                let function = self
                    .ffi_function_table
                    .get(param_a)
                    .ok_or(SharkyInterrupt::InvalidFFICall)?; // INVALID FFI CALL
                SharkyFFILibrary::call_function(
                    function,
                    self.memory.read().get_parameter_stack().get_vec(),
                );
            }
            // -----------------------------------------------------------------------
            // -----------------------------End FFI Ops-------------------------------
            // -----------------------------------------------------------------------
            _ => {}
        }

        // as long as a jump didn't occur, increase the program counter.
        if self.program_counter == prev_counter {
            self.program_counter += 1;
        }

        let program_memory_length = self.program_memory.as_ref().len();
        if self.program_counter >= program_memory_length {
            self.running = false;
        }
        Ok(())
    }
}
