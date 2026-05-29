#![allow(dead_code)]
use std::{fmt::Display, sync::{Arc, RwLock}};

use crate::{sharky_data_types::*, sharky_vm::{SharkyInterpreter, SharkyMemoryLayout}};

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
    
    pub fn search(&self, condition: impl Fn(&T) -> bool) -> bool {
        self.stack.iter().any(condition)
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

pub type SharkyDataStack = SharkyStack<SharkyDataType>;
pub type SharkyHeapStack = Vec<Option<RwLock<SharkyDataStack>>>;
pub type SharkySyncedHeapStack = Arc<RwLock<SharkyHeapStack>>;
