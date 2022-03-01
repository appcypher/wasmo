// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.

pub const _POINTER_SIZE: u8 = std::mem::size_of::<usize>() as u8;
pub const _LENGTH_SIZE: u8 = std::mem::size_of::<u32>() as u8;
pub const _TYPE_INDEX_SIZE: u8 = std::mem::size_of::<u32>() as u8;
pub const _LIMIT_MIN_SIZE: u8 = std::mem::size_of::<u64>() as u8;
pub const _LIMIT_MAX_SIZE: u8 = std::mem::size_of::<u64>() as u8;
