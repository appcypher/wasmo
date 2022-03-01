// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use serde::{Serialize, Deserialize};
use llvm_sys::{
    target::{LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeTarget},
    LLVMModule,
};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LLVM {
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) module: Option<LLVMModule>,
}

// Generates extern callable functions:
// - add_imported_function_resolver -> save address to designated data section
// - resolve_imported_memories -> save address to designated data section
// - resolve_imported_tables -> save address to designated data section
// - resolve_imported_globals -> save address to designated data section
// - initialize_local_memories -> save address to designated data section
// - initialize_local_tables -> save address to designated data section
// - initialize_local_globals -> save content to designated data section
// - _start
// - set_store_base_addresses
//
// Generate code for:
// - resolving individual imported function
// - loading table and memory addresses into easily accessible variables in function
// - calling indirect function
//
// Generate data sections:
// - memories -> (length, (memory_addr, min, max, size)*)
// - tables -> (length, (table_addr, min, max, size)*)
// - globals -> (length, global_addr*)
// - functions -> (length, (address, type)*)
// - imported_function_resolver -> (function_addr)
impl LLVM {
    pub fn new() -> Self {
        // Initialize native target and asm printer.
        unsafe {
            LLVM_InitializeNativeTarget();
            LLVM_InitializeNativeAsmPrinter();
        }
        unimplemented!()
    }
}
