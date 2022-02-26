// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use bytecheck::CheckBytes;
use log::debug;
use rkyv::{Archive, Deserialize, Serialize};
use utilities::result::Result;
use wasmparser::{
    DataSectionReader, ElementSectionReader, ExportSectionReader, FunctionSectionReader,
    GlobalSectionReader, ImportSectionEntryType, ImportSectionReader, MemorySectionReader, Parser,
    Payload, TableSectionReader, TypeDef, TypeSectionReader,
};

use crate::{
    context::CompileTimeResolver,
    errors::CompilerError,
    store::{Data, Element, Function, Global, Memory, Table},
    types::{FuncType, Limits},
};

use super::{
    exports::{Export, Exports},
    imports::{Import, Imports},
    llvm::LLVM,
    utils::convert,
    value::Value,
};

/// The compiler is responsible for compiling a module.
#[derive(Debug, Serialize, Deserialize, Archive, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct Compiler {
    /// Represents the result of LLVM codegen.
    pub llvm: Option<LLVM>,
    /// The address resolver.
    pub resolver: CompileTimeResolver,
    /// Option for enabling lift-off compilation.
    pub liftoff: bool,
    /// List of imported components of a module.
    pub imports: Imports,
    /// List of exported components of a module.
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
    /// Represents the current function being compiled.
    pub current_frame: Option<FunctionFrame>,
}

/// Represents the current function being compiled.
#[derive(Debug, Serialize, Deserialize, Archive, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct FunctionFrame {
    /// Local variables.
    pub locals: Vec<Value>,
    /// An implicit stack only needed during compilation.
    pub stack: Vec<Value>,
}

impl Compiler {
    /// Creates a new `Compiler` with the given options.
    pub fn new(liftoff: bool) -> Self {
        Self {
            liftoff,
            ..Default::default()
        }
    }

    /// Compiles the given wasm bytes.
    pub fn compile(&mut self, wasm: &[u8]) -> Result<()> {
        let mut _llvm = LLVM::new();

        for payload in Parser::new(0).parse_all(wasm) {
            match payload? {
                Payload::Version { .. } => (),
                Payload::TypeSection(reader) => {
                    debug!("======= TypeSection =======");
                    self.compile_types(reader)?;
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
                    // llvm.codegen_globals(reader)?;
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
                Payload::CustomSection {
                    name,
                    data_offset,
                    data,
                    range,
                } => {
                    debug!("======= CustomSection =======");
                    debug!("custom section name: {:?}", name);
                    debug!("data offset: {:?}", data_offset);
                    debug!("data: {:?}", data);
                    debug!("range: {:?}", range);
                }
                Payload::CodeSectionStart { .. } => {
                    debug!("======= CodeSectionStart =======");
                }
                Payload::CodeSectionEntry(body) => {
                    debug!("======= CodeSectionEntry =======");

                    debug!("function body: {:?}", body);

                    body.get_locals_reader().into_iter().for_each(|r| {
                        r.into_iter().for_each(|i| {
                            debug!("local: {:?}", i);
                        });
                    });

                    body.get_operators_reader().into_iter().for_each(|r| {
                        r.into_iter().for_each(|i| {
                            debug!("operator: {:?}", i);
                        });
                    });
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
                }
                t => {
                    return Err(CompilerError::UnsupportedSection(format!("{:?}", t)).into());
                }
            }
        }

        Ok(())
    }
}

impl Compiler {
    /// Compiles function types in type section.
    pub fn compile_types(&mut self, reader: TypeSectionReader) -> Result<()> {
        for result in reader.into_iter() {
            let typedef = result?;

            debug!("type: {:?}", typedef);

            match typedef {
                TypeDef::Func(ty) => {
                    self.types.push(convert::to_wasmo_functype(&ty)?);
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

            match import.ty {
                ImportSectionEntryType::Function(index) => {
                    self.imports.functions.push(Import::new(
                        import.module.to_string(),
                        import.field.map(|s| s.to_string()),
                        self.functions.len() as u32,
                    ));

                    self.functions.push(Function::new(index));
                }
                ImportSectionEntryType::Table(ty) => {
                    self.imports.tables.push(Import::new(
                        import.module.to_string(),
                        import.field.map(|s| s.to_string()),
                        self.tables.len() as u32,
                    ));

                    self.tables.push(Table::new(
                        Limits::new(ty.initial as u64, ty.maximum.map(|x| x as u64)),
                        convert::to_wasmo_valtype(&ty.element_type)?,
                    ));
                }
                ImportSectionEntryType::Memory(ty) => {
                    // TODO(appcypher): Wasmo does not support memory64 proposal yet.
                    if ty.memory64 {
                        return Err(CompilerError::UnsupportedMemory64Proposal.into());
                    }

                    self.imports.memories.push(Import::new(
                        import.module.to_string(),
                        import.field.map(|s| s.to_string()),
                        self.memories.len() as u32,
                    ));

                    self.memories
                        .push(Memory::new(Limits::new(ty.initial, ty.maximum), ty.shared));
                }
                ImportSectionEntryType::Global(ty) => {
                    self.imports.globals.push(Import::new(
                        import.module.to_string(),
                        import.field.map(|s| s.to_string()),
                        self.globals.len() as u32,
                    ));

                    self.globals.push(Global::new(
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

            self.functions.push(Function::new(type_index));
        }

        Ok(())
    }

    /// Compiles tables in table section.
    pub fn compile_tables(&mut self, reader: TableSectionReader) -> Result<()> {
        for result in reader.into_iter() {
            let ty = result?;

            debug!("table type: {:?}", ty);

            self.tables.push(Table::new(
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

            self.memories
                .push(Memory::new(Limits::new(ty.initial, ty.maximum), ty.shared));
        }

        Ok(())
    }

    /// Compiles globals in global section.
    pub fn compile_globals(&mut self, reader: GlobalSectionReader) -> Result<()> {
        for result in reader.into_iter() {
            let global = result?;

            debug!("global: {:?}", global);

            self.globals.push(Global::new(
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

            self.data
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

            self.elements
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
                    self.exports
                        .functions
                        .push(Export::new(export.field.to_string(), export.index));
                }
                wasmparser::ExternalKind::Table => {
                    self.exports
                        .tables
                        .push(Export::new(export.field.to_string(), export.index));
                }
                wasmparser::ExternalKind::Memory => {
                    self.exports
                        .memories
                        .push(Export::new(export.field.to_string(), export.index));
                }
                wasmparser::ExternalKind::Global => {
                    self.exports
                        .globals
                        .push(Export::new(export.field.to_string(), export.index));
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
        // llvm.codegen_start_function(reader)?;
        Ok(())
    }
}
