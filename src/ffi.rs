#![allow(dead_code)]
use libloading::*;
use std::{path::{Path}, sync::Arc};

use crate::data_types::{SharkyDataType};


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

type SharkyFFIAllocFunctionSignature = unsafe extern "C" fn (usize, usize) -> *mut();
type SharkyFFIFreeFunctionSignature = unsafe extern "C" fn (ptr: *mut(), count: usize, align: usize) -> ();
type SharkyFFIInitFunctionSignature = unsafe extern "C" fn (SharkyFFIContext) -> ();

#[repr(C)]
struct SharkyFFIContext {
    alloc_function: *const SharkyFFIAllocFunctionSignature,
    free_function: *const SharkyFFIFreeFunctionSignature,
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
            let init_func = lib.get::<SharkyFFIInitFunctionSignature>(b"init_sharky_lib").expect(format!("Could not find init_sharky_lib function in sharky library: {}", library_location.to_string_lossy()).as_str());
            init_func(SharkyFFIContext { 
                alloc_function: sharky_alloc as *mut SharkyFFIAllocFunctionSignature, 
                free_function: sharky_free  as *mut SharkyFFIFreeFunctionSignature,
            });
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sharky_alloc(count: usize, align: usize) -> *mut() {
    let layout = std::alloc::Layout::from_size_align(count, align).unwrap();
    (unsafe { std::alloc::alloc(layout) }) as *mut()
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn sharky_free(ptr: *mut(), count: usize, align: usize) {
    let layout = std::alloc::Layout::from_size_align(count, align).unwrap();
    unsafe { std::alloc::dealloc(ptr as *mut u8, layout); }
}