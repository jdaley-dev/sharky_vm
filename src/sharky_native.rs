use libloading::*;
use std::{collections::HashMap, ops::Deref, path::{Path, PathBuf}, sync::{Mutex, OnceLock}};

use crate::sharky_data_types::SharkyDataType;

type SharkyFunctionSignature = unsafe extern "C" fn(*const SharkyDataType, usize) -> SharkyDataType;

pub struct SharkyNativeLibrary {
    name: String,
    location: PathBuf,
    library: Library,
    functions: Vec<SharkyFunctionSignature>,
    symbol_map: HashMap<String, usize>    
}

impl SharkyNativeLibrary {

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn load_library(library_name: &str, library_location: &Path) -> Option<SharkyNativeLibrary> {
        // Safety Audit: Option<> prevents faulty libraries from being passed along.
        unsafe {
            let lib = Library::new(library_location).ok()?;
            Some(SharkyNativeLibrary { 
                name: String::from(library_name), 
                location: PathBuf::from(library_location), 
                library: lib, 
                functions: vec![], 
                symbol_map: HashMap::new()
            })
        }
    }

    pub fn load_symbol(&mut self, symbol_name: &str) -> Option<usize> {
        if let Some(&symbol) = self.symbol_map.get(symbol_name) { return Some(symbol); }
        // Safety audit: This thoroughly checks if the symbol does actually exist. If it doesn't we get None.
        unsafe {
            let symbol = self.library.get::<SharkyFunctionSignature>(symbol_name.as_bytes()).ok()?;
            let id = self.functions.len();
            self.functions.push(*symbol);
            self.symbol_map.insert(String::from(symbol_name), id); 
            Some(id)
        }
    }

    pub fn call(&mut self, index: usize, parameters: Vec<SharkyDataType>) -> Option<SharkyDataType> {
        // Safety audit: There's no way I'm aware of to make calling a raw C function safe. 
        // The arguments are always consistent and the function (should) always exist.
        // Short of a bluescreen the worst we SHOULD get in return is a None. Only time will tell.
        unsafe {
            Some(self.functions.get(index)?(parameters.as_ptr(), parameters.len()))
        }
    }
}