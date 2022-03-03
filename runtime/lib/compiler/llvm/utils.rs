// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use std::{ffi::CString, os::raw::c_char, pin::Pin};
use utilities::result::Result;

#[derive(Debug)]
pub(crate) struct LLString {
    // Keep string around.
    string: CString,
    ptr: *const c_char,
}

impl LLString {
    pub(crate) fn as_ptr(&self) -> *const c_char {
        self.ptr
    }

    pub(crate) fn try_from(s: &str) -> Result<Pin<Box<Self>>> {
        let mut this = LLString {
            string: CString::new(s)?,
            ptr: std::ptr::null(),
        };

        this.ptr = this.string.as_ptr();

        Ok(Box::pin(this))
    }
}
