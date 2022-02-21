// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};
use utilities::result::Result;
use wasmparser::{Parser, Payload, TypeDef, TypeSectionReader};

use crate::{
    context::CompileTimeResolver, errors::CompilerError, store::Function, types::FuncType,
};

use super::{llvm::LLVM, utils::convert, value::Value};

#[derive(Debug, Serialize, Deserialize, Archive, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct Compiler {
    pub llvm: Option<LLVM>,
    pub resolver: CompileTimeResolver,
    pub liftoff: bool,
    pub types: Vec<FuncType>,
    pub functions: Vec<Function>,
    pub current_frame: Option<FunctionFrame>,
    // ...
}

#[derive(Debug, Serialize, Deserialize, Archive, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct FunctionFrame {
    pub locals: Vec<Value>,
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
                    self.register_func_types(reader)?;
                }
                Payload::ImportSection(reader) => {
                    println!("======= ImportSection =======");
                    reader.into_iter().for_each(|i| {
                        println!("import: {:?}", i);
                    });
                }
                Payload::AliasSection(reader) => {
                    println!("======= AliasSection =======");
                    reader.into_iter().for_each(|i| {
                        println!("alias: {:?}", i);
                    });
                }
                Payload::InstanceSection(_) => (),
                Payload::FunctionSection(reader) => {
                    println!("======= FunctionSection =======");
                    reader.into_iter().for_each(|i| {
                        println!("function: {:?}", i);
                    });
                }
                Payload::TableSection(reader) => {
                    println!("======= TableSection =======");
                    reader.into_iter().for_each(|i| {
                        println!("table: {:?}", i);
                    });
                }
                Payload::MemorySection(reader) => {
                    println!("======= MemorySection =======");
                    reader.into_iter().for_each(|i| {
                        println!("memory: {:?}", i);
                    });
                }
                Payload::TagSection(reader) => {
                    println!("======= TagSection =======");
                    reader.into_iter().for_each(|i| {
                        println!("tag: {:?}", i);
                    });
                }
                Payload::GlobalSection(reader) => {
                    println!("======= GlobalSection =======");
                    reader.into_iter().for_each(|i| {
                        println!("global: {:?}", i);
                    });
                }
                Payload::ExportSection(reader) => {
                    println!("======= ExportSection =======");
                    reader.into_iter().for_each(|i| {
                        println!("export: {:?}", i);
                    });
                }
                Payload::StartSection { func, range } => {
                    println!("======= StartSection =======");
                    println!("start func: {:?}", func);
                    println!("range: {:?}", range);
                }
                Payload::ElementSection(reader) => {
                    println!("======= ElementSection =======");
                    reader.into_iter().for_each(|r| {
                        r.iter().for_each(|i| {
                            println!("element.range: {:?}", i.range);
                            println!("element.ty: {:?}", i.ty);
                            println!("element.items: {:?}", i.items);
                        });
                    });
                }
                Payload::DataCountSection { count, range } => {
                    println!("======= DataCountSection =======");
                    println!("data count: {:?}", count);
                    println!("range: {:?}", range);
                }
                Payload::DataSection(reader) => {
                    println!("======= DataSection =======");
                    reader.into_iter().for_each(|i| {
                        println!("data: {:?}", i);
                    });
                }
                Payload::CustomSection {
                    name,
                    data_offset,
                    data,
                    range,
                } => {
                    println!("======= CustomSection =======");
                    println!("custom section name: {:?}", name);
                    println!("data offset: {:?}", data_offset);
                    println!("data: {:?}", data);
                    println!("range: {:?}", range);
                }
                Payload::CodeSectionStart { count, range, size } => {
                    println!("======= CodeSectionStart =======");
                    println!("code section start count: {:?}", count);
                    println!("range: {:?}", range);
                    println!("size: {:?}", size);
                }
                Payload::CodeSectionEntry(body) => {
                    println!("======= CodeSectionEntry =======");
                    println!("function body: {:?}", body);
                    body.get_locals_reader().into_iter().for_each(|r| {
                        r.into_iter().for_each(|i| {
                            println!("local: {:?}", i);
                        });
                    });

                    body.get_operators_reader().into_iter().for_each(|r| {
                        r.into_iter().for_each(|i| {
                            println!("operator: {:?}", i);
                        });
                    });
                }
                Payload::ModuleSectionStart { count, range, size } => {
                    println!("======= ModuleSectionStart =======");
                    println!("module section start count: {:?}", count);
                    println!("range: {:?}", range);
                    println!("size: {:?}", size);
                }
                Payload::ModuleSectionEntry { parser, range } => {
                    println!("======= ModuleSectionEntry =======");
                    println!("module section entry: {:?}", parser);
                    println!("range: {:?}", range);
                }
                Payload::UnknownSection {
                    id,
                    contents,
                    range,
                } => {
                    println!("======= UnknownSection =======");
                    println!("unknown section id: {:?}", id);
                    println!("contents: {:?}", contents);
                    println!("range: {:?}", range);
                }
                Payload::End => {
                    println!("======= End =======");
                    println!("end");
                }
            }
        }

        Ok(())
    }

    /// Registers function type in type section.
    pub fn register_func_types(&mut self, reader: TypeSectionReader) -> Result<()> {
        for result in reader.into_iter() {
            match result? {
                TypeDef::Func(ty) => {
                    self.types.push(convert::to_wasmo_functype(&ty)?);
                }
                TypeDef::Instance(_) => return Err(CompilerError::UnsupportedInstanceType.into()),
                TypeDef::Module(_) => return Err(CompilerError::UnsupportedModuleType.into()),
            };
        }

        Ok(())
    }
}
