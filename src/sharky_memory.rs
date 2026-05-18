use crate::sharky_data_types::*;

pub struct SharkyStack {
    stack: Vec<SharkyDataType>,
}

impl SharkyStack {
    pub fn default() -> SharkyStack {
        SharkyStack { 
            stack: Vec::new()
        }
    }
    
    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn push(&mut self, data: SharkyDataType) {
        self.stack.push(data);
    }

    pub fn set(&mut self, index: usize, data: SharkyDataType) {
        if let Some(val) = self.stack.get_mut(index) {
            *val = data;
        }
    }

    pub fn read(&self, index: usize) -> SharkyDataType {
        self.stack.get(index).unwrap_or(&SharkyDataType::Nil).clone()
    }

    pub fn read_top(&self) -> SharkyDataType {
        self.read(self.size() - 1).clone()
    }

    pub fn debug_print_stack(&self) {
        let mut point = 0;
        println!("--- SHARKY STACK DEBUG PRINT ---");
        for i in self.stack.iter() {
            println!("{point} - [{i}]\n--------------------------------");
            point += 1;
        }
    }

    pub fn size(&self) -> usize {
        self.stack.len()
    }

    pub fn clear(&mut self) {
        self.stack.clear();
    }
}

pub type SharkyStackVec = Vec<SharkyStack>;