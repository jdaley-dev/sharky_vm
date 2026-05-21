use crate::sharky_data_types::SharkyByte;

#[derive(Default)]
pub struct SharkyStringPool {
    buffer: Vec<SharkyByte>,
    spans: Vec<(usize, usize)>
}

impl SharkyStringPool {

    pub fn new() -> SharkyStringPool {Self::default()}

    pub fn create_string(&mut self, stack: &[u8]) -> usize {
        let begin = self.buffer.len(); // given len is the current size of the vec, this also represents the first index of our extension.
        let end = begin + stack.len();

        self.buffer.extend(stack); 
        self.spans.push((begin, end)); 
        
        self.spans.len() - 1 // our new index  
    }

    pub fn get_slice(&self, id: usize) -> Option<&[u8]>{
        let span = self.spans.get(id)?;
        self.buffer.get(span.0..span.1)
    }
}