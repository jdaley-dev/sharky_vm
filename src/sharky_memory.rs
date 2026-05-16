#[derive(Debug, Clone)]
pub struct HeapAddress {
    frame: usize,
    index: usize,
}

#[derive(Debug, Clone)]
pub enum SharkyDataType {
    Max(usize),
    Int(i64),
    Real(f64),
    Byte(u8),
    Bool(bool),
    String(String),
    HeapReference(HeapAddress),
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

pub struct SharkyFrame {
    stack: Vec<SharkyDataType>,
}

impl SharkyFrame {
    pub fn default() -> SharkyFrame {
        SharkyFrame { 
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

    pub fn get(&mut self, index: usize) -> SharkyDataType {
        return if let Some(val) = self.stack.get_mut(index) {
            val.clone()
        } else {
            SharkyDataType::Nil
        }
    }
}