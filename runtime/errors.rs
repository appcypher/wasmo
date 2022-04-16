use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerError {
    UnsupportedTypeSectionEntry(String),
    UnsupportedExportSectionEntry(String),
    UnsupportedImportSectionEntry(String),
    UnsupportedWasmoValType(String),
    UnsupportedMemory64Proposal,
    UnsupportedSection(String),
    UnsupportedInstruction(String),
}

impl std::error::Error for CompilerError {}

impl Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
