use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::convert::{From, TryFrom}; // Importar From y TryFrom

pub use macro_toli::tool;

pub trait IATool {
    fn call(&self, args: HashMap<String, WrappedData>) -> WrappedData;
    fn get_description(&self) -> IAToolDefinition;
}

pub struct IAToolSet {
    pub tools: HashMap<&'static str, Box<dyn IATool>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IAToolDefinition {
    pub name: String,
    pub description: String,
    pub arguments: HashMap<String, IAArgument>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IAArgument {
    pub name: String,
    pub description: String,
    pub arg_type: ArgumentType,
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ArgumentType {
    I8, U8,
    I16, U16,
    I32, U32,
    I64, U64,
    Text,
    Boolean,
    Float,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WrappedData {
    Number(WrappedInt),
    Text(String),
    Boolean(bool),
    Float(f64),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WrappedInt {
    I8(i8), U8(u8),
    I16(i16), U16(u16),
    I32(i32), U32(u32),
    I64(i64), U64(u64),
}

// --- Implementaciones From<PrimitiveType> for WrappedInt ---
impl From<i8> for WrappedInt { fn from(val: i8) -> Self { WrappedInt::I8(val) } }
impl From<u8> for WrappedInt { fn from(val: u8) -> Self { WrappedInt::U8(val) } }
impl From<i16> for WrappedInt { fn from(val: i16) -> Self { WrappedInt::I16(val) } }
impl From<u16> for WrappedInt { fn from(val: u16) -> Self { WrappedInt::U16(val) } }
impl From<i32> for WrappedInt { fn from(val: i32) -> Self { WrappedInt::I32(val) } }
impl From<u32> for WrappedInt { fn from(val: u32) -> Self { WrappedInt::U32(val) } }
impl From<i64> for WrappedInt { fn from(val: i64) -> Self { WrappedInt::I64(val) } }
impl From<u64> for WrappedInt { fn from(val: u64) -> Self { WrappedInt::U64(val) } }

// --- TryFrom<WrappedInt> for PrimitiveType Implementations ---
impl TryFrom<WrappedInt> for i8 {
    type Error = &'static str;
    fn try_from(wrapped: WrappedInt) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedInt::I8(v) => Ok(v),
            WrappedInt::U8(v) => v.try_into().map_err(|_| "WrappedInt::U8 out of range for i8"),
            WrappedInt::I16(v) => v.try_into().map_err(|_| "WrappedInt::I16 out of range for i8"),
            WrappedInt::U16(v) => v.try_into().map_err(|_| "WrappedInt::U16 out of range for i8"),
            WrappedInt::I32(v) => v.try_into().map_err(|_| "WrappedInt::I32 out of range for i8"),
            WrappedInt::U32(v) => v.try_into().map_err(|_| "WrappedInt::U32 out of range for i8"),
            WrappedInt::I64(v) => v.try_into().map_err(|_| "WrappedInt::I64 out of range for i8"),
            WrappedInt::U64(v) => v.try_into().map_err(|_| "WrappedInt::U64 out of range for i8"),
        }
    }
}

impl TryFrom<WrappedInt> for u8 {
    type Error = &'static str;
    fn try_from(wrapped: WrappedInt) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedInt::U8(v) => Ok(v),
            WrappedInt::I8(v) => v.try_into().map_err(|_| "WrappedInt::I8 out of range for u8"),
            WrappedInt::I16(v) => v.try_into().map_err(|_| "WrappedInt::I16 out of range for u8"),
            WrappedInt::U16(v) => v.try_into().map_err(|_| "WrappedInt::U16 out of range for u8"),
            WrappedInt::I32(v) => v.try_into().map_err(|_| "WrappedInt::I32 out of range for u8"),
            WrappedInt::U32(v) => v.try_into().map_err(|_| "WrappedInt::U32 out of range for u8"),
            WrappedInt::I64(v) => v.try_into().map_err(|_| "WrappedInt::I64 out of range for u8"),
            WrappedInt::U64(v) => v.try_into().map_err(|_| "WrappedInt::U64 out of range for u8"),
        }
    }
}

impl TryFrom<WrappedInt> for i16 {
    type Error = &'static str;
    fn try_from(wrapped: WrappedInt) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedInt::I16(v) => Ok(v),
            WrappedInt::I8(v) => Ok(v.into()),
            WrappedInt::U8(v) => Ok(v.into()),
            WrappedInt::U16(v) => v.try_into().map_err(|_| "WrappedInt::U16 out of range for i16"),
            WrappedInt::I32(v) => v.try_into().map_err(|_| "WrappedInt::I32 out of range for i16"),
            WrappedInt::U32(v) => v.try_into().map_err(|_| "WrappedInt::U32 out of range for i16"),
            WrappedInt::I64(v) => v.try_into().map_err(|_| "WrappedInt::I64 out of range for i16"),
            WrappedInt::U64(v) => v.try_into().map_err(|_| "WrappedInt::U64 out of range for i16"),
        }
    }
}

impl TryFrom<WrappedInt> for u16 {
    type Error = &'static str;
    fn try_from(wrapped: WrappedInt) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedInt::U16(v) => Ok(v),
            WrappedInt::U8(v) => Ok(v.into()),
            WrappedInt::I8(v) => v.try_into().map_err(|_| "WrappedInt::I8 out of range for u16"),
            WrappedInt::I16(v) => v.try_into().map_err(|_| "WrappedInt::I16 out of range for u16"),
            WrappedInt::I32(v) => v.try_into().map_err(|_| "WrappedInt::I32 out of range for u16"),
            WrappedInt::U32(v) => v.try_into().map_err(|_| "WrappedInt::U32 out of range for u16"),
            WrappedInt::I64(v) => v.try_into().map_err(|_| "WrappedInt::I64 out of range for u16"),
            WrappedInt::U64(v) => v.try_into().map_err(|_| "WrappedInt::U64 out of range for u16"),
        }
    }
}

impl TryFrom<WrappedInt> for i32 {
    type Error = &'static str;
    fn try_from(wrapped: WrappedInt) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedInt::I32(v) => Ok(v),
            WrappedInt::I8(v) => Ok(v.into()),
            WrappedInt::U8(v) => Ok(v.into()),
            WrappedInt::I16(v) => Ok(v.into()),
            WrappedInt::U16(v) => Ok(v.into()),
            WrappedInt::U32(v) => v.try_into().map_err(|_| "WrappedInt::U32 out of range for i32"),
            WrappedInt::I64(v) => v.try_into().map_err(|_| "WrappedInt::I64 out of range for i32"),
            WrappedInt::U64(v) => v.try_into().map_err(|_| "WrappedInt::U64 out of range for i32"),
        }
    }
}

impl TryFrom<WrappedInt> for u32 {
    type Error = &'static str;
    fn try_from(wrapped: WrappedInt) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedInt::U32(v) => Ok(v),
            WrappedInt::U8(v) => Ok(v.into()),
            WrappedInt::U16(v) => Ok(v.into()),
            WrappedInt::I8(v) => v.try_into().map_err(|_| "WrappedInt::I8 out of range for u32"),
            WrappedInt::I16(v) => v.try_into().map_err(|_| "WrappedInt::I16 out of range for u32"),
            WrappedInt::I32(v) => v.try_into().map_err(|_| "WrappedInt::I32 out of range for u32"),
            WrappedInt::I64(v) => v.try_into().map_err(|_| "WrappedInt::I64 out of range for u32"),
            WrappedInt::U64(v) => v.try_into().map_err(|_| "WrappedInt::U64 out of range for u32"),
        }
    }
}

impl TryFrom<WrappedInt> for i64 {
    type Error = &'static str;
    fn try_from(wrapped: WrappedInt) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedInt::I64(v) => Ok(v),
            WrappedInt::I8(v) => Ok(v.into()),
            WrappedInt::U8(v) => Ok(v.into()),
            WrappedInt::I16(v) => Ok(v.into()),
            WrappedInt::U16(v) => Ok(v.into()),
            WrappedInt::I32(v) => Ok(v.into()),
            WrappedInt::U32(v) => Ok(v.into()),
            WrappedInt::U64(v) => v.try_into().map_err(|_| "WrappedInt::U64 out of range for i64"),
        }
    }
}

impl TryFrom<WrappedInt> for u64 {
    type Error = &'static str;
    fn try_from(wrapped: WrappedInt) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedInt::U64(v) => Ok(v),
            WrappedInt::U8(v) => Ok(v.into()),
            WrappedInt::U16(v) => Ok(v.into()),
            WrappedInt::U32(v) => Ok(v.into()),
            WrappedInt::I8(v) => v.try_into().map_err(|_| "WrappedInt::I8 out of range for u64"),
            WrappedInt::I16(v) => v.try_into().map_err(|_| "WrappedInt::I16 out of range for u64"),
            WrappedInt::I32(v) => v.try_into().map_err(|_| "WrappedInt::I32 out of range for u64"),
            WrappedInt::I64(v) => v.try_into().map_err(|_| "WrappedInt::I64 out of range for u64"),
        }
    }
}