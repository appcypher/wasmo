use llvm_sys::target_machine::{
    LLVMGetTargetMachineCPU, LLVMGetTargetMachineFeatureString, LLVMGetTargetMachineTriple,
    LLVMTargetMachineRef,
};

use crate::string::LLString;

pub struct LLTargetMachine(LLVMTargetMachineRef);

impl LLTargetMachine {
    /// Returns the target triple of the target machine.
    pub fn target_triple(&self) -> String {
        LLString::from_ptr(unsafe { LLVMGetTargetMachineTriple(self.0) }).to_string()
    }

    /// Returns the CPU of the target machine.
    pub fn target_cpu(&self) -> String {
        LLString::from_ptr(unsafe { LLVMGetTargetMachineCPU(self.0) }).to_string()
    }

    /// Returns the feature string of the target machine.
    pub fn target_features(&self) -> String {
        LLString::from_ptr(unsafe { LLVMGetTargetMachineFeatureString(self.0) }).to_string()
    }
}
