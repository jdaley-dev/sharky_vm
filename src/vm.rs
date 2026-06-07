#![allow(dead_code)]
use parking_lot::{RwLock, RwLockReadGuard};
use std::{fmt::Display, sync::Arc};

use sharky_env::{collections::*, data_types::*, ffi::*};

use crate::{
    app::SharkyTaskPool,
    instructions::{SharkyInstruction::NoOperation, *},
    vm::SharkyInterrupt::TypeMismatch,
};

#[macro_use]
mod vm_macros;

pub enum SharkyInterrupt {
    OutOfOps = 0,
    InvalidParameter = 1,
    InvalidSelectedStack = 2,
    InvalidStackMode = 3,
    InvalidStackObjectIndex = 4,
    InvalidHeapIndex = 5,
    TypeMismatch = 6,
    InvalidConversion = 7,
    InvalidFFICall = 8,
    DivideByZero = 9,
    NonIndexValue = 10,
}

impl Display for SharkyInterrupt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SharkyInterrupt::OutOfOps => write!(f, "OutOfOps"),
            SharkyInterrupt::InvalidParameter => write!(f, "InvalidParameter"),
            SharkyInterrupt::InvalidSelectedStack => write!(f, "InvalidStackIndex"),
            SharkyInterrupt::InvalidStackMode => write!(f, "InvalidStackMode"),
            SharkyInterrupt::InvalidStackObjectIndex => write!(f, "InvalidStackObjectIndex"),
            SharkyInterrupt::InvalidHeapIndex => write!(f, "InvalidHeapIndex"),
            SharkyInterrupt::TypeMismatch => write!(f, "TypeMismatch"),
            SharkyInterrupt::InvalidConversion => write!(f, "InvalidConversion"),
            SharkyInterrupt::InvalidFFICall => write!(f, "InvalidFFICall"),
            SharkyInterrupt::DivideByZero => write!(f, "DivideByZero"),
            SharkyInterrupt::NonIndexValue => write!(f, "NonIndexValue"),
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
            NoOperation => (),

            SharkyInstruction::SelectStack(stack_index, mode) => {
                let stack_index: SharkyMax = self.read_parameter_val(stack_index)?;
                match mode {
                    SelectStackMode::Fixed => {
                        // Stack modes are stored in the FixedStackMode enum, bytecode representations need to be able to just post a value.
                        // this simplifies the bytecode interpreter as we can just parse simple bytes into the stack mode and take in the same
                        // kind of parameter. "stack_index" has the ability to just be a number and is interpreted based on the mode.
                        let u8_mode = stack_index as u8;
                        let fixed_stack: FixedStackMode = u8_mode
                            .try_into()
                            .map_err(|_| SharkyInterrupt::InvalidStackMode)?;
                        match fixed_stack {
                            FixedStackMode::Operative => self
                                .memory
                                .write()
                                .set_stack_mode(SharkyStackMode::Operative),
                            FixedStackMode::Transitional => self
                                .memory
                                .write()
                                .set_stack_mode(SharkyStackMode::Transitional),
                            FixedStackMode::Parameter => self
                                .memory
                                .write()
                                .set_stack_mode(SharkyStackMode::Parameter),
                        }
                    }
                    SelectStackMode::Indexed => {
                        let mut memory = self.memory.write();
                        memory.set_stack_mode(SharkyStackMode::Indexed);
                        // Design decision: The VM will let you SELECT an invalid stack, but throws an error if you try to use it.
                        // This is because technically a stack could become invalid even after selecting it, so it has to check
                        // any way. This may or may not be beneficial.
                        memory.select_local_stack(stack_index);
                    }
                }
            }
            SharkyInstruction::PushStack => self.memory.write().new_local_stack(),

            SharkyInstruction::PopStack => self.memory.write().pop_local_stack(),

            SharkyInstruction::CopyToTransition(src_) => {
                let src: SharkyMax = self.read_parameter_val(src_)?;
                let val = self.read_active_stack(src)?;
                self.memory.write().get_transitional_stack().push(val);
            }

            SharkyInstruction::CopyFromTransition(src_) => {
                let src: SharkyMax = self.read_parameter_val(src_)?;
                let value = self
                    .memory
                    .write()
                    .get_transitional_stack()
                    .read(src)
                    .ok_or(SharkyInterrupt::InvalidStackObjectIndex)?
                    .clone();
                self.push_active_stack(value)?;
            }

            SharkyInstruction::Push(value) => {
                self.push_active_stack(value)?;
            }
            SharkyInstruction::Convert(src_, mode) => {
                let src = self.read_parameter(src_)?;
                let converted = Self::convert_to(src, mode)?;
                self.push_active_stack(converted)?;
            }

            SharkyInstruction::Nilify(dest_) => {
                let dest: SharkyMax = self.read_parameter_val(dest_)?;
                self.write_active_stack(dest, SharkyDataType::Nil)?;
            }

            SharkyInstruction::Set(dest_, src_) => {
                let dest: SharkyMax = self.read_parameter_val(dest_)?;
                let src: SharkyMax = self.read_parameter_val(src_)?;
                let src_val = self.read_active_stack(src)?;
                self.write_active_stack(dest, src_val)?;
            }

            SharkyInstruction::Copy(src_) => {
                let src_index: SharkyMax = self.read_parameter_val(src_)?;
                let src_val = self.read_active_stack(src_index)?;
                self.push_active_stack(src_val)?;
            }

            SharkyInstruction::Pop => {
                self.memory
                    .write()
                    .get_active_stack_mut()
                    .ok_or(SharkyInterrupt::InvalidSelectedStack)?
                    .pop();
            }

            SharkyInstruction::Clear => {
                self.memory
                    .write()
                    .get_active_stack_mut()
                    .ok_or(SharkyInterrupt::InvalidSelectedStack)?
                    .clear();
            }

            SharkyInstruction::ArithmeticOp(lhs_, rhs_, mode) => {
                let lhs = self.read_parameter(lhs_)?;
                let rhs = self.read_parameter(rhs_)?;
                SharkyDataType::operate_if_are_type::<SharkyMax, _>(lhs, rhs, |a, b| {
                    SharkyDataType::Nil
                });
            }
            SharkyInstruction::BitwiseOp(lhs, rhs, mode) => todo!(),
            SharkyInstruction::ComparisonOp(lhs, rhs, mode) => todo!(),

            SharkyInstruction::PushBytes(dest, start, end) => todo!(),
            SharkyInstruction::SetByte(_) => todo!(),
            SharkyInstruction::ReadByteAs(_) => todo!(),

            SharkyInstruction::Goto(a) => {
                self.program_counter = self.read_parameter_val(a)?;
            }

            SharkyInstruction::LogicalGoto(a, b, condition) => todo!(),

            SharkyInstruction::Heap(mode) => todo!(),
            SharkyInstruction::SelectHeap(index) => todo!(),
            SharkyInstruction::CopyToHeapItem(dest, src) => todo!(),
            SharkyInstruction::CopyFromHeapItem(dest, src) => todo!(),

            SharkyInstruction::SpawnSubtask(a) => {
                let param_a: SharkyMax = self.read_parameter_val(a)?;
                let subtask = self.new_subvm(param_a);
                self.task_pool.spawn_subtask(subtask);
            }

            SharkyInstruction::EndTask => {
                self.running = false;
            }

            SharkyInstruction::FFICall(a) => {
                let param_a: SharkyMax = self.read_parameter_val(a)?;
                let function = self
                    .ffi_function_table
                    .get(param_a)
                    .ok_or(SharkyInterrupt::InvalidFFICall)?;
                SharkyFFILibrary::call_function(
                    function,
                    self.memory.read().get_parameter_stack().get_vec(),
                );
            }
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

    fn push_active_stack(&mut self, value: SharkyDataType) -> SharkyInterpreterStatus {
        self.memory
            .write()
            .get_active_stack_mut()
            .ok_or(SharkyInterrupt::InvalidSelectedStack)?
            .push(value);
        Ok(())
    }

    fn write_active_stack(
        &mut self,
        index: usize,
        value: SharkyDataType,
    ) -> SharkyInterpreterStatus {
        self.memory
            .write()
            .get_active_stack_mut()
            .ok_or(SharkyInterrupt::InvalidSelectedStack)?
            .set(index, value)
            .ok_or(SharkyInterrupt::InvalidStackObjectIndex)
    }

    fn read_active_stack(&self, index: usize) -> Result<SharkyDataType, SharkyInterrupt> {
        self.memory
            .read()
            .get_active_stack()
            .ok_or(SharkyInterrupt::InvalidSelectedStack)?
            .read(index)
            .cloned()
            .ok_or(SharkyInterrupt::InvalidStackObjectIndex)
    }

    fn convert_to(
        value: SharkyDataType,
        mode: ConversionMode,
    ) -> Result<SharkyDataType, SharkyInterrupt> {
        match mode {
            ConversionMode::Max => value.to_max().ok_or(SharkyInterrupt::TypeMismatch),
            ConversionMode::Int => value.to_int().ok_or(SharkyInterrupt::TypeMismatch),
            ConversionMode::Real => value.to_real().ok_or(SharkyInterrupt::TypeMismatch),
            ConversionMode::Byte => value.to_byte().ok_or(SharkyInterrupt::TypeMismatch),
            ConversionMode::Bool => value.to_bool().ok_or(SharkyInterrupt::TypeMismatch),
            ConversionMode::HeapReference => value
                .to_heap_reference()
                .ok_or(SharkyInterrupt::TypeMismatch),
            _ => Err(TypeMismatch),
        }
    }

    fn read_parameter(&self, parameter: OpParameter) -> Result<SharkyDataType, SharkyInterrupt> {
        match parameter {
            OpParameter::Constant(val) => Ok(val),
            OpParameter::Pointer(index) => {
                let ptr_index = SharkyMax::try_from(self.read_active_stack(index)?)
                    .map_err(|_| SharkyInterrupt::NonIndexValue)?;
                Ok(self.read_active_stack(ptr_index)?)
            }
            OpParameter::None => Err(SharkyInterrupt::InvalidParameter),
        }
    }

    fn read_parameter_val<T: SharkyValue>(
        &self,
        parameter: OpParameter,
    ) -> Result<T, SharkyInterrupt>
    where
        SharkyDataType: From<T> + TryInto<T> + Clone,
    {
        match parameter {
            OpParameter::Constant(val) => val
                .try_into()
                .map_err(|_| SharkyInterrupt::InvalidParameter),
            OpParameter::Pointer(index) => {
                let ptr_index = SharkyMax::try_from(self.read_active_stack(index)?)
                    .map_err(|_| SharkyInterrupt::NonIndexValue)?;
                Ok(self
                    .read_active_stack(ptr_index)?
                    .try_into()
                    .map_err(|_| SharkyInterrupt::TypeMismatch)?)
            }
            OpParameter::None => Err(SharkyInterrupt::InvalidParameter),
        }
    }
}
