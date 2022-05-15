use anyhow::Result;
use llvm::{builder::LLBuilder, values::LLValue, LLVM};
use log::debug;
use wasmparser::FunctionBody;

use crate::compiler::{
    conversions,
    generator::{Control, OperatorGenerator},
    ModuleInfo,
};

use super::Generator;

/// Generates LLVM IR for a function.
pub(crate) struct FunctionBodyGenerator<'a> {
    pub(crate) llvm: &'a mut LLVM,
    pub(crate) info: &'a ModuleInfo,
    pub(crate) body: &'a FunctionBody<'a>,
    pub(crate) body_index: usize,
}

impl<'a> Generator for FunctionBodyGenerator<'a> {
    fn generate(&mut self) -> Result<()> {
        debug!("function body: {:?}", self.body);
        debug!("function body index: {:?}", self.body_index);

        // Get LLVM function type.
        let local_function_offset = self.info.imports.functions.len();
        let function_index = self.body_index + local_function_offset;
        let type_index = self.info.functions[function_index].type_index;
        let llvm_func_type = &self.llvm.info.types[type_index as usize];

        // Create an LLVM function.
        let llvm_module = self.llvm.module.as_mut().unwrap();
        let mut llvm_func =
            llvm_module.add_function(&format!("func_{}", self.body_index), llvm_func_type)?;

        // Create entry basic block.
        let llvm_context = &self.llvm.context;
        let llvm_entry_bb = llvm_func.create_basic_block("entry", llvm_context)?;

        // Create a builder.
        let mut llvm_builder = LLBuilder::new(llvm_context);
        llvm_builder.position_at_end(&llvm_entry_bb);

        // Build locals.
        let locals_reader = self.body.get_locals_reader()?;
        let func_type = &self.info.types[type_index as usize];

        // First the params.
        let mut llvm_params = Vec::with_capacity(func_type.params.len());
        for (index, _) in func_type.params.iter().enumerate() {
            let llvm_param = llvm_func.get_param(index as u32);
            llvm_params.push(llvm_param);
        }

        // Then the locals.
        let mut llvm_locals = Vec::with_capacity(locals_reader.get_count() as usize);
        for local in locals_reader.into_iter() {
            let (index, ref ty) = local?;
            let llvm_local_ty = &conversions::wasmparser_to_llvm_numtype(llvm_context, ty);
            let llvm_local = llvm_builder.build_alloca(llvm_local_ty, &format!("local_{index}"))?;
            llvm_locals.push(llvm_local);
        }

        // The stacks.
        let mut control_stack: Vec<Control> = vec![];
        let mut value_stack: Vec<Box<dyn LLValue>> = vec![];

        // Operators.
        for operator in self.body.get_operators_reader()?.into_iter() {
            let block_count = control_stack.len();

            let mut op_gen = OperatorGenerator {
                operator: &operator?,
                llvm_context,
                llvm_params: &llvm_params,
                llvm_locals: &llvm_locals,
                llvm_builder: &mut llvm_builder,
                llvm_func: &mut llvm_func,
                control_stack: &mut control_stack,
                value_stack: &mut value_stack,
                block_count,
            };

            op_gen.generate()?;
        }

        Ok(())
    }
}

// /// TODO(appcypher): Implement.
// pub(crate) fn generate_start_function() {}
