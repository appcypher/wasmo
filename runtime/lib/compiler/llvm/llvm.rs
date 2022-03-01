// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use llvm_sys::{
    prelude::{LLVMContextRef, LLVMModuleRef},
    target::{LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeTarget},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LLVM {
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) module: Vec<LLVMModuleRef>,

    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) context: Vec<LLVMContextRef>,
}

// Generates extern'd functions:
// - add_imported_function_resolver(resolver_addr) -> save address to designated data section
//
// - resolve_imported_memories(store_ref_addr) -> call intrinsics, save address to designated data section
// - resolve_imported_tables(store_ref_addr) -> call intrinsics, save address to designated data section
// - resolve_imported_globals(store_ref_addr) -> call intrinsics, save address to designated data section
//
// - initialize_local_memories -> save address to designated data section
// - initialize_local_tables -> save address to designated data section
// - initialize_local_globals -> save content to designated data section
//
// - _start
//
// Generate code for:
// - resolving individual imported function
// - loading table and memory addresses into easily accessible variables in function
// - calling indirect function
//
// Generate data sections:
// - memories -> (length, memory_addr*)
// - tables -> (length, table_addr*)
// - globals -> (length, global_addr*)
// - functions -> (length, (type, func_addr)*)
// - imported_function_resolver -> returns the resolved func_addr
impl LLVM {
    pub fn new() -> Self {
        // TODO(appcypher): Add error handling.
        unsafe {
            LLVM_InitializeNativeTarget();
            LLVM_InitializeNativeAsmPrinter();
        }

        // TODO(appcypher): Initialize LLVM context.
        Self {
            module: vec![],
            context: vec![],
        }
    }
}
