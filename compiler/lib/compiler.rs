// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Archive)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug))]
pub struct EagerArtefact {}

pub struct LazyArtefact {}

pub enum Artefact {
    Lazy(LazyArtefact),
    Eager(EagerArtefact),
}

pub enum CompileMode {
    Eager,
    Lazy,
}

pub struct Compiler {
    mode: CompileMode,
}

impl Compiler {
    pub fn new(mode: CompileMode) -> Self {
        Self { mode }
    }

    pub fn compile(&self) -> Artefact {
        // TODO(appcypher): Compile the wasm bytes.
        unimplemented!()
    }

    fn _compile_lazy() -> LazyArtefact {
        // TODO(appcypher): Compile the wasm bytes.
        unimplemented!()
    }
}
