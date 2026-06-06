pub mod data_types;
pub mod collections;
pub mod ffi;
pub mod ffi_collections;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let x: ffi_collections::CVec<usize> = ffi_collections::CVec::new();
    }
}
