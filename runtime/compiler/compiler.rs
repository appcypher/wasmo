use std::pin::Pin;

use anyhow::Result;
use hashbrown::HashMap;
use llvm::{
    basic_block::LLBasicBlock,
    builder::LLBuilder,
    types::{LLNumType, LLNumTypeKind},
    values::{LLFunction, LLValue},
    LLVM,
};
use log::debug;
use serde::{Deserialize, Serialize};
use wasmparser::{
    DataSectionReader, ElementSectionReader, ExportSectionReader, FunctionBody,
    FunctionSectionReader, GlobalSectionReader, ImportSectionEntryType, ImportSectionReader,
    MemorySectionReader, Operator, Parser, Payload, TableSectionReader, TypeDef, TypeSectionReader,
};

use super::{
    exports::{Export, Exports},
    imports::{Import, Imports},
    utils::convert,
    value::Value,
    Data, Element, Function, Global, Memory, Table,
};
use crate::{
    compiler::exports::ExportKind,
    errors::CompilerError,
    types::{FuncType, Limits},
};

//--------------------------------------------------------------------------------------------------
// Type Definitions
//--------------------------------------------------------------------------------------------------

/// The compiler is responsible for compiling a module.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Compiler {
    /// The LLVM context.
    #[serde(skip)]
    pub(crate) llvm: Option<Pin<Box<LLVM>>>,
    /// Option for enabling lift-off compilation.
    pub liftoff: bool,
    /// Compiler data.
    pub info: ModuleInfo,
}

/// This type holds general WebAssembly module information gathered during compilation.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ModuleInfo {
    /// List of imported components of a module.
    pub imports: Imports,
    /// List of exported members of a module.
    pub exports: Exports,
    /// An ordered list of types from the type section.
    pub types: Vec<FuncType>,
    /// An ordered list of functions from the function section.
    pub functions: Vec<Function>,
    /// An ordered list of tables from the table section.
    pub tables: Vec<Table>,
    /// An ordered list of memories from the memory section.
    pub memories: Vec<Memory>,
    /// An ordered list of globals from the global section.
    pub globals: Vec<Global>,
    /// An ordered list of elements from the element section.
    pub elements: Vec<Element>,
    /// An ordered list of data from the data section.
    pub data: Vec<Data>,
    /// The start function.
    pub start_function: Option<u32>,
}

/// TODO(appcypher): Document this.
pub enum Block {
    If {
        then: LLBasicBlock,
        r#else: Option<LLBasicBlock>,
        cont: Option<LLBasicBlock>,
    },
    Loop {
        main: LLBasicBlock,
        cont: Option<LLBasicBlock>,
    },
    Block {
        main: LLBasicBlock,
        cont: Option<LLBasicBlock>,
    },
}

//--------------------------------------------------------------------------------------------------
// Implementations
//--------------------------------------------------------------------------------------------------

impl Compiler {
    /// Creates a new `Compiler` with the given options.
    pub fn new(liftoff: bool) -> Self {
        Self {
            liftoff,
            ..Default::default()
        }
    }

    /// Compiles provided wasm bytes.
    pub fn compile(&mut self, wasm: &[u8]) -> Result<()> {
        // The LLVM module.
        let mut llvm = LLVM::new()?;

        // Body index.
        let mut body_index = 0;

        for payload in Parser::new(0).parse_all(wasm) {
            match payload? {
                Payload::Version { .. } => (),
                Payload::TypeSection(reader) => {
                    debug!("======= TypeSection =======");
                    self.compile_types(reader, &mut llvm)?;
                }
                Payload::ImportSection(reader) => {
                    debug!("======= ImportSection =======");
                    self.compile_imports(reader)?;
                }
                Payload::FunctionSection(reader) => {
                    debug!("======= FunctionSection =======");
                    self.compile_functions(reader)?;
                }
                Payload::TableSection(reader) => {
                    debug!("======= TableSection =======");
                    self.compile_tables(reader)?;
                }
                Payload::MemorySection(reader) => {
                    debug!("======= MemorySection =======");
                    self.compile_memories(reader)?;
                }
                Payload::GlobalSection(reader) => {
                    debug!("======= GlobalSection =======");
                    self.compile_globals(reader)?;
                }
                Payload::ExportSection(reader) => {
                    debug!("======= ExportSection =======");
                    self.compile_exports(reader)?;
                }
                Payload::StartSection { func, .. } => {
                    debug!("======= StartSection =======");
                    self.compile_start_function(func)?;
                }
                Payload::ElementSection(reader) => {
                    debug!("======= ElementSection =======");
                    self.compile_elements(reader)?;
                }
                Payload::DataCountSection { .. } => {
                    debug!("======= DataCountSection =======");
                }
                Payload::DataSection(reader) => {
                    debug!("======= DataSection =======");
                    self.compile_data(reader)?;
                }
                Payload::CustomSection { name, .. } => {
                    debug!("======= CustomSection =======");
                    debug!("custom section name: {:?}", name);
                    // TODO(appcypher): Generate index space mappings to names to be used in codegen
                    // self.compile_name_section()?;
                }
                Payload::CodeSectionStart { .. } => {
                    debug!("======= CodeSectionStart =======");
                }
                Payload::CodeSectionEntry(body) => {
                    debug!("======= CodeSectionEntry =======");
                    self.compile_function_body(body, &mut llvm, &mut body_index)?;
                }
                Payload::ModuleSectionStart { .. } => {
                    debug!("======= ModuleSectionStart =======");
                }
                Payload::ModuleSectionEntry { .. } => {
                    debug!("======= ModuleSectionEntry =======");
                }
                Payload::UnknownSection { .. } => {
                    debug!("======= UnknownSection =======");
                }
                Payload::End => {
                    debug!("======= End =======");
                    // TODO(appcypher): Generate misc LLVM code here.
                    // self.compile_context()?;
                    // - allocate fixup sections
                    // - generate resolver functions
                    // - generate setup functions
                    // - generate initializer functions
                    // - generate main function
                    // - start function export
                }
                t => {
                    return Err(CompilerError::UnsupportedSection(format!("{:?}", t)).into());
                }
            }
        }

        // Print module.
        llvm.module.as_ref().unwrap().print();

        self.llvm = Some(llvm);

        Ok(())
    }
}

impl Compiler {
    /// Compiles function types in type section.
    pub(crate) fn compile_types(
        &mut self,
        reader: TypeSectionReader,
        llvm: &mut LLVM,
    ) -> Result<()> {
        for result in reader.into_iter() {
            let typedef = result?;

            debug!("type: {:?}", typedef);

            match typedef {
                TypeDef::Func(ty) => {
                    let wasmo_func_ty = convert::to_wasmo_functype(&ty)?;
                    let llvm_func_ty = convert::to_llvm_functype(&llvm.context, &wasmo_func_ty);
                    llvm.info.types.push(llvm_func_ty);
                    self.info.types.push(wasmo_func_ty);
                }
                t => {
                    return Err(
                        CompilerError::UnsupportedTypeSectionEntry(format!("{:?}", t)).into(),
                    )
                }
            };
        }

        Ok(())
    }

    /// Compiles imports in import section.
    pub fn compile_imports(&mut self, reader: ImportSectionReader) -> Result<()> {
        for result in reader.into_iter() {
            let import = result?;

            debug!("import: {:?}", import);

            // TODO(appcypher): Generate LLVM code for imports.
            // - generate `resolve_imported_*` functions.
            match import.ty {
                ImportSectionEntryType::Function(index) => {
                    self.info.imports.functions.push(Import::new(
                        import.module.to_string(),
                        import.field.map(|s| s.to_string()),
                        self.info.functions.len() as u32,
                    ));

                    self.info.functions.push(Function::new(index));
                }
                ImportSectionEntryType::Table(ty) => {
                    self.info.imports.tables.push(Import::new(
                        import.module.to_string(),
                        import.field.map(|s| s.to_string()),
                        self.info.tables.len() as u32,
                    ));

                    self.info.tables.push(Table::new(
                        Limits::new(ty.initial as u64, ty.maximum.map(|x| x as u64)),
                        convert::to_wasmo_valtype(&ty.element_type)?,
                    ));
                }
                ImportSectionEntryType::Memory(ty) => {
                    // TODO(appcypher): Wasmo does not support memory64 proposal yet.
                    if ty.memory64 {
                        return Err(CompilerError::UnsupportedMemory64Proposal.into());
                    }

                    self.info.imports.memories.push(Import::new(
                        import.module.to_string(),
                        import.field.map(|s| s.to_string()),
                        self.info.memories.len() as u32,
                    ));

                    self.info
                        .memories
                        .push(Memory::new(Limits::new(ty.initial, ty.maximum), ty.shared));
                }
                ImportSectionEntryType::Global(ty) => {
                    self.info.imports.globals.push(Import::new(
                        import.module.to_string(),
                        import.field.map(|s| s.to_string()),
                        self.info.globals.len() as u32,
                    ));

                    self.info.globals.push(Global::new(
                        convert::to_wasmo_valtype(&ty.content_type)?,
                        ty.mutable,
                    ));
                }
                t => {
                    return Err(
                        CompilerError::UnsupportedImportSectionEntry(format!("{:?}", t)).into(),
                    )
                }
            }
        }

        Ok(())
    }

    /// Compiles functions in function section.
    pub fn compile_functions(&mut self, reader: FunctionSectionReader) -> Result<()> {
        for result in reader.into_iter() {
            let type_index = result?;

            debug!("function type_index: {:?}", type_index);

            self.info.functions.push(Function::new(type_index));
        }

        Ok(())
    }

    /// Compiles tables in table section.
    pub fn compile_tables(&mut self, reader: TableSectionReader) -> Result<()> {
        for result in reader.into_iter() {
            let ty = result?;

            debug!("table type: {:?}", ty);

            self.info.tables.push(Table::new(
                Limits::new(ty.initial as u64, ty.maximum.map(|x| x as u64)),
                convert::to_wasmo_valtype(&ty.element_type)?,
            ));
        }

        Ok(())
    }

    /// Compiles memories in memory section.
    pub fn compile_memories(&mut self, reader: MemorySectionReader) -> Result<()> {
        for result in reader.into_iter() {
            let ty = result?;

            debug!("memory type: {:?}", ty);

            self.info
                .memories
                .push(Memory::new(Limits::new(ty.initial, ty.maximum), ty.shared));
        }

        Ok(())
    }

    /// Compiles globals in global section.
    pub fn compile_globals(&mut self, reader: GlobalSectionReader) -> Result<()> {
        for result in reader.into_iter() {
            let global = result?;

            debug!("global: {:?}", global);

            self.info.globals.push(Global::new(
                convert::to_wasmo_valtype(&global.ty.content_type)?,
                global.ty.mutable,
            ));

            // llvm.codegen_global(reader)?;
        }

        Ok(())
    }

    /// Compiles data in data section.
    pub fn compile_data(&mut self, reader: DataSectionReader) -> Result<()> {
        for result in reader.into_iter() {
            let data = result?;

            debug!("data: {:?}", data);

            self.info
                .data
                .push(Data::new(convert::to_wasmo_data_kind(&data.kind)));

            // llvm.codegen_data(reader)?;
        }

        Ok(())
    }

    /// Compiles elements in element section.
    pub fn compile_elements(&mut self, reader: ElementSectionReader) -> Result<()> {
        for result in reader.into_iter() {
            let elem = result?;

            debug!("elem items: {:?}", elem.items);

            self.info
                .elements
                .push(Element::new(convert::to_wasmo_element_kind(&elem.kind)));

            // llvm.codegen_element(reader)?;
        }

        Ok(())
    }

    /// Compiles exports in export section.
    pub fn compile_exports(&mut self, reader: ExportSectionReader) -> Result<()> {
        for result in reader.into_iter() {
            let export = result?;

            debug!("export: {:?}", export);

            match export.kind {
                wasmparser::ExternalKind::Function => {
                    self.info.exports.inner.insert(
                        export.field.to_string(),
                        Export::new(ExportKind::Function, export.index),
                    );
                }
                wasmparser::ExternalKind::Table => {
                    self.info.exports.inner.insert(
                        export.field.to_string(),
                        Export::new(ExportKind::Table, export.index),
                    );
                }
                wasmparser::ExternalKind::Memory => {
                    self.info.exports.inner.insert(
                        export.field.to_string(),
                        Export::new(ExportKind::Memory, export.index),
                    );
                }
                wasmparser::ExternalKind::Global => {
                    self.info.exports.inner.insert(
                        export.field.to_string(),
                        Export::new(ExportKind::Global, export.index),
                    );
                }
                t => {
                    return Err(
                        CompilerError::UnsupportedExportSectionEntry(format!("{:?}", t)).into(),
                    )
                }
            }
        }

        Ok(())
    }

    /// Compiles start function.
    pub fn compile_start_function(&mut self, _func: u32) -> Result<()> {
        self.info.start_function = Some(_func);
        // llvm.codegen_start_function(reader)?;
        Ok(())
    }

    /// Compiles function body.
    pub fn compile_function_body(
        &mut self,
        body: FunctionBody,
        llvm: &mut LLVM,
        body_index: &mut usize,
    ) -> Result<()> {
        debug!("function body: {:?}", body);

        // Get LLVM function type.
        let local_function_offset = self.info.imports.functions.len();
        let function_index = *body_index + local_function_offset;
        let type_index = self.info.functions[function_index].type_index;
        let llvm_func_type = &llvm.info.types[type_index as usize];

        // Create an LLVM function.
        let llvm_module = llvm.module.as_mut().unwrap();
        let llvm_func = llvm_module.add_function(&format!("func_{body_index}"), llvm_func_type)?;

        // Create entry basic block.
        let llvm_context = &llvm.context;
        let llvm_entry_bb = llvm_func.create_basic_block("entry", llvm_context)?;

        // Create a builder.
        let mut llvm_builder = LLBuilder::new(llvm_context);
        llvm_builder.position_at_end(&llvm_entry_bb);

        // Build locals.
        let mut llvm_locals = HashMap::new();
        for r in body.get_locals_reader().into_iter() {
            for local in r.into_iter() {
                let (index, ty) = local?;

                // Get the local type. // TODO(appcypher): Skip conversion to wasmo valtype.
                let wasmo_local_ty = &convert::to_wasmo_valtype(&ty)?;
                let llvm_local_ty = &convert::to_llvm_valtype(llvm_context, wasmo_local_ty);

                // Create local variable.
                let llvm_local =
                    llvm_builder.build_alloca(llvm_local_ty, &format!("local_{index}"))?;

                llvm_locals.insert(index, llvm_local);
            }
        }

        // Body local offset.
        let _body_local_offset = self.info.types[type_index as usize].params.len();

        // The stacks.
        let mut block_stack: Vec<Block> = vec![];
        let mut implicit_stack: Vec<Box<dyn LLValue>> = vec![];

        // Operators.
        for r in body.get_operators_reader().into_iter() {
            for operator in r.into_iter() {
                let block_count = block_stack.len();
                match operator? {
                    Operator::Unreachable => {
                        llvm_builder.build_unreachable();
                    }
                    Operator::Nop => {
                        llvm_builder.build_unreachable();
                    }
                    Operator::Block { ty } => {
                        let llvm_bb = llvm_func.create_basic_block(& format!("block_{}", block_count), llvm_context)?;

                        block_stack.push(Block::Block {
                            main: llvm_bb,
                            cont: None,
                        });
                    }
                    Operator::Loop { ty } => {
                        let llvm_bb = llvm_func.create_basic_block(& format!("loop_{}", block_count), llvm_context)?;

                        block_stack.push(Block::Loop {
                            main: llvm_bb,
                            cont: None,
                        });
                    }
                    Operator::If { ty } => {
                        let llvm_then_bb = llvm_func.create_basic_block(
                            &format!("if_then_{}", block_count),
                            llvm_context,
                        )?;

                        let llvm_else_bb = llvm_func.create_basic_block(
                            &format!("if_else_{}", block_count),
                            llvm_context,
                        )?;

                        // Add conditional branching instruction.
                        let stack_value = implicit_stack.pop().unwrap();
                        llvm_builder.build_cond_br(stack_value.as_ref(), &llvm_then_bb, &llvm_else_bb);

                        block_stack.push(Block::If {
                            then: llvm_then_bb,
                            r#else: Some(llvm_else_bb),
                            cont: None,
                        });
                    }
                    Operator::Else => {
                        let llvm_cont_bb = llvm_func.create_basic_block(
                            &format!("if_cont_{}", block_count),
                            llvm_context,
                        )?;

                        if let Block::If { ref mut cont, .. } = block_stack.last_mut().unwrap() {
                            *cont = Some(llvm_cont_bb)
                        };
                    }
                    // Operator::Try { ty } => todo!(),
                    // Operator::Catch { index } => todo!(),
                    // Operator::Throw { index } => todo!(),
                    // Operator::Rethrow { relative_depth } => todo!(),
                    // Operator::End => todo!(),
                    // Operator::Br { relative_depth } => todo!(),
                    // Operator::BrIf { relative_depth } => todo!(),
                    // Operator::BrTable { table } => todo!(),
                    // Operator::Return => todo!(),
                    // Operator::Call { function_index } => todo!(),
                    // Operator::CallIndirect { index, table_index } => todo!(),
                    // Operator::ReturnCall { function_index } => todo!(),
                    // Operator::ReturnCallIndirect { index, table_index } => todo!(),
                    // Operator::Delegate { relative_depth } => todo!(),
                    // Operator::CatchAll => todo!(),
                    // Operator::Drop => todo!(),
                    // Operator::Select => todo!(),
                    // Operator::TypedSelect { ty } => todo!(),
                    // Operator::LocalGet { local_index } => todo!(),
                    // Operator::LocalSet { local_index } => todo!(),
                    // Operator::LocalTee { local_index } => todo!(),
                    // Operator::GlobalGet { global_index } => todo!(),
                    // Operator::GlobalSet { global_index } => todo!(),
                    // Operator::I32Load { memarg } => todo!(),
                    // Operator::I64Load { memarg } => todo!(),
                    // Operator::F32Load { memarg } => todo!(),
                    // Operator::F64Load { memarg } => todo!(),
                    // Operator::I32Load8S { memarg } => todo!(),
                    // Operator::I32Load8U { memarg } => todo!(),
                    // Operator::I32Load16S { memarg } => todo!(),
                    // Operator::I32Load16U { memarg } => todo!(),
                    // Operator::I64Load8S { memarg } => todo!(),
                    // Operator::I64Load8U { memarg } => todo!(),
                    // Operator::I64Load16S { memarg } => todo!(),
                    // Operator::I64Load16U { memarg } => todo!(),
                    // Operator::I64Load32S { memarg } => todo!(),
                    // Operator::I64Load32U { memarg } => todo!(),
                    // Operator::I32Store { memarg } => todo!(),
                    // Operator::I64Store { memarg } => todo!(),
                    // Operator::F32Store { memarg } => todo!(),
                    // Operator::F64Store { memarg } => todo!(),
                    // Operator::I32Store8 { memarg } => todo!(),
                    // Operator::I32Store16 { memarg } => todo!(),
                    // Operator::I64Store8 { memarg } => todo!(),
                    // Operator::I64Store16 { memarg } => todo!(),
                    // Operator::I64Store32 { memarg } => todo!(),
                    // Operator::MemorySize { mem, mem_byte } => todo!(),
                    // Operator::MemoryGrow { mem, mem_byte } => todo!(),
                    // Operator::I32Const { value } => todo!(),
                    // Operator::I64Const { value } => todo!(),
                    // Operator::F32Const { value } => todo!(),
                    // Operator::F64Const { value } => todo!(),
                    // Operator::RefNull { ty } => todo!(),
                    // Operator::RefIsNull => todo!(),
                    // Operator::RefFunc { function_index } => todo!(),
                    // Operator::I32Eqz => todo!(),
                    // Operator::I32Eq => todo!(),
                    // Operator::I32Ne => todo!(),
                    // Operator::I32LtS => todo!(),
                    // Operator::I32LtU => todo!(),
                    // Operator::I32GtS => todo!(),
                    // Operator::I32GtU => todo!(),
                    // Operator::I32LeS => todo!(),
                    // Operator::I32LeU => todo!(),
                    // Operator::I32GeS => todo!(),
                    // Operator::I32GeU => todo!(),
                    // Operator::I64Eqz => todo!(),
                    // Operator::I64Eq => todo!(),
                    // Operator::I64Ne => todo!(),
                    // Operator::I64LtS => todo!(),
                    // Operator::I64LtU => todo!(),
                    // Operator::I64GtS => todo!(),
                    // Operator::I64GtU => todo!(),
                    // Operator::I64LeS => todo!(),
                    // Operator::I64LeU => todo!(),
                    // Operator::I64GeS => todo!(),
                    // Operator::I64GeU => todo!(),
                    // Operator::F32Eq => todo!(),
                    // Operator::F32Ne => todo!(),
                    // Operator::F32Lt => todo!(),
                    // Operator::F32Gt => todo!(),
                    // Operator::F32Le => todo!(),
                    // Operator::F32Ge => todo!(),
                    // Operator::F64Eq => todo!(),
                    // Operator::F64Ne => todo!(),
                    // Operator::F64Lt => todo!(),
                    // Operator::F64Gt => todo!(),
                    // Operator::F64Le => todo!(),
                    // Operator::F64Ge => todo!(),
                    // Operator::I32Clz => todo!(),
                    // Operator::I32Ctz => todo!(),
                    // Operator::I32Popcnt => todo!(),
                    // Operator::I32Add => todo!(),
                    // Operator::I32Sub => todo!(),
                    // Operator::I32Mul => todo!(),
                    // Operator::I32DivS => todo!(),
                    // Operator::I32DivU => todo!(),
                    // Operator::I32RemS => todo!(),
                    // Operator::I32RemU => todo!(),
                    // Operator::I32And => todo!(),
                    // Operator::I32Or => todo!(),
                    // Operator::I32Xor => todo!(),
                    // Operator::I32Shl => todo!(),
                    // Operator::I32ShrS => todo!(),
                    // Operator::I32ShrU => todo!(),
                    // Operator::I32Rotl => todo!(),
                    // Operator::I32Rotr => todo!(),
                    // Operator::I64Clz => todo!(),
                    // Operator::I64Ctz => todo!(),
                    // Operator::I64Popcnt => todo!(),
                    // Operator::I64Add => todo!(),
                    // Operator::I64Sub => todo!(),
                    // Operator::I64Mul => todo!(),
                    // Operator::I64DivS => todo!(),
                    // Operator::I64DivU => todo!(),
                    // Operator::I64RemS => todo!(),
                    // Operator::I64RemU => todo!(),
                    // Operator::I64And => todo!(),
                    // Operator::I64Or => todo!(),
                    // Operator::I64Xor => todo!(),
                    // Operator::I64Shl => todo!(),
                    // Operator::I64ShrS => todo!(),
                    // Operator::I64ShrU => todo!(),
                    // Operator::I64Rotl => todo!(),
                    // Operator::I64Rotr => todo!(),
                    // Operator::F32Abs => todo!(),
                    // Operator::F32Neg => todo!(),
                    // Operator::F32Ceil => todo!(),
                    // Operator::F32Floor => todo!(),
                    // Operator::F32Trunc => todo!(),
                    // Operator::F32Nearest => todo!(),
                    // Operator::F32Sqrt => todo!(),
                    // Operator::F32Add => todo!(),
                    // Operator::F32Sub => todo!(),
                    // Operator::F32Mul => todo!(),
                    // Operator::F32Div => todo!(),
                    // Operator::F32Min => todo!(),
                    // Operator::F32Max => todo!(),
                    // Operator::F32Copysign => todo!(),
                    // Operator::F64Abs => todo!(),
                    // Operator::F64Neg => todo!(),
                    // Operator::F64Ceil => todo!(),
                    // Operator::F64Floor => todo!(),
                    // Operator::F64Trunc => todo!(),
                    // Operator::F64Nearest => todo!(),
                    // Operator::F64Sqrt => todo!(),
                    // Operator::F64Add => todo!(),
                    // Operator::F64Sub => todo!(),
                    // Operator::F64Mul => todo!(),
                    // Operator::F64Div => todo!(),
                    // Operator::F64Min => todo!(),
                    // Operator::F64Max => todo!(),
                    // Operator::F64Copysign => todo!(),
                    // Operator::I32WrapI64 => todo!(),
                    // Operator::I32TruncF32S => todo!(),
                    // Operator::I32TruncF32U => todo!(),
                    // Operator::I32TruncF64S => todo!(),
                    // Operator::I32TruncF64U => todo!(),
                    // Operator::I64ExtendI32S => todo!(),
                    // Operator::I64ExtendI32U => todo!(),
                    // Operator::I64TruncF32S => todo!(),
                    // Operator::I64TruncF32U => todo!(),
                    // Operator::I64TruncF64S => todo!(),
                    // Operator::I64TruncF64U => todo!(),
                    // Operator::F32ConvertI32S => todo!(),
                    // Operator::F32ConvertI32U => todo!(),
                    // Operator::F32ConvertI64S => todo!(),
                    // Operator::F32ConvertI64U => todo!(),
                    // Operator::F32DemoteF64 => todo!(),
                    // Operator::F64ConvertI32S => todo!(),
                    // Operator::F64ConvertI32U => todo!(),
                    // Operator::F64ConvertI64S => todo!(),
                    // Operator::F64ConvertI64U => todo!(),
                    // Operator::F64PromoteF32 => todo!(),
                    // Operator::I32ReinterpretF32 => todo!(),
                    // Operator::I64ReinterpretF64 => todo!(),
                    // Operator::F32ReinterpretI32 => todo!(),
                    // Operator::F64ReinterpretI64 => todo!(),
                    // Operator::I32Extend8S => todo!(),
                    // Operator::I32Extend16S => todo!(),
                    // Operator::I64Extend8S => todo!(),
                    // Operator::I64Extend16S => todo!(),
                    // Operator::I64Extend32S => todo!(),
                    // Operator::I32TruncSatF32S => todo!(),
                    // Operator::I32TruncSatF32U => todo!(),
                    // Operator::I32TruncSatF64S => todo!(),
                    // Operator::I32TruncSatF64U => todo!(),
                    // Operator::I64TruncSatF32S => todo!(),
                    // Operator::I64TruncSatF32U => todo!(),
                    // Operator::I64TruncSatF64S => todo!(),
                    // Operator::I64TruncSatF64U => todo!(),
                    // Operator::MemoryInit { segment, mem } => todo!(),
                    // Operator::DataDrop { segment } => todo!(),
                    // Operator::MemoryCopy { src, dst } => todo!(),
                    // Operator::MemoryFill { mem } => todo!(),
                    // Operator::TableInit { segment, table } => todo!(),
                    // Operator::ElemDrop { segment } => todo!(),
                    // Operator::TableCopy {
                    //     dst_table,
                    //     src_table,
                    // } => todo!(),
                    // Operator::TableFill { table } => todo!(),
                    // Operator::TableGet { table } => todo!(),
                    // Operator::TableSet { table } => todo!(),
                    // Operator::TableGrow { table } => todo!(),
                    // Operator::TableSize { table } => todo!(),
                    // Operator::MemoryAtomicNotify { memarg } => todo!(),
                    // Operator::MemoryAtomicWait32 { memarg } => todo!(),
                    // Operator::MemoryAtomicWait64 { memarg } => todo!(),
                    // Operator::AtomicFence { flags } => todo!(),
                    // Operator::I32AtomicLoad { memarg } => todo!(),
                    // Operator::I64AtomicLoad { memarg } => todo!(),
                    // Operator::I32AtomicLoad8U { memarg } => todo!(),
                    // Operator::I32AtomicLoad16U { memarg } => todo!(),
                    // Operator::I64AtomicLoad8U { memarg } => todo!(),
                    // Operator::I64AtomicLoad16U { memarg } => todo!(),
                    // Operator::I64AtomicLoad32U { memarg } => todo!(),
                    // Operator::I32AtomicStore { memarg } => todo!(),
                    // Operator::I64AtomicStore { memarg } => todo!(),
                    // Operator::I32AtomicStore8 { memarg } => todo!(),
                    // Operator::I32AtomicStore16 { memarg } => todo!(),
                    // Operator::I64AtomicStore8 { memarg } => todo!(),
                    // Operator::I64AtomicStore16 { memarg } => todo!(),
                    // Operator::I64AtomicStore32 { memarg } => todo!(),
                    // Operator::I32AtomicRmwAdd { memarg } => todo!(),
                    // Operator::I64AtomicRmwAdd { memarg } => todo!(),
                    // Operator::I32AtomicRmw8AddU { memarg } => todo!(),
                    // Operator::I32AtomicRmw16AddU { memarg } => todo!(),
                    // Operator::I64AtomicRmw8AddU { memarg } => todo!(),
                    // Operator::I64AtomicRmw16AddU { memarg } => todo!(),
                    // Operator::I64AtomicRmw32AddU { memarg } => todo!(),
                    // Operator::I32AtomicRmwSub { memarg } => todo!(),
                    // Operator::I64AtomicRmwSub { memarg } => todo!(),
                    // Operator::I32AtomicRmw8SubU { memarg } => todo!(),
                    // Operator::I32AtomicRmw16SubU { memarg } => todo!(),
                    // Operator::I64AtomicRmw8SubU { memarg } => todo!(),
                    // Operator::I64AtomicRmw16SubU { memarg } => todo!(),
                    // Operator::I64AtomicRmw32SubU { memarg } => todo!(),
                    // Operator::I32AtomicRmwAnd { memarg } => todo!(),
                    // Operator::I64AtomicRmwAnd { memarg } => todo!(),
                    // Operator::I32AtomicRmw8AndU { memarg } => todo!(),
                    // Operator::I32AtomicRmw16AndU { memarg } => todo!(),
                    // Operator::I64AtomicRmw8AndU { memarg } => todo!(),
                    // Operator::I64AtomicRmw16AndU { memarg } => todo!(),
                    // Operator::I64AtomicRmw32AndU { memarg } => todo!(),
                    // Operator::I32AtomicRmwOr { memarg } => todo!(),
                    // Operator::I64AtomicRmwOr { memarg } => todo!(),
                    // Operator::I32AtomicRmw8OrU { memarg } => todo!(),
                    // Operator::I32AtomicRmw16OrU { memarg } => todo!(),
                    // Operator::I64AtomicRmw8OrU { memarg } => todo!(),
                    // Operator::I64AtomicRmw16OrU { memarg } => todo!(),
                    // Operator::I64AtomicRmw32OrU { memarg } => todo!(),
                    // Operator::I32AtomicRmwXor { memarg } => todo!(),
                    // Operator::I64AtomicRmwXor { memarg } => todo!(),
                    // Operator::I32AtomicRmw8XorU { memarg } => todo!(),
                    // Operator::I32AtomicRmw16XorU { memarg } => todo!(),
                    // Operator::I64AtomicRmw8XorU { memarg } => todo!(),
                    // Operator::I64AtomicRmw16XorU { memarg } => todo!(),
                    // Operator::I64AtomicRmw32XorU { memarg } => todo!(),
                    // Operator::I32AtomicRmwXchg { memarg } => todo!(),
                    // Operator::I64AtomicRmwXchg { memarg } => todo!(),
                    // Operator::I32AtomicRmw8XchgU { memarg } => todo!(),
                    // Operator::I32AtomicRmw16XchgU { memarg } => todo!(),
                    // Operator::I64AtomicRmw8XchgU { memarg } => todo!(),
                    // Operator::I64AtomicRmw16XchgU { memarg } => todo!(),
                    // Operator::I64AtomicRmw32XchgU { memarg } => todo!(),
                    // Operator::I32AtomicRmwCmpxchg { memarg } => todo!(),
                    // Operator::I64AtomicRmwCmpxchg { memarg } => todo!(),
                    // Operator::I32AtomicRmw8CmpxchgU { memarg } => todo!(),
                    // Operator::I32AtomicRmw16CmpxchgU { memarg } => todo!(),
                    // Operator::I64AtomicRmw8CmpxchgU { memarg } => todo!(),
                    // Operator::I64AtomicRmw16CmpxchgU { memarg } => todo!(),
                    // Operator::I64AtomicRmw32CmpxchgU { memarg } => todo!(),
                    // Operator::V128Load { memarg } => todo!(),
                    // Operator::V128Load8x8S { memarg } => todo!(),
                    // Operator::V128Load8x8U { memarg } => todo!(),
                    // Operator::V128Load16x4S { memarg } => todo!(),
                    // Operator::V128Load16x4U { memarg } => todo!(),
                    // Operator::V128Load32x2S { memarg } => todo!(),
                    // Operator::V128Load32x2U { memarg } => todo!(),
                    // Operator::V128Load8Splat { memarg } => todo!(),
                    // Operator::V128Load16Splat { memarg } => todo!(),
                    // Operator::V128Load32Splat { memarg } => todo!(),
                    // Operator::V128Load64Splat { memarg } => todo!(),
                    // Operator::V128Load32Zero { memarg } => todo!(),
                    // Operator::V128Load64Zero { memarg } => todo!(),
                    // Operator::V128Store { memarg } => todo!(),
                    // Operator::V128Load8Lane { memarg, lane } => todo!(),
                    // Operator::V128Load16Lane { memarg, lane } => todo!(),
                    // Operator::V128Load32Lane { memarg, lane } => todo!(),
                    // Operator::V128Load64Lane { memarg, lane } => todo!(),
                    // Operator::V128Store8Lane { memarg, lane } => todo!(),
                    // Operator::V128Store16Lane { memarg, lane } => todo!(),
                    // Operator::V128Store32Lane { memarg, lane } => todo!(),
                    // Operator::V128Store64Lane { memarg, lane } => todo!(),
                    // Operator::V128Const { value } => todo!(),
                    // Operator::I8x16Shuffle { lanes } => todo!(),
                    // Operator::I8x16ExtractLaneS { lane } => todo!(),
                    // Operator::I8x16ExtractLaneU { lane } => todo!(),
                    // Operator::I8x16ReplaceLane { lane } => todo!(),
                    // Operator::I16x8ExtractLaneS { lane } => todo!(),
                    // Operator::I16x8ExtractLaneU { lane } => todo!(),
                    // Operator::I16x8ReplaceLane { lane } => todo!(),
                    // Operator::I32x4ExtractLane { lane } => todo!(),
                    // Operator::I32x4ReplaceLane { lane } => todo!(),
                    // Operator::I64x2ExtractLane { lane } => todo!(),
                    // Operator::I64x2ReplaceLane { lane } => todo!(),
                    // Operator::F32x4ExtractLane { lane } => todo!(),
                    // Operator::F32x4ReplaceLane { lane } => todo!(),
                    // Operator::F64x2ExtractLane { lane } => todo!(),
                    // Operator::F64x2ReplaceLane { lane } => todo!(),
                    // Operator::I8x16Swizzle => todo!(),
                    // Operator::I8x16Splat => todo!(),
                    // Operator::I16x8Splat => todo!(),
                    // Operator::I32x4Splat => todo!(),
                    // Operator::I64x2Splat => todo!(),
                    // Operator::F32x4Splat => todo!(),
                    // Operator::F64x2Splat => todo!(),
                    // Operator::I8x16Eq => todo!(),
                    // Operator::I8x16Ne => todo!(),
                    // Operator::I8x16LtS => todo!(),
                    // Operator::I8x16LtU => todo!(),
                    // Operator::I8x16GtS => todo!(),
                    // Operator::I8x16GtU => todo!(),
                    // Operator::I8x16LeS => todo!(),
                    // Operator::I8x16LeU => todo!(),
                    // Operator::I8x16GeS => todo!(),
                    // Operator::I8x16GeU => todo!(),
                    // Operator::I16x8Eq => todo!(),
                    // Operator::I16x8Ne => todo!(),
                    // Operator::I16x8LtS => todo!(),
                    // Operator::I16x8LtU => todo!(),
                    // Operator::I16x8GtS => todo!(),
                    // Operator::I16x8GtU => todo!(),
                    // Operator::I16x8LeS => todo!(),
                    // Operator::I16x8LeU => todo!(),
                    // Operator::I16x8GeS => todo!(),
                    // Operator::I16x8GeU => todo!(),
                    // Operator::I32x4Eq => todo!(),
                    // Operator::I32x4Ne => todo!(),
                    // Operator::I32x4LtS => todo!(),
                    // Operator::I32x4LtU => todo!(),
                    // Operator::I32x4GtS => todo!(),
                    // Operator::I32x4GtU => todo!(),
                    // Operator::I32x4LeS => todo!(),
                    // Operator::I32x4LeU => todo!(),
                    // Operator::I32x4GeS => todo!(),
                    // Operator::I32x4GeU => todo!(),
                    // Operator::I64x2Eq => todo!(),
                    // Operator::I64x2Ne => todo!(),
                    // Operator::I64x2LtS => todo!(),
                    // Operator::I64x2GtS => todo!(),
                    // Operator::I64x2LeS => todo!(),
                    // Operator::I64x2GeS => todo!(),
                    // Operator::F32x4Eq => todo!(),
                    // Operator::F32x4Ne => todo!(),
                    // Operator::F32x4Lt => todo!(),
                    // Operator::F32x4Gt => todo!(),
                    // Operator::F32x4Le => todo!(),
                    // Operator::F32x4Ge => todo!(),
                    // Operator::F64x2Eq => todo!(),
                    // Operator::F64x2Ne => todo!(),
                    // Operator::F64x2Lt => todo!(),
                    // Operator::F64x2Gt => todo!(),
                    // Operator::F64x2Le => todo!(),
                    // Operator::F64x2Ge => todo!(),
                    // Operator::V128Not => todo!(),
                    // Operator::V128And => todo!(),
                    // Operator::V128AndNot => todo!(),
                    // Operator::V128Or => todo!(),
                    // Operator::V128Xor => todo!(),
                    // Operator::V128Bitselect => todo!(),
                    // Operator::V128AnyTrue => todo!(),
                    // Operator::I8x16Abs => todo!(),
                    // Operator::I8x16Neg => todo!(),
                    // Operator::I8x16Popcnt => todo!(),
                    // Operator::I8x16AllTrue => todo!(),
                    // Operator::I8x16Bitmask => todo!(),
                    // Operator::I8x16NarrowI16x8S => todo!(),
                    // Operator::I8x16NarrowI16x8U => todo!(),
                    // Operator::I8x16Shl => todo!(),
                    // Operator::I8x16ShrS => todo!(),
                    // Operator::I8x16ShrU => todo!(),
                    // Operator::I8x16Add => todo!(),
                    // Operator::I8x16AddSatS => todo!(),
                    // Operator::I8x16AddSatU => todo!(),
                    // Operator::I8x16Sub => todo!(),
                    // Operator::I8x16SubSatS => todo!(),
                    // Operator::I8x16SubSatU => todo!(),
                    // Operator::I8x16MinS => todo!(),
                    // Operator::I8x16MinU => todo!(),
                    // Operator::I8x16MaxS => todo!(),
                    // Operator::I8x16MaxU => todo!(),
                    // Operator::I8x16RoundingAverageU => todo!(),
                    // Operator::I16x8ExtAddPairwiseI8x16S => todo!(),
                    // Operator::I16x8ExtAddPairwiseI8x16U => todo!(),
                    // Operator::I16x8Abs => todo!(),
                    // Operator::I16x8Neg => todo!(),
                    // Operator::I16x8Q15MulrSatS => todo!(),
                    // Operator::I16x8AllTrue => todo!(),
                    // Operator::I16x8Bitmask => todo!(),
                    // Operator::I16x8NarrowI32x4S => todo!(),
                    // Operator::I16x8NarrowI32x4U => todo!(),
                    // Operator::I16x8ExtendLowI8x16S => todo!(),
                    // Operator::I16x8ExtendHighI8x16S => todo!(),
                    // Operator::I16x8ExtendLowI8x16U => todo!(),
                    // Operator::I16x8ExtendHighI8x16U => todo!(),
                    // Operator::I16x8Shl => todo!(),
                    // Operator::I16x8ShrS => todo!(),
                    // Operator::I16x8ShrU => todo!(),
                    // Operator::I16x8Add => todo!(),
                    // Operator::I16x8AddSatS => todo!(),
                    // Operator::I16x8AddSatU => todo!(),
                    // Operator::I16x8Sub => todo!(),
                    // Operator::I16x8SubSatS => todo!(),
                    // Operator::I16x8SubSatU => todo!(),
                    // Operator::I16x8Mul => todo!(),
                    // Operator::I16x8MinS => todo!(),
                    // Operator::I16x8MinU => todo!(),
                    // Operator::I16x8MaxS => todo!(),
                    // Operator::I16x8MaxU => todo!(),
                    // Operator::I16x8RoundingAverageU => todo!(),
                    // Operator::I16x8ExtMulLowI8x16S => todo!(),
                    // Operator::I16x8ExtMulHighI8x16S => todo!(),
                    // Operator::I16x8ExtMulLowI8x16U => todo!(),
                    // Operator::I16x8ExtMulHighI8x16U => todo!(),
                    // Operator::I32x4ExtAddPairwiseI16x8S => todo!(),
                    // Operator::I32x4ExtAddPairwiseI16x8U => todo!(),
                    // Operator::I32x4Abs => todo!(),
                    // Operator::I32x4Neg => todo!(),
                    // Operator::I32x4AllTrue => todo!(),
                    // Operator::I32x4Bitmask => todo!(),
                    // Operator::I32x4ExtendLowI16x8S => todo!(),
                    // Operator::I32x4ExtendHighI16x8S => todo!(),
                    // Operator::I32x4ExtendLowI16x8U => todo!(),
                    // Operator::I32x4ExtendHighI16x8U => todo!(),
                    // Operator::I32x4Shl => todo!(),
                    // Operator::I32x4ShrS => todo!(),
                    // Operator::I32x4ShrU => todo!(),
                    // Operator::I32x4Add => todo!(),
                    // Operator::I32x4Sub => todo!(),
                    // Operator::I32x4Mul => todo!(),
                    // Operator::I32x4MinS => todo!(),
                    // Operator::I32x4MinU => todo!(),
                    // Operator::I32x4MaxS => todo!(),
                    // Operator::I32x4MaxU => todo!(),
                    // Operator::I32x4DotI16x8S => todo!(),
                    // Operator::I32x4ExtMulLowI16x8S => todo!(),
                    // Operator::I32x4ExtMulHighI16x8S => todo!(),
                    // Operator::I32x4ExtMulLowI16x8U => todo!(),
                    // Operator::I32x4ExtMulHighI16x8U => todo!(),
                    // Operator::I64x2Abs => todo!(),
                    // Operator::I64x2Neg => todo!(),
                    // Operator::I64x2AllTrue => todo!(),
                    // Operator::I64x2Bitmask => todo!(),
                    // Operator::I64x2ExtendLowI32x4S => todo!(),
                    // Operator::I64x2ExtendHighI32x4S => todo!(),
                    // Operator::I64x2ExtendLowI32x4U => todo!(),
                    // Operator::I64x2ExtendHighI32x4U => todo!(),
                    // Operator::I64x2Shl => todo!(),
                    // Operator::I64x2ShrS => todo!(),
                    // Operator::I64x2ShrU => todo!(),
                    // Operator::I64x2Add => todo!(),
                    // Operator::I64x2Sub => todo!(),
                    // Operator::I64x2Mul => todo!(),
                    // Operator::I64x2ExtMulLowI32x4S => todo!(),
                    // Operator::I64x2ExtMulHighI32x4S => todo!(),
                    // Operator::I64x2ExtMulLowI32x4U => todo!(),
                    // Operator::I64x2ExtMulHighI32x4U => todo!(),
                    // Operator::F32x4Ceil => todo!(),
                    // Operator::F32x4Floor => todo!(),
                    // Operator::F32x4Trunc => todo!(),
                    // Operator::F32x4Nearest => todo!(),
                    // Operator::F32x4Abs => todo!(),
                    // Operator::F32x4Neg => todo!(),
                    // Operator::F32x4Sqrt => todo!(),
                    // Operator::F32x4Add => todo!(),
                    // Operator::F32x4Sub => todo!(),
                    // Operator::F32x4Mul => todo!(),
                    // Operator::F32x4Div => todo!(),
                    // Operator::F32x4Min => todo!(),
                    // Operator::F32x4Max => todo!(),
                    // Operator::F32x4PMin => todo!(),
                    // Operator::F32x4PMax => todo!(),
                    // Operator::F64x2Ceil => todo!(),
                    // Operator::F64x2Floor => todo!(),
                    // Operator::F64x2Trunc => todo!(),
                    // Operator::F64x2Nearest => todo!(),
                    // Operator::F64x2Abs => todo!(),
                    // Operator::F64x2Neg => todo!(),
                    // Operator::F64x2Sqrt => todo!(),
                    // Operator::F64x2Add => todo!(),
                    // Operator::F64x2Sub => todo!(),
                    // Operator::F64x2Mul => todo!(),
                    // Operator::F64x2Div => todo!(),
                    // Operator::F64x2Min => todo!(),
                    // Operator::F64x2Max => todo!(),
                    // Operator::F64x2PMin => todo!(),
                    // Operator::F64x2PMax => todo!(),
                    // Operator::I32x4TruncSatF32x4S => todo!(),
                    // Operator::I32x4TruncSatF32x4U => todo!(),
                    // Operator::F32x4ConvertI32x4S => todo!(),
                    // Operator::F32x4ConvertI32x4U => todo!(),
                    // Operator::I32x4TruncSatF64x2SZero => todo!(),
                    // Operator::I32x4TruncSatF64x2UZero => todo!(),
                    // Operator::F64x2ConvertLowI32x4S => todo!(),
                    // Operator::F64x2ConvertLowI32x4U => todo!(),
                    // Operator::F32x4DemoteF64x2Zero => todo!(),
                    // Operator::F64x2PromoteLowF32x4 => todo!(),
                    // Operator::I8x16SwizzleRelaxed => todo!(),
                    // Operator::I32x4TruncSatF32x4SRelaxed => todo!(),
                    // Operator::I32x4TruncSatF32x4URelaxed => todo!(),
                    // Operator::I32x4TruncSatF64x2SZeroRelaxed => todo!(),
                    // Operator::I32x4TruncSatF64x2UZeroRelaxed => todo!(),
                    // Operator::F32x4FmaRelaxed => todo!(),
                    // Operator::F32x4FmsRelaxed => todo!(),
                    // Operator::F64x2FmaRelaxed => todo!(),
                    // Operator::F64x2FmsRelaxed => todo!(),
                    // Operator::I8x16LaneSelect => todo!(),
                    // Operator::I16x8LaneSelect => todo!(),
                    // Operator::I32x4LaneSelect => todo!(),
                    // Operator::I64x2LaneSelect => todo!(),
                    // Operator::F32x4MinRelaxed => todo!(),
                    // Operator::F32x4MaxRelaxed => todo!(),
                    // Operator::F64x2MinRelaxed => todo!(),
                    // Operator::F64x2MaxRelaxed => todo!(),
                    _ => {}
                }
            }
        }
        // Increment body index.
        *body_index += 1;

        Ok(())
    }
}

#[cfg(test)]
mod compiler_tests {}
