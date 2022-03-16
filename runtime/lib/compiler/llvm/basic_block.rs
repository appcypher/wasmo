use llvm_sys::prelude::LLVMBasicBlockRef;

pub(crate) struct BasicBlock {
    basic_block_ref: LLVMBasicBlockRef,
}
