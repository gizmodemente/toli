//! # Toli
//!
//! `toli` is a library that provides the necessary abstractions to define and manage tools
//! that can be used by Artificial Intelligence (AI) language models.
//! It facilitates the creation of tools with typed arguments and descriptions,
//! allowing AI models to interact with external functions in a structured manner.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::convert::{From, TryFrom};
use serde_json::Value;

pub use macro_toli::tool;

/// Trait that defines the interface for an AI tool.
///
/// Any type implementing this trait can be considered a tool
/// that an AI model can invoke.
pub trait IATool {
    /// The actual return type of the function wrapped by this tool.
    type OriginalReturnType;

    /// Executes the tool's logic with the provided arguments.
    ///
    /// The arguments are provided as a `serde_json::Value`, typically representing
    /// a JSON object where keys are argument names and values are their corresponding data.
    /// The implementation will parse this JSON into a `HashMap<String, WrappedData>`
    /// before invoking the original function.
    ///
    /// # Arguments
    /// * `json_args` - A `serde_json::Value` containing the arguments for the tool.
    ///
    /// # Returns
    /// The direct result of the tool's execution, which is `Self::OriginalReturnType`.
    /// No further conversion is performed on the return value by the `IATool` trait itself.
    ///
    /// # Panics
    /// If the `json_args` cannot be parsed into the expected `HashMap<String, WrappedData>`
    /// or if argument type conversions fail.
    fn call(&self, json_string_args: String) -> Self::OriginalReturnType;

    /// Converts a `serde_json::Value` into a `HashMap<String, WrappedData>`.
    ///
    /// This function is used internally by the `call` method to prepare arguments
    /// for the original function. It validates and converts JSON values
    /// into the `WrappedData` enum based on the tool's argument definitions.
    ///
    /// # Arguments
    /// * `json_args` - The `serde_json::Value` to parse.
    ///
    /// # Returns
    /// A `HashMap<String, WrappedData>` containing the parsed arguments.
    ///
    /// # Panics
    /// Panics if the JSON structure does not match the expected argument types
    /// or if required arguments are missing.
    fn parse_json_args(&self, json_string_args: String) -> HashMap<String, WrappedData> {
        let mut parsed_args = HashMap::new();

        let json_value: Value = serde_json::from_str(&json_string_args)
            .expect("Failed to parse JSON string for tool arguments.");

        let json_obj = json_value.as_object()
            .expect("JSON input for tool arguments must be an object.");

        let tool_description = self.get_description();

        for (arg_name, arg_def) in tool_description.arguments {
            let json_value = json_obj.get(&arg_name)
                .expect(&format!("Missing required argument '{}' in JSON input.", arg_name));

            let wrapped_data = match arg_def.arg_type {
                ArgumentType::I8 => {
                    if let Some(num) = json_value.as_i64() {
                        WrappedData::Number(num.try_into().expect(&format!("Number out of range for i8: {}", num)))
                    } else if let Some(s) = json_value.as_str() {
                        let num: i8 = s.parse().expect(&format!("Argument '{}' expected i8 number or string convertible to i8, got string '{}'", arg_name, s));
                        WrappedData::Number(num.into())
                    } else {
                        panic!("Argument '{}' expected i8 number or string convertible to i8, got {:?}", arg_name, json_value);
                    }
                },
                ArgumentType::U8 => {
                    if let Some(num) = json_value.as_u64() {
                        WrappedData::Number(num.try_into().expect(&format!("Number out of range for u8: {}", num)))
                    } else if let Some(s) = json_value.as_str() {
                        let num: u8 = s.parse().expect(&format!("Argument '{}' expected u8 number or string convertible to u8, got string '{}'", arg_name, s));
                        WrappedData::Number(num.into())
                    } else {
                        panic!("Argument '{}' expected u8 number or string convertible to u8, got {:?}", arg_name, json_value);
                    }
                },
                ArgumentType::I16 => {
                    if let Some(num) = json_value.as_i64() {
                        WrappedData::Number(num.try_into().expect(&format!("Number out of range for i16: {}", num)))
                    } else if let Some(s) = json_value.as_str() {
                        let num: i16 = s.parse().expect(&format!("Argument '{}' expected i16 number or string convertible to i16, got string '{}'", arg_name, s));
                        WrappedData::Number(num.into())
                    } else {
                        panic!("Argument '{}' expected i16 number or string convertible to i16, got {:?}", arg_name, json_value);
                    }
                },
                ArgumentType::U16 => {
                    if let Some(num) = json_value.as_u64() {
                        WrappedData::Number(num.try_into().expect(&format!("Number out of range for u16: {}", num)))
                    } else if let Some(s) = json_value.as_str() {
                        let num: u16 = s.parse().expect(&format!("Argument '{}' expected u16 number or string convertible to u16, got string '{}'", arg_name, s));
                        WrappedData::Number(num.into())
                    } else {
                        panic!("Argument '{}' expected u16 number or string convertible to u16, got {:?}", arg_name, json_value);
                    }
                },
                ArgumentType::I32 => {
                    if let Some(num) = json_value.as_i64() {
                        WrappedData::Number(num.try_into().expect(&format!("Number out of range for i32: {}", num)))
                    } else if let Some(s) = json_value.as_str() {
                        let num: i32 = s.parse().expect(&format!("Argument '{}' expected i32 number or string convertible to i32, got string '{}'", arg_name, s));
                        WrappedData::Number(num.into())
                    } else {
                        panic!("Argument '{}' expected i32 number or string convertible to i32, got {:?}", arg_name, json_value);
                    }
                },
                ArgumentType::U32 => {
                    if let Some(num) = json_value.as_u64() {
                        WrappedData::Number(num.try_into().expect(&format!("Number out of range for u32: {}", num)))
                    } else if let Some(s) = json_value.as_str() {
                        let num: u32 = s.parse().expect(&format!("Argument '{}' expected u32 number or string convertible to u32, got string '{}'", arg_name, s));
                        WrappedData::Number(num.into())
                    } else {
                        panic!("Argument '{}' expected u32 number or string convertible to u32, got {:?}", arg_name, json_value);
                    }
                },
                ArgumentType::I64 => {
                    if let Some(num) = json_value.as_i64() {
                        WrappedData::Number(num.into()) // i64 is infallible from i64
                    } else if let Some(s) = json_value.as_str() {
                        let num: i64 = s.parse().expect(&format!("Argument '{}' expected i64 number or string convertible to i64, got string '{}'", arg_name, s));
                        WrappedData::Number(num.into())
                    } else {
                        panic!("Argument '{}' expected i64 number or string convertible to i64, got {:?}", arg_name, json_value);
                    }
                },
                ArgumentType::U64 => {
                    if let Some(num) = json_value.as_u64() {
                        WrappedData::Number(num.into()) // u64 is infallible from u64
                    } else if let Some(s) = json_value.as_str() {
                        let num: u64 = s.parse().expect(&format!("Argument '{}' expected u64 number or string convertible to u64, got string '{}'", arg_name, s));
                        WrappedData::Number(num.into())
                    } else {
                        panic!("Argument '{}' expected u64 number or string convertible to u64, got {:?}", arg_name, json_value);
                    }
                },
                ArgumentType::Text => {
                    let s = json_value.as_str().expect(&format!("Argument '{}' expected a string, got {:?}", arg_name, json_value));
                    WrappedData::Text(s.to_string())
                },
                ArgumentType::Boolean => {
                    if let Some(b) = json_value.as_bool() {
                        WrappedData::Boolean(b)
                    } else if let Some(s) = json_value.as_str() {
                        let b: bool = s.parse().expect(&format!("Argument '{}' expected boolean or string convertible to boolean, got string '{}'", arg_name, s));
                        WrappedData::Boolean(b)
                    } else {
                        panic!("Argument '{}' expected boolean or string convertible to boolean, got {:?}", arg_name, json_value);
                    }
                },
                ArgumentType::Float => {
                    if let Some(f) = json_value.as_f64() {
                        WrappedData::Float(f)
                    } else if let Some(s) = json_value.as_str() {
                        let f: f64 = s.parse().expect(&format!("Argument '{}' expected float or string convertible to float, got string '{}'", arg_name, s));
                        WrappedData::Float(f)
                    } else {
                        panic!("Argument '{}' expected float or string convertible to float, got {:?}", arg_name, json_value);
                    }
                },
            };
            parsed_args.insert(arg_name, wrapped_data);
        }

        parsed_args
    }

    /// Retrieves the structured definition of the tool.
    ///
    /// This definition includes the tool's name, description, and expected arguments,
    /// allowing AI models to understand how to use it.
    ///
    /// # Returns
    /// An `IAToolDefinition` instance describing the tool.
    fn get_description(&self) -> IAToolDefinition;
}

/// Defines the structure of an AI tool, including its name, description, and arguments.
///
/// This structure is used by AI models to understand the capabilities
/// and usage of a specific tool.
#[derive(Debug, Serialize, Deserialize)]
pub struct IAToolDefinition {
    /// The unique name of the tool.
    pub name: String,
    /// A detailed description of the tool's purpose.
    pub description: String,
    /// A `HashMap` describing the arguments the tool accepts,
    /// where the key is the argument name and the value is its definition.
    pub arguments: HashMap<String, IAArgument>,
}

/// Defines an individual argument for an AI tool.
///
/// Specifies the name, description, expected data type, and whether the argument is required.
#[derive(Debug, Serialize, Deserialize)]
pub struct IAArgument {
    /// The name of the argument.
    pub name: String,
    /// A description of what the argument represents.
    pub description: String,
    /// The type of data expected for the argument.
    pub arg_type: ArgumentType,
    /// Whether the argument is required or optional.
    pub required: bool,
}

/// Enumerates the primitive data types that can be used as arguments for AI tools.
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

/// An enum that wraps different data types that can be passed
/// as arguments to AI tools.
///
/// Provides a unified way to handle various data types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WrappedData {
    Number(WrappedInt),
    Text(String),
    Boolean(bool),
    Float(f64),
}

/// An enum that wraps different integer types.
///
/// Used within `WrappedData::Number` to represent integers of various sizes,
/// both signed and unsigned.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WrappedInt {
    I8(i8), U8(u8),
    I16(i16), U16(u16),
    I32(i32), U32(u32),
    I64(i64), U64(u64),
}

// --- From<PrimitiveType> for WrappedInt Implementations ---
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