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

    pub fn load_library(library_name: &str, library_location: &Path) -> Option<SharkyNativeLibrary> {
        // Safety Audit: Option<> prevents faulty libraries from being passed along.
        unsafe {
            match Library::new(library_location) {
                Ok(lib) => 
                Some(SharkyNativeLibrary { 
                    name: String::from(library_name), 
                    location: PathBuf::from(library_location), 
                    library: lib, 
                    functions: vec![], 
                    symbol_map: HashMap::new()
                }),
                Err(_) => {
                    None
                }
            }
        }
    }

    pub fn load_symbol(&mut self, symbol_name: &str) -> Option<usize> {
        if !self.location.exists() {
            return None;
        }


        if self.symbol_map.contains_key(symbol_name) {
            return None; 
        }

        unsafe {
            return if let Ok(symbol) = self.library.get::<SharkyFunctionSignature>(symbol_name.as_bytes()) {
                let id = self.functions.len();
                self.functions.push(*symbol);
                self.symbol_map.insert(String::from(symbol_name), id); 
                Some(id)
            } else {
                None
            }
        }
    }

    pub fn call(&mut self, index: usize, parameters: Vec<SharkyDataType>) -> Option<SharkyDataType> {
        if index < self.functions.len() {
            unsafe {
                Some(self.functions[index](parameters.as_ptr(), parameters.len()))
            }
        } else {
            None
        }
    }
}