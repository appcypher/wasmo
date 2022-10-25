use llvm_sys::orc2::{LLVMOrcJITTargetMachineBuilderDetectHost, LLVMOrcJITTargetMachineBuilderRef};

use crate::target_machine::LLTargetMachine;

pub struct LLJitTargetMachineBuilder(LLVMOrcJITTargetMachineBuilderRef);

impl LLJitTargetMachineBuilder {
    pub fn detect_host(&mut self) -> LLTargetMachine {
        let _opaque_error = unsafe { LLVMOrcJITTargetMachineBuilderDetectHost(&mut self.0 as *mut _) };

        // LLTargetMachine::from_ptr();
        todo!()
    }
}
