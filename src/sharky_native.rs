#![allow(dead_code)]
use libloading::*;
use std::{path::{Path}, sync::Arc};

use crate::sharky_data_types::{SharkyDataType, SharkySynced};


#[repr(C)]
struct SharkyFFIStack {
    count: usize,
    ptr: *const SharkyDataType
}

type SharkyFFIFunctionSignature = unsafe extern "C" fn(SharkyFFIStack) -> SharkyFFIStack;

#[derive(Clone, Copy)]
pub struct SharkyFFIFunctionHandle {
    ptr: *const ()
}

unsafe impl Send for SharkyFFIFunctionHandle {}
unsafe impl Sync for SharkyFFIFunctionHandle {}
pub struct SharkyFFILibrary {
    library: Arc<Library>,
}

impl SharkyFFILibrary {

    pub fn load_library(library_location: &Path) -> Option<Self> {
        // Safety Audit: Option<> prevents faulty libraries from being passed along.
        // That's all we can really guarantee here. It's up to the libloading library
        // for safety.
        unsafe {
            let lib = Library::new(library_location).ok()?;
            Some(Self { 
                library: Arc::new(lib), 
            })
        }
    }

    pub fn query_function(&self, name: &str) -> Option<SharkyFFIFunctionHandle> {
        // Safety audit: careful listening to error states. The fundamental danger here
        // is we're casting data into a raw type-erased form. BUT, we do this because it's
        // NECESSARY. In my opinion: this is tasteful and careful.
        unsafe {
            let symbol = self.library.get::<SharkyFFIFunctionSignature>(name.as_bytes()).ok()?;
            let fn_ptr = *symbol;
            let raw_ptr = std::mem::transmute::<SharkyFFIFunctionSignature, *const()>(fn_ptr);
            Some(SharkyFFIFunctionHandle { ptr:  raw_ptr })
        }
    }

    pub fn call_function(func: &SharkyFFIFunctionHandle, parameters: &Vec<SharkyDataType>) -> Option<Vec<SharkyDataType>> {
        unsafe {
            let func = std::mem::transmute::<*const(), SharkyFFIFunctionSignature>(func.ptr);
            let ffi_stack = SharkyFFIStack {ptr: parameters.as_ptr(), count: parameters.len()};
            let result = func(ffi_stack);
            if result.count > 0 {
                Some(Vec::from_raw_parts(result.ptr as *mut SharkyDataType, result.count, result.count))
            } else {
                None
            } 
        }
    }

}

#[derive(Default)]
pub struct SharkyFFIPool {
    libraries: Vec<SharkyFFILibrary>,
    functions: Vec<SharkyFFIFunctionHandle>,
}

impl SharkyFFIPool {
    pub fn new() -> Self {
        Self::default()
    }

    /// Thread Safety: Call only from the using thread.
    pub fn load_library(&mut self, path: &str) -> Option<usize> {
        let id = self.libraries.len();
        self.libraries.push(SharkyFFILibrary::load_library(Path::new(path))?);
        Some(id)
    }

    /// Thread Safety: Call only from the using thread.
    pub fn load_function(&mut self, library: usize, symbol: &str) -> Option<usize> {
        let id = self.functions.len();
        self.functions.push(self.libraries.get(library)?.query_function(symbol)?);
        Some(id)
    }

    /// Thread Safety: Call only from the using thread.
    pub fn clone_function_arc_vec(&self) -> Arc<Vec<SharkyFFIFunctionHandle>> {
        Arc::new(self.functions.clone())
    }
}