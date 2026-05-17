#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
pub struct SharkyHeapAddress {
    frame: usize,
    index: usize,
}

pub type SharkyMax = usize;
pub type SharkyInt = i64;
pub type SharkyReal = f64;
pub type SharkyByte = u8;
pub type SharkyBool = bool;
pub type SharkyString = String;
pub type SharkyHeapReference = SharkyHeapAddress;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum SharkyDataType {
    Max(usize),
    Int(i64),
    Real(f64),
    Byte(u8),
    Bool(bool),
    String(String),
    HeapReference(SharkyHeapReference),
    Nil
}


impl std::fmt::Display for SharkyDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SharkyDataType::Max(v)            => write!(f, "{}", v),
            SharkyDataType::Int(v)            => write!(f, "{}", v),
            SharkyDataType::Real(v)           => write!(f, "{}", v),
            SharkyDataType::Byte(v)           => write!(f, "{}", v),
            SharkyDataType::Bool(v)           => write!(f, "{}", v),
            SharkyDataType::String(v)         => write!(f, "{}", v),
            SharkyDataType::HeapReference(v)  => write!(f, "<ref {}:{}>", v.frame, v.index),
            SharkyDataType::Nil               => write!(f, "nil"),
        }
    }
}

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