use std::{ffi::CStr, fmt::Display};

use llvm_sys::core::LLVMDisposeMessage;

use crate::not_null;

pub(crate) struct LLString(*mut i8);

impl LLString {
    pub(crate) fn from_ptr(ptr: *mut i8) -> Self {
        Self(not_null!(ptr))
    }
}

impl Display for LLString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c_str = unsafe { CStr::from_ptr(self.0) };
        let s = c_str.to_str().unwrap();
        write!(f, "{}", s)
    }
}

impl Drop for LLString {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeMessage(self.0);
        }
    }
}
