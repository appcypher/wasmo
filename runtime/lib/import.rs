// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

#[derive(Debug, Default)]
pub struct Imports {
    pub memories: Vec<u32>,
    pub tables: Vec<u32>,
    pub functions: Vec<u32>,
    pub globals: Vec<u32>,
}
