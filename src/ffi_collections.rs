use std::{mem::ManuallyDrop, ops::{Deref, DerefMut}};

// This bridges the gap between C and Rust, providing a C-ABI Compatible Vector
/// A simplistic C-Layout guaranteed Vec. It can be operated upon just like a vec.
/// It's purely for FFI compatibility and simplicity. 
#[derive(Debug, PartialEq, PartialOrd)]
#[repr(C)]
pub struct CVec<T> {
    ptr: *mut T,
    len: usize,
}

unsafe impl<T: Send> Send for CVec<T> {}
unsafe impl<T: Sync> Sync for CVec<T> {}

impl<T> CVec<T> {
    pub fn new() -> Self {
        Self {
            ptr: std::ptr::NonNull::<T>::dangling().as_ptr(),
            len: 0
        }
    }

    pub fn operate<F: FnOnce(&mut Vec<T>)>(&mut self, f: F) {        
        // Safety audit: as with all FFI calls we really just need unsafe
        // code sometimes. Luckily we have lots of rust standard lib 
        // tools offering some guardrails. By simply building a vec
        // and forgetting it we guarantee the memory is never raw-ly provided
        // to the user.
        unsafe {
            let mut upon: Vec<T> = if self.len > 0 {
                Vec::from_raw_parts(self.ptr, self.len, self.len)
            } else {
                Vec::new()
            };

            f(&mut upon);
            upon.shrink_to_fit();
            self.ptr = upon.as_mut_ptr();
            self.len = upon.len();
            std::mem::forget(upon); 
        }
    }

    pub fn get_operator_mut(&mut self) -> SharkyCVecOperatorMut<'_, T> {
        SharkyCVecOperatorMut::new(self)
    }

    pub fn get_operator(&self) -> SharkyCVecOperator<'_, T> {
        SharkyCVecOperator::new(self)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_ptr_mut(&self) -> *mut T {
        self.ptr
    }

}

impl<T: Clone> Clone for CVec<T> {
    fn clone(&self) -> Self {
        if self.len == 0 {
            return Self::new()
        }
        let vec = self.get_operator();
        let mut cloned = vec.as_slice().to_vec();
        cloned.shrink_to_fit();
        let ptr = cloned.as_mut_ptr();
        std::mem::forget(cloned);
        Self {
            ptr: ptr,
            len: self.len
        }
    }
}

impl<T> From<Vec<T>> for CVec<T> {
    // Takes ownership of a Vec<T> and returns it as a SharkyCVec
    fn from(mut vec: Vec<T>) -> Self {
        vec.shrink_to_fit();
        let ptr = vec.as_mut_ptr();
        let len = vec.len();
        std::mem::forget(vec);
        Self { ptr, len }
    }
}

impl<T> Drop for CVec<T> {
    // gives ownership of the pointer and len to a vec for the drop. (Note no values need updated here since this is the DROP function, struct's ownership goes to the abyss anyway.)
    fn drop(&mut self) {
        if self.len > 0 {
            unsafe {
                let _ = Vec::from_raw_parts(self.ptr, self.len, self.len);
            }
        }
    }
}

// A simple (mutable) operator borrow structure that enables direct-(mutable)dereferencing to a Vector.
// Remember ultimately this should just operate as a vector- just with a very basic and
// guaranteed C-ABI layout.
pub struct SharkyCVecOperator<'a, T> {
    #[allow(unused)] // We need to tie the object's lifetime to the CVec itself.
    operate: &'a CVec<T>,
    vec: ManuallyDrop<Vec<T>>, // we manually drop to prevent it from actually deallocating the memory, as that's not this object's job.
}

impl<'a, T> SharkyCVecOperator<'a, T> {
    fn new(cvec: &'a CVec<T>) -> Self {
        unsafe {
            let vector = ManuallyDrop::new(Vec::from_raw_parts(cvec.as_ptr_mut(), cvec.len(), cvec.len()));
            Self  {
                operate: cvec,
                vec: vector,
            }
        }
    }
}

impl<T> Deref for SharkyCVecOperator<'_, T> {
    fn deref(&self) -> &Self::Target {
        &self.vec
    }

    type Target = Vec<T>;
}

impl<T> Drop for SharkyCVecOperator<'_, T> {
    fn drop(&mut self) {
        unsafe {
            let owned = ManuallyDrop::take(&mut self.vec);
            std::mem::forget(owned);
        }
    }
}

/// A simple operator borrow structure that enables direct-dereferencing to a Vector.
/// Remember ultimately this should just operate as a vector- just with a very basic and
/// guaranteed C-ABI layout.
pub struct SharkyCVecOperatorMut<'a, T> {
    operate: &'a mut CVec<T>,
    vec: ManuallyDrop<Vec<T>>, // we manually drop to prevent it from actually deallocating the memory, as that's not this object's job.
}

impl<'a, T> SharkyCVecOperatorMut<'a, T> {
    fn new(cvec: &'a mut CVec<T>) -> Self {
        if cvec.len > 0 {
            unsafe {
                let vector = ManuallyDrop::new(Vec::from_raw_parts(cvec.as_ptr_mut(), cvec.len(), cvec.len()));
                Self  {
                    operate: cvec,
                    vec: vector,
                }
            }
        } else {
            Self  {
                operate: cvec,
                vec: ManuallyDrop::new(Vec::new())
            }
        }
    }
}

impl<T> Deref for SharkyCVecOperatorMut<'_, T> {
    fn deref(&self) -> &Self::Target {
        &self.vec
    }

    type Target = Vec<T>;
}

impl<T> DerefMut for SharkyCVecOperatorMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

impl<T> Drop for SharkyCVecOperatorMut<'_, T> {
    fn drop(&mut self) {
        unsafe {
            let mut owned = ManuallyDrop::take(&mut self.vec);
            owned.shrink_to_fit();
            self.operate.ptr = owned.as_mut_ptr();
            self.operate.len = owned.len();
            std::mem::forget(owned);
        }
    }
}