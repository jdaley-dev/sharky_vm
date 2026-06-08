#![allow(dead_code)]
use parking_lot::{RwLock, RwLockReadGuard};
use std::{fmt::Display, sync::Arc};

use sharky_env::{collections::*, data_types::*, ffi::*, ffi_collections::CVec};

use crate::{
    app::SharkyTaskPool,
    instructions::{SharkyInstruction::NoOperation, *},
    vm::SharkyInterrupt::{InvalidOp, TypeMismatch},
};

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
    InvalidOp = 11,
    ByteStringOpOnNonBytestring = 12,
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
            SharkyInterrupt::InvalidOp => write!(f, "OperationalyIncompatible"),
            SharkyInterrupt::ByteStringOpOnNonBytestring => {
                write!(f, "ByteStringOpOnNonBytestring")
            }
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
                let src = self.read_parameter(src_)?;
                self.write_active_stack(dest, src)?;
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

            SharkyInstruction::FillByteStringWithValue(dest_, val_) => {
                let dest: SharkyMax = self.read_parameter_val(dest_)?;
                let val: SharkyByte = self.read_parameter_val(val_)?;
                self.mutate_byte_string(dest, |string| {
                    string.fill(val);
                    Ok(())
                })?;
            }

            SharkyInstruction::ExtendByteString(dest_, len_) => {
                let dest: SharkyMax = self.read_parameter_val(dest_)?;
                let len: SharkyMax = self.read_parameter_val(len_)?;
                self.mutate_byte_string(dest, |string| {
                    string.resize_with(string.len() + len, || 0);
                    Ok(())
                })?;
            }

            SharkyInstruction::CopyToByteString(dest_, src_) => {
                let dest: SharkyMax = self.read_parameter_val(dest_)?;
                let src: SharkyByte = self.read_parameter_val(src_)?;
                self.mutate_byte_string(dest, |string| {
                    let dest_ref = string
                        .get_mut(dest)
                        .ok_or(SharkyInterrupt::InvalidStackObjectIndex)?; // TODO: This might call for it's own interrupt.
                    *dest_ref = src;
                    Ok(())
                })?;
            }

            SharkyInstruction::CopyFromByteString(dest_, src_) => {
                let dest: SharkyMax = self.read_parameter_val(dest_)?;
                let src: SharkyMax = self.read_parameter_val(src_)?;
                let mut output: SharkyDataType = SharkyDataType::Nil;
                self.mutate_byte_string(src, |string| {
                    let dest_ref = string
                        .get(dest)
                        .ok_or(SharkyInterrupt::InvalidStackObjectIndex)?; // TODO: This might call for it's own interrupt.
                    output = SharkyDataType::Byte(*dest_ref);
                    Ok(())
                })?;
                self.write_active_stack(dest, output)?;
            }

            SharkyInstruction::AppendByteString(dest_, src_) => {
                let dest: SharkyMax = self.read_parameter_val(dest_)?;
                let src: SharkyMax = self.read_parameter_val(src_)?;

                let mut copy_data: Vec<u8> = Vec::new();
                self.mutate_byte_string(src, |string| {
                    copy_data.copy_from_slice(string);
                    Ok(())
                })?;

                self.mutate_byte_string(dest, move |string| {
                    string.append(&mut copy_data);
                    Ok(())
                })?;
            }

            SharkyInstruction::ClearByteString(string_) => {
                let string_index: SharkyMax = self.read_parameter_val(string_)?;
                self.mutate_byte_string(string_index, |string| {
                    string.clear();
                    Ok(())
                })?;
            }

            SharkyInstruction::ByteStringSize(string_) => {
                let string_index: SharkyMax = self.read_parameter_val(string_)?;
                let mut size: SharkyMax = 0;
                self.mutate_byte_string(string_index, |string| {
                    size = string.len();
                    Ok(())
                })?;
                self.push_active_stack(SharkyDataType::Max(size))?;
            }

            SharkyInstruction::SliceByteString(src_, begin_, end_) => {
                let src: SharkyMax = self.read_parameter_val(src_)?;
                let begin: SharkyMax = self.read_parameter_val(begin_)?;
                let end: SharkyMax = self.read_parameter_val(end_)?;

                let mut output: Vec<u8> = Vec::new();
                self.mutate_byte_string(src, |string| {
                    let len = string.len();
                    if begin > end || begin > len || end > len {
                        return Err(SharkyInterrupt::InvalidStackObjectIndex);
                    }
                    let slice = &string[begin..end];
                    output = slice.to_vec();
                    Ok(())
                })?;
                self.push_active_stack(SharkyDataType::ByteString(CVec::from(output)))?;
            }

            SharkyInstruction::ArithmeticOp(lhs_, rhs_, mode) => {
                let lhs = self.read_parameter(lhs_)?;
                let rhs = self.read_parameter(rhs_)?;
                match mode {
                    ArithmeticMode::Add => {
                        self.push_operational_stack(
                            SharkyDataType::try_add(lhs, rhs).ok_or(SharkyInterrupt::InvalidOp)?,
                        );
                    }
                    ArithmeticMode::Subtract => {
                        self.push_operational_stack(
                            SharkyDataType::try_subtract(lhs, rhs)
                                .ok_or(SharkyInterrupt::InvalidOp)?,
                        );
                    }
                    ArithmeticMode::Multiply => {
                        self.push_operational_stack(
                            SharkyDataType::try_multiply(lhs, rhs)
                                .ok_or(SharkyInterrupt::InvalidOp)?,
                        );
                    }
                    ArithmeticMode::Divide => {
                        self.push_operational_stack(
                            SharkyDataType::try_divide(lhs, rhs)
                                .ok_or(SharkyInterrupt::InvalidOp)?,
                        );
                    }
                    ArithmeticMode::Mod => {
                        self.push_operational_stack(
                            SharkyDataType::try_mod(lhs, rhs).ok_or(SharkyInterrupt::InvalidOp)?,
                        );
                    }
                }
            }

            SharkyInstruction::BitwiseOp(lhs_, rhs_, mode) => {
                let lhs = self.read_parameter(lhs_)?;
                let rhs = self.read_parameter(rhs_)?;
                match mode {
                    BitwiseMode::ShiftLeft => {
                        self.push_operational_stack(
                            SharkyDataType::try_shift_left(lhs, rhs)
                                .ok_or(SharkyInterrupt::InvalidOp)?,
                        );
                    }
                    BitwiseMode::ShiftRight => {
                        self.push_operational_stack(
                            SharkyDataType::try_shift_right(lhs, rhs)
                                .ok_or(SharkyInterrupt::InvalidOp)?,
                        );
                    }
                    BitwiseMode::And => {
                        self.push_operational_stack(
                            SharkyDataType::try_bitwise_and(lhs, rhs)
                                .ok_or(SharkyInterrupt::InvalidOp)?,
                        );
                    }
                    BitwiseMode::Or => {
                        self.push_operational_stack(
                            SharkyDataType::try_bitwise_or(lhs, rhs)
                                .ok_or(SharkyInterrupt::InvalidOp)?,
                        );
                    }
                    BitwiseMode::Xor => {
                        self.push_operational_stack(
                            SharkyDataType::try_bitwise_xor(lhs, rhs)
                                .ok_or(SharkyInterrupt::InvalidOp)?,
                        );
                    }
                    BitwiseMode::Not => {
                        self.push_operational_stack(
                            SharkyDataType::try_bitwise_not(lhs)
                                .ok_or(SharkyInterrupt::InvalidOp)?,
                        );
                    }
                }
            }

            SharkyInstruction::ComparisonOp(lhs_, rhs_, mode) => {
                let lhs = self.read_parameter(lhs_)?;
                let rhs = self.read_parameter(rhs_)?;
                match mode {
                    ComparisonMode::And => {
                        self.push_operational_stack(
                            if let SharkyDataType::Bool(lbool) = lhs
                                && let SharkyDataType::Bool(rbool) = rhs
                            {
                                SharkyDataType::Bool(lbool == rbool)
                            } else {
                                return Err(InvalidOp);
                            },
                        );
                    }
                    ComparisonMode::Or => {
                        self.push_operational_stack(
                            if let SharkyDataType::Bool(lbool) = lhs
                                && let SharkyDataType::Bool(rbool) = rhs
                            {
                                SharkyDataType::Bool(lbool || rbool)
                            } else {
                                return Err(InvalidOp);
                            },
                        );
                    }

                    ComparisonMode::Equals => {
                        self.push_operational_stack(
                            SharkyDataType::try_equals(lhs, rhs)
                                .ok_or(SharkyInterrupt::InvalidOp)?,
                        );
                    }
                    ComparisonMode::NotEquals => {
                        if let SharkyDataType::Bool(val) = SharkyDataType::try_equals(lhs, rhs)
                            .ok_or(SharkyInterrupt::InvalidOp)?
                        {
                            self.push_operational_stack(SharkyDataType::Bool(!val));
                        }
                    }
                    ComparisonMode::GreaterThan => {
                        self.push_operational_stack(
                            SharkyDataType::try_greater_than(lhs, rhs)
                                .ok_or(SharkyInterrupt::InvalidOp)?,
                        );
                    }
                    ComparisonMode::LessThan => {
                        self.push_operational_stack(
                            SharkyDataType::try_less_than(lhs, rhs)
                                .ok_or(SharkyInterrupt::InvalidOp)?,
                        );
                    }
                    ComparisonMode::GreaterThanOrEquals => {
                        if let SharkyDataType::Bool(greater_than) =
                            SharkyDataType::try_greater_than(lhs.clone(), rhs.clone())
                                .ok_or(SharkyInterrupt::InvalidOp)?
                        {
                            if let SharkyDataType::Bool(equal_to) =
                                SharkyDataType::try_equals(lhs, rhs)
                                    .ok_or(SharkyInterrupt::InvalidOp)?
                            {
                                self.push_operational_stack(SharkyDataType::Bool(
                                    greater_than || equal_to,
                                ));
                            }
                        }
                    }
                    ComparisonMode::LessThanOrEquals => {
                        if let SharkyDataType::Bool(less_than) =
                            SharkyDataType::try_less_than(lhs.clone(), rhs.clone())
                                .ok_or(SharkyInterrupt::InvalidOp)?
                        {
                            if let SharkyDataType::Bool(equal_to) =
                                SharkyDataType::try_equals(lhs, rhs)
                                    .ok_or(SharkyInterrupt::InvalidOp)?
                            {
                                self.push_operational_stack(SharkyDataType::Bool(
                                    less_than || equal_to,
                                ));
                            }
                        }
                    }
                }
            }

            SharkyInstruction::NotBool(val_) => {
                let val: SharkyBool = self.read_parameter_val(val_)?;
                self.push_active_stack(SharkyDataType::Bool(!val))?;
            }

            SharkyInstruction::Goto(a) => {
                self.program_counter = self.read_parameter_val(a)?;
            }

            SharkyInstruction::LogicalGoto(to_, check_, condition) => {
                let check: SharkyBool = self.read_parameter_val(check_)?;
                let to: SharkyMax = self.read_parameter_val(to_)?;
                match condition {
                    LogicMode::If => {
                        if check {
                            self.program_counter = to
                        }
                    }
                    LogicMode::IfNot => {
                        if !check {
                            self.program_counter = to
                        }
                    }
                }
            }

            SharkyInstruction::Heap(mode) => match mode {
                HeapOpMode::Create => {
                    let address = self.heap.allocate();
                    self.push_active_stack(SharkyDataType::HeapReference(address))?;
                }

                HeapOpMode::Clone => {
                    let address = self
                        .heap
                        .clone_frame(self.selected_frame)
                        .ok_or(SharkyInterrupt::InvalidHeapIndex)?;
                    self.push_active_stack(SharkyDataType::HeapReference(address))?;
                }
                HeapOpMode::NewItem => {
                    self.heap
                        .push(self.selected_frame, &SharkyDataType::Nil)
                        .ok_or(SharkyInterrupt::InvalidHeapIndex)?;
                }
                HeapOpMode::Size => {
                    let size = self
                        .heap
                        .size_frame(self.selected_frame)
                        .ok_or(SharkyInterrupt::InvalidHeapIndex)?;
                    self.push_active_stack(SharkyDataType::Max(size))?;
                }
            },

            SharkyInstruction::SelectHeap(index_) => {
                let index: SharkyMax = self.read_parameter_val(index_)?;
                self.selected_frame = index;
            }

            SharkyInstruction::CopyToHeapItem(dest_, src_) => {
                let dest: SharkyMax = self.read_parameter_val(dest_)?;
                let src = self.read_parameter(src_)?;
                self.heap
                    .set(self.selected_frame, dest, &src)
                    .ok_or(SharkyInterrupt::InvalidHeapIndex)?;
            }

            SharkyInstruction::CopyFromHeapItem(dest_, src_) => {
                let dest: SharkyMax = self.read_parameter_val(dest_)?;
                let src: SharkyMax = self.read_parameter_val(src_)?;
                let heap_val = self
                    .heap
                    .read(self.selected_frame, src)
                    .ok_or(SharkyInterrupt::InvalidHeapIndex)?;
                self.write_active_stack(dest, heap_val)?;
            }

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

    fn mutate_byte_string<F: FnOnce(&mut Vec<u8>) -> SharkyInterpreterStatus>(
        &mut self,
        index: SharkyMax,
        operate: F,
    ) -> SharkyInterpreterStatus {
        let mut memory = self.memory.write();
        let stack = memory
            .get_active_stack_mut()
            .ok_or(SharkyInterrupt::InvalidStackMode)?;
        let stack_vec = stack.get_vec_mut();

        let data_bytestring = stack_vec
            .get_mut(index)
            .ok_or(SharkyInterrupt::InvalidStackObjectIndex)?;

        let SharkyDataType::ByteString(bytestring) = data_bytestring else {
            return Err(SharkyInterrupt::ByteStringOpOnNonBytestring);
        };

        let mut operator = bytestring.get_operator_mut();
        operate(&mut *operator)?;

        Ok(())
    }

    fn push_operational_stack(&mut self, value: SharkyDataType) {
        self.memory.write().get_operational_stack().push(value);
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

    /// Returns the direct value at an index.
    /// ## Notes
    /// Typically most of the time `dest` is an index while `src` is just a value. So typically `src` would be read_parameter
    /// while `dest` would be read_parameter_val (tries to cast the value directly to a type I.E SharkyDataType::Max() to SharkyMax)
    /// It can be easy to get these confused but remember you can never write to a read, so if you're writing you have to get an index,
    /// but if you're reading you can just take a value.
    ///
    /// A clear example: Set(dest_, src_)
    /// dest_ is where we want to place our value (an index on the stack)
    /// src_ is what we want to put there. Now we could pass 3 things: a Constant, an Index, or a Pointer
    /// - Constant: Just a plain value, used for constants in bytecode
    /// - Index: The value at a stack-cell.
    /// - Pointer: The cell pointed to by another stack cell (ptr a = &b, *a = value of b)
    ///
    /// Whereas dest_ will also take these parameters but they'd all typically refer to one thing: a SharkyMax index into the active stack.
    /// - Constant: A plain static index into the stack
    /// - Index: An index stored in a stack cell
    /// - Pointer: Same as above, just with one level of indirection.
    ///
    /// These semantics change with some of the ByteString and HeapReference ops. This is because they're pointers so you always just index into them.
    /// This will tend to be the only time you'll see src also being an index- because the src itself is a pointer. This can get confusing, and might
    /// call for a future refactor with the interpreter. V0 shipped with this so only time will tell.
    fn read_parameter(&self, parameter: OpParameter) -> Result<SharkyDataType, SharkyInterrupt> {
        match parameter {
            OpParameter::Constant(val) => Ok(val),
            OpParameter::Index(val) => Ok(self.read_active_stack(val)?),
            OpParameter::Pointer(index) => {
                let ptr_index = SharkyMax::try_from(self.read_active_stack(index)?)
                    .map_err(|_| SharkyInterrupt::NonIndexValue)?;
                Ok(self.read_active_stack(ptr_index)?)
            }
            OpParameter::None => Ok(SharkyDataType::Nil),
        }
    }

    /// Like `read_parameter` but attempts to coerce the value into a type like `SharkyInt`.
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
            OpParameter::Index(val) => self
                .read_active_stack(val)?
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
