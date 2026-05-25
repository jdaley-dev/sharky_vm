#![allow(dead_code)]
use std::{fmt::Display, sync::{RwLock}};
use crate::sharky_data_types::*;

pub trait TypicalData: Clone + Default {}
impl<T> TypicalData for T where T: Clone + Default {}

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

pub type SharkyDataStack = SharkyStack<SharkyDataType>;


#[derive(Default, Clone)]
pub enum SharkyHeapCell {
    DynamicCell(SharkyDataStack),
    
    ByteCell(SharkyStack<SharkyByte>),
    IntCell(SharkyStack<SharkyInt>),
    MaxCell(SharkyStack<SharkyMax>),
    RealCell(SharkyStack<SharkyReal>),
    
    #[default]
    EmptyCell
}

pub trait SharkyHeapExtractable: TypicalData + Sized {
    fn extract(cell: &SharkyHeapCell) -> Option<&SharkyStack<Self>>;
    fn extract_mut(cell: &mut SharkyHeapCell) -> Option<&mut SharkyStack<Self>>;
}

impl SharkyHeapExtractable for SharkyByte {
    fn extract(cell: &SharkyHeapCell) -> Option<&SharkyStack<Self>> {
        match cell {
            SharkyHeapCell::ByteCell(cell) => Some(cell), 
            _ => None
        }
    }

    fn extract_mut(cell: &mut SharkyHeapCell) -> Option<&mut SharkyStack<Self>> {
        match cell {
            SharkyHeapCell::ByteCell(cell) => Some(cell), 
            _ => None
        }
    }
}

impl SharkyHeapExtractable for SharkyInt {
    fn extract(cell: &SharkyHeapCell) -> Option<&SharkyStack<Self>> {
        match cell {
            SharkyHeapCell::IntCell(cell) => Some(cell), 
            _ => None
        }
    }

    fn extract_mut(cell: &mut SharkyHeapCell) -> Option<&mut SharkyStack<Self>> {
        match cell {
            SharkyHeapCell::IntCell(cell) => Some(cell), 
            _ => None
        }
    }
}

impl SharkyHeapExtractable for SharkyMax {
    fn extract(cell: &SharkyHeapCell) -> Option<&SharkyStack<Self>> {
        match cell {
            SharkyHeapCell::MaxCell(cell) => Some(cell), 
            _ => None
        }
    }

    fn extract_mut(cell: &mut SharkyHeapCell) -> Option<&mut SharkyStack<Self>> {
        match cell {
            SharkyHeapCell::MaxCell(cell) => Some(cell), 
            _ => None
        }
    }
}

impl SharkyHeapExtractable for SharkyReal {
    fn extract(cell: &SharkyHeapCell) -> Option<&SharkyStack<Self>> {
        match cell {
            SharkyHeapCell::RealCell(cell) => Some(cell), 
            _ => None
        }
    }

    fn extract_mut(cell: &mut SharkyHeapCell) -> Option<&mut SharkyStack<Self>> {
        match cell {
            SharkyHeapCell::RealCell(cell) => Some(cell), 
            _ => None
        }
    }
}

#[derive(Default)]
pub struct SharkyHeap {
    heap: Vec<RwLock<SharkyHeapCell>>,
}

impl SharkyHeap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get<T: SharkyHeapExtractable>(&self, address: SharkyHeapAddress) -> Option<T> {
        let frame = self.heap.get(address.frame)?.read().unwrap();
        Some(T::extract(&frame)?.read(address.index)?.clone())
    }

    pub fn set<T: SharkyHeapExtractable>(&mut self, address: SharkyHeapAddress, value: &T) -> Option<()> {
        let mut frame = self.heap.get_mut(address.frame)?.write().unwrap();
        T::extract_mut(&mut frame)?.set(address.index, value.clone());
        Some(())
    }
}