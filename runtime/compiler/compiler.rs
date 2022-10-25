use std::pin::Pin;

use anyhow::Result;
use llvm::LLVM;
use log::debug;
use serde::{Deserialize, Serialize};
use wasmparser::{
    DataSectionReader, ElementSectionReader, ExportSectionReader, FunctionSectionReader, GlobalSectionReader,
    ImportSectionReader, MemorySectionReader, Parser, Payload, TableSectionReader, Type, TypeRef, TypeSectionReader,
    Validator,
};

use super::{
    conversions,
    exports::{Export, Exports},
    generator::{FunctionBodyGenerator, Generator},
    imports::{Import, Imports},
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

        // The validator.
        let mut validator = Validator::new();

        // Body index.
        let mut body_index = 0;
        for payload in Parser::new(0).parse_all(wasm) {
            match payload? {
                Payload::Version {
                    num,
                    encoding,
                    ref range,
                } => {
                    validator.version(num, encoding, range)?;
                }
                Payload::TypeSection(reader) => {
                    validator.type_section(&reader)?;
                    self.compile_types(reader, &mut llvm)?;
                }
                Payload::ImportSection(reader) => {
                    validator.import_section(&reader)?;
                    self.compile_imports(reader)?;
                }
                Payload::FunctionSection(reader) => {
                    validator.function_section(&reader)?;
                    self.compile_functions(reader)?;
                }
                Payload::TableSection(reader) => {
                    validator.table_section(&reader)?;
                    self.compile_tables(reader)?;
                }
                Payload::MemorySection(reader) => {
                    validator.memory_section(&reader)?;
                    self.compile_memories(reader)?;
                }
                Payload::GlobalSection(reader) => {
                    validator.global_section(&reader)?;
                    self.compile_globals(reader)?;
                }
                Payload::ExportSection(reader) => {
                    validator.export_section(&reader)?;
                    self.compile_exports(reader)?;
                }
                Payload::StartSection { func, range } => {
                    validator.start_section(func, &range)?;
                    self.compile_start_function(func)?;
                }
                Payload::ElementSection(reader) => {
                    validator.element_section(&reader)?;
                    self.compile_elements(reader)?;
                }
                Payload::DataCountSection { count, range } => {
                    validator.data_count_section(count, &range)?;
                    // TODO(appcypher): Implement data section.
                    debug!("data section count: {:?}", count);
                }
                Payload::DataSection(reader) => {
                    validator.data_section(&reader)?;
                    self.compile_data(reader)?;
                }
                Payload::CustomSection(_) => {
                    // TODO(appcypher): Generate index space mappings to names to be used in codegen. self.compile_name_section()?;
                    debug!("custom section");
                }
                Payload::CodeSectionStart { count, range, .. } => {
                    validator.code_section_start(count, &range)?;
                }
                Payload::CodeSectionEntry(body) => {
                    validator.code_section_entry(&body)?;
                    let mut body_gen = FunctionBodyGenerator {
                        llvm: &mut llvm,
                        info: &self.info,
                        body: &body,
                        body_index,
                    };

                    body_gen.generate()?;
                    body_index += 1;
                }
                Payload::UnknownSection { id, range, .. } => {
                    validator.unknown_section(id, &range)?;
                }
                Payload::End(_) => (),
                other => {
                    validator.payload(&other)?;
                    return Err(CompilerError::UnsupportedSection(format!("{:?}", other)).into());
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
    pub(crate) fn compile_types(&mut self, reader: TypeSectionReader, llvm: &mut LLVM) -> Result<()> {
        for result in reader.into_iter() {
            let typedef = result?;

            debug!("type: {:?}", typedef);

            match &typedef {
                Type::Func(ty) => {
                    let wasmo_func_ty = ty.into();
                    let llvm_func_ty = conversions::wasmparser_to_llvm_functype(&llvm.context, ty);

                    llvm.info.types.push(llvm_func_ty);
                    self.info.types.push(wasmo_func_ty);
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
            match &import.ty {
                TypeRef::Func(index) => {
                    self.info.imports.functions.push(Import::new(
                        import.module.to_string(),
                        import.name.to_string(),
                        self.info.functions.len() as u32,
                    ));

                    self.info.functions.push(Function::new(*index));
                }
                TypeRef::Table(ty) => {
                    self.info.imports.tables.push(Import::new(
                        import.module.to_string(),
                        import.name.to_string(),
                        self.info.tables.len() as u32,
                    ));

                    self.info.tables.push(Table::new(
                        Limits::new(ty.initial as u64, ty.maximum.map(|x| x as u64)),
                        (&ty.element_type).into(),
                    ));
                }
                TypeRef::Memory(ty) => {
                    // TODO(appcypher): Wasmo does not support memory64 proposal yet.
                    if ty.memory64 {
                        return Err(CompilerError::UnsupportedMemory64Proposal.into());
                    }

                    self.info.imports.memories.push(Import::new(
                        import.module.to_string(),
                        import.name.to_string(),
                        self.info.memories.len() as u32,
                    ));

                    self.info
                        .memories
                        .push(Memory::new(Limits::new(ty.initial, ty.maximum), ty.shared));
                }
                TypeRef::Global(ty) => {
                    self.info.imports.globals.push(Import::new(
                        import.module.to_string(),
                        import.name.to_string(),
                        self.info.globals.len() as u32,
                    ));

                    self.info
                        .globals
                        .push(Global::new((&ty.content_type).into(), ty.mutable));
                }
                t => return Err(CompilerError::UnsupportedImportSectionEntry(format!("{:?}", t)).into()),
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
                (&ty.element_type).into(),
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

            self.info
                .globals
                .push(Global::new((&global.ty.content_type).into(), global.ty.mutable));

            // TODO(appcypher): Implement this.
            // llvm.codegen_global(reader)?;
        }

        Ok(())
    }

    /// Compiles data in data section.
    pub fn compile_data(&mut self, reader: DataSectionReader) -> Result<()> {
        for result in reader.into_iter() {
            let data = result?;

            debug!("data: {:?}", data);

            self.info.data.push(Data::new((&data.kind).into()));

            // TODO(appcypher): Implement this.
            // llvm.codegen_data(reader)?;
        }

        Ok(())
    }

    /// Compiles elements in element section.
    pub fn compile_elements(&mut self, reader: ElementSectionReader) -> Result<()> {
        for result in reader.into_iter() {
            let elem = result?;

            debug!("elem items: {:?}", elem.items);

            self.info.elements.push(Element::new((&elem.kind).into()));

            // TODO(appcypher): Implement this.
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
                wasmparser::ExternalKind::Func => {
                    self.info
                        .exports
                        .inner
                        .insert(export.name.to_string(), Export::new(ExportKind::Function, export.index));
                }
                wasmparser::ExternalKind::Table => {
                    self.info
                        .exports
                        .inner
                        .insert(export.name.to_string(), Export::new(ExportKind::Table, export.index));
                }
                wasmparser::ExternalKind::Memory => {
                    self.info
                        .exports
                        .inner
                        .insert(export.name.to_string(), Export::new(ExportKind::Memory, export.index));
                }
                wasmparser::ExternalKind::Global => {
                    self.info
                        .exports
                        .inner
                        .insert(export.name.to_string(), Export::new(ExportKind::Global, export.index));
                }
                t => return Err(CompilerError::UnsupportedExportSectionEntry(format!("{:?}", t)).into()),
            }
        }

        Ok(())
    }

    /// Compiles start function.
    pub fn compile_start_function(&mut self, _func: u32) -> Result<()> {
        self.info.start_function = Some(_func);
        Ok(())
    }
}
