// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

/// `Imports` is a set of user-supplied objects that are exposed to a WebAssembly `Instance`.
///
/// It is different from compiler `Imports` type because it does not necessarily contain a resolution of all the imports an Instance needs.
pub struct Imports {}
