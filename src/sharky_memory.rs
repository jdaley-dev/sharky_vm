#![allow(dead_code)]
use std::{fmt::Display, sync::{Arc}};
use parking_lot::{RawRwLock, RwLock, RwLockWriteGuard, lock_api::RwLockReadGuard};
use slab::Slab;
use crate::{sharky_data_types::*, sharky_vm::{SharkyVM}};


pub trait TypicalData: Clone + Default + std::cmp::PartialEq {}
impl<T> TypicalData for T where T: Clone + Default + std::cmp::PartialEq {}

#[derive(Default, Clone)]
pub struct SharkyStack<T> {
    stack: Vec<T>,
}

impl<T: TypicalData> SharkyStack<T> {
    pub fn default() -> Self {
        Self { 
            stack: Vec::new()
        }
    }
    
    pub fn get_vec(&self) -> &Vec<T> {
        &self.stack
    }

    pub fn search(&self, value: &T) -> bool {
        self.stack.iter().any(move |v| { *v == *value })
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.stack.iter()
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn push(&mut self, data: T) {
        self.stack.push(data);
    }

    pub fn set(&mut self, index: usize, data: T) {
        if let Some(val) = self.stack.get_mut(index) {
            *val = data;
        }
    }

    pub fn read(&self, index: usize) -> Option<&T> {
        self.stack.get(index)
    }

    pub fn read_top(&self) -> Option<&T> {
        self.read(self.size() - 1)
    }

    pub fn size(&self) -> usize {
        self.stack.len()
    }

    pub fn clear(&mut self) {
        self.stack.clear();
    }
}

impl<T: TypicalData + Display> SharkyStack<T> {
        pub fn debug_print_stack(&self) {
        let mut point = 0;
        println!("--- SHARKY STACK DEBUG PRINT ---");
        for i in self.stack.iter() {
            println!("{point} - [{i}]\n--------------------------------");
            point += 1;
        }
        
    }
}

#[derive(Default)]
pub struct SharkyMemoryLayout {
    operational_stack: SharkyDataStack,
    transitional_stack: SharkyDataStack,
    parameter_stack: SharkyDataStack,
    string_stack: SharkyDataStack,

    local_stacks: Vec<SharkyDataStack>,
    selected_local_stack: usize,
    stack_mode: SharkyStackMode,
}

impl SharkyMemoryLayout {
    pub fn new() -> Self { 
        let mut result = Self::default();
        result.local_stacks.push(SharkyDataStack::default()); // initialize the local stacks with a minimum of one stack.
        result 
    }
    
    pub fn has_heap_ref(&self, index: SharkyHeapFrameIndex) -> bool {
        let heap_value = SharkyDataType::HeapReference(index);
        self.transitional_stack.search(&heap_value) ||
        self.operational_stack.search(&heap_value) ||
        self.parameter_stack.search(&heap_value) ||
        self.local_stacks.iter().any(move |v| { v.search(&heap_value) }) 
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

    pub fn get_parameter_stack_mut(&mut self) -> &mut SharkyDataStack {
        &mut self.parameter_stack
    }    
    
    pub fn get_parameter_stack(&self) -> &SharkyDataStack {
        &self.parameter_stack
    }

    pub fn get_operational_stack(&mut self) -> &mut SharkyDataStack {
        &mut self.operational_stack
    }

    pub fn set_parameter_stack(&mut self, stack: &SharkyDataStack) {
        self.parameter_stack = stack.clone();
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
            SharkyStackMode::Parameter => {
                Some(&mut self.parameter_stack)
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
            SharkyStackMode::Parameter => {
                Some(&self.parameter_stack)
            }
            _ => None,
        }
    }
}

pub type SharkyHeapFrame = RwLock<SharkyDataStack>;
#[derive(Default, Clone)]
pub struct SharkyHeap {
    memory: SharkySynced<Slab<SharkyHeapFrame>>,
    allocation_count: SharkySynced<usize>,
}

impl SharkyHeap {
    pub fn default() -> Self {
        Self {
            memory: Arc::new(RwLock::new(Slab::new())),
            allocation_count: Arc::new(RwLock::new(0)),
        }
    }

    pub fn new() -> Self {Self::default()}

    fn get_memory(&self) -> RwLockReadGuard<'_, RawRwLock, Slab<RwLock<SharkyDataStack>>> {
        self.memory.read()
    }

    pub fn print_debug(&self) {
        let memory = self.memory.read();
        for (_, frame) in memory.iter() {
            println!("---SHARKY HEAP FRAME---");
            for cell in frame.read().iter() {
                println!("-- SHARKY CELL --");
                println!("{cell}");
            }
        }
    }

    pub fn get_allocation_count(&self) -> usize {
        *self.allocation_count.read()
    }

    pub fn reset_allocation_count(&mut self) {
        *self.allocation_count.write() = 0;
    }

    pub fn allocate(&mut self) -> SharkyHeapFrameIndex {
        *self.allocation_count.write() += 1;
        self.memory.write().insert(RwLock::new(SharkyDataStack::default()))
    }

    pub fn free(&mut self, frame: SharkyHeapFrameIndex) {
        self.memory.write().remove(frame);
    }

    pub fn push(&self, frame_addr: SharkyHeapFrameIndex, data: &SharkyDataType) -> Option<()> {
        let memory = self.get_memory();
        let mut frame = memory.get(frame_addr)?.write();
        frame.push(data.clone());
        Some(())
    }

    pub fn read(&self, frame_addr: SharkyHeapFrameIndex, index: SharkyHeapCellIndex) -> Option<SharkyDataType> {
        let memory = self.get_memory();
        let frame = memory.get(frame_addr)?.read();
        let value = frame.read(index)?;
        Some(value.clone())
    }

    pub fn take_frame(&mut self, frame: SharkyDataStack) -> SharkyHeapFrameIndex {
        self.memory.write().insert(RwLock::new(frame))
    }

    pub fn get_frame_clone(&self, frame_addr: SharkyHeapFrameIndex) -> Option<SharkyDataStack> {
        Some(self.memory.read().get(frame_addr)?.read().clone())
    }

    pub fn set(&self, frame_addr: SharkyHeapFrameIndex, index: SharkyHeapCellIndex, data: &SharkyDataType) -> Option<()> {
        let memory = self.get_memory();
        let mut frame = memory.get(frame_addr)?.write();
        frame.set(index, data.clone());
        Some(())
    }
    
    pub fn clone_frame(&mut self, frame_addr: SharkyHeapFrameIndex) -> Option<SharkyHeapFrameIndex>{
        let mut memory = self.memory.write();
        let frame = memory.get(frame_addr)?.read().clone();
        *self.allocation_count.write() += 1;
        Some(memory.insert(RwLock::new(frame)))
    }

    pub fn size_frame(&self, frame_addr: SharkyHeapFrameIndex) -> Option<usize> {
        let memory = self.get_memory();
        Some(memory.get(frame_addr)?.read().size())
    }

    pub fn collect_heap_indexes(&self) -> Vec<SharkyHeapFrameIndex> {
        self.memory.read().iter().map(|(key, _)| {key}).collect()
    }

    pub fn has_reference(&self, reference: SharkyHeapFrameIndex) -> bool {
        for (_, frame) in self.memory.read().iter() {
            if frame.read().search(&SharkyDataType::HeapReference(reference)) { return true; }
        }
        false 
    }

}


pub type SharkyDataStack = SharkyStack<SharkyDataType>;