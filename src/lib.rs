//! # Toli
//!
//! `toli` is a library that provides the necessary abstractions to define and manage tools
//! that can be used by Artificial Intelligence (AI) language models.
//! It facilitates the creation of tools with typed arguments and descriptions,
//! allowing AI models to interact with external functions in a structured manner.

pub mod tool_set;
pub mod open_ai;

use serde_json::Value;
use std::collections::HashMap;
use std::convert::{From, TryFrom};

/// Re-exports the `async_trait` macro for defining async traits.
#[doc(hidden)]
pub use async_trait::async_trait;
/// Re-exports the `tool` macro for defining synchronous AI tools.
pub use macro_toli::tool;
/// Re-exports the `async_tool` macro for defining asynchronous AI tools.
pub use macro_toli::async_tool;
/// Re-exports `ToolSet` for managing collections of AI tools and `IADescriptor` for tool descriptions.
pub use crate::tool_set::{ToolSet, IADescriptor};

/// Trait that defines the interface for an AI tool.
///
/// Any type implementing this trait can be considered a tool
/// that an AI model can invoke.
pub trait IATool {
    /// The actual return type of the function wrapped by this tool.
    type OriginalReturnType;

    /// Executes the tool's logic with the provided arguments.
    ///
    /// The arguments are provided as a JSON `String`, typically representing
    /// a JSON object where keys are argument names and values are their corresponding data.
    /// The implementation will parse this JSON into a `HashMap<String, WrappedData>`
    /// before invoking the original function.
    ///
    /// # Arguments
    /// * `json_string_args` - A JSON `String` containing the arguments for the tool.
    ///
    /// # Returns
    /// The direct result of the tool's execution, which is `Self::OriginalReturnType`.
    /// No further conversion is performed on the return value by the `IATool` trait itself.
    ///
    /// # Panics
    /// If the `json_string_args` cannot be parsed into the expected `HashMap<String, WrappedData>`
    /// or if argument type conversions fail.
    fn call(&self, json_string_args: String) -> Self::OriginalReturnType;

    /// Converts a JSON `String` into a `HashMap<String, WrappedData>`.
    ///
    /// This function is used internally by the `call` method to prepare arguments
    /// for the original function. It validates and converts JSON values
    /// into the `WrappedData` enum based on the tool's argument definitions.
    ///
    /// For optional arguments (where `IAArgument.required` is `false`), if the argument
    /// is missing from the JSON input or its value is `null`, `WrappedData::None` will be
    /// inserted into the map. For required arguments, missing or `null` values will cause a panic.
    ///
    /// # Arguments
    /// * `json_string_args` - The JSON `String` to parse.
    ///
    /// # Returns
    /// A `HashMap<String, WrappedData>` containing the parsed arguments.
    ///
    /// # Panics
    /// Panics if the JSON structure does not match the expected argument types
    /// or if required arguments are missing or `null`.
    fn parse_json_args(&self, json_string_args: String) -> HashMap<String, WrappedData> {
        parse_json_args_internal(self.get_description(), json_string_args)
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

/// Trait that defines the interface for an asynchronous AI tool.
///
/// Any type implementing this trait can be considered an async tool
/// that an AI model can invoke.
#[async_trait]
pub trait IAAsyncTool {
    /// The actual return type of the async function wrapped by this tool.
    type OriginalReturnType: Send;

    /// Executes the async tool's logic with the provided arguments.
    ///
    /// The arguments are provided as a JSON `String`, typically representing
    /// a JSON object where keys are argument names and values are their corresponding data.
    /// The implementation will parse this JSON into a `HashMap<String, WrappedData>`
    /// before invoking the original async function.
    ///
    /// # Arguments
    /// * `json_string_args` - A JSON `String` containing the arguments for the tool.
    ///
    /// # Returns
    /// The direct result of the tool's asynchronous execution, which is `Self::OriginalReturnType`.
    ///
    /// # Panics
    /// If the `json_string_args` cannot be parsed into the expected `HashMap<String, WrappedData>`
    /// or if argument type conversions fail.
    async fn call(&self, json_string_args: String) -> Self::OriginalReturnType;

    /// Converts a JSON `String` into a `HashMap<String, WrappedData>`.
    ///
    /// This function is used internally by the `call` method to prepare arguments
    /// for the original function. It validates and converts JSON values
    /// into the `WrappedData` enum based on the tool's argument definitions.
    ///
    /// For optional arguments (where `IAArgument.required` is `false`), if the argument
    /// is missing from the JSON input or its value is `null`, `WrappedData::None` will be
    /// inserted into the map. For required arguments, missing or `null` values will cause a panic.
    ///
    /// # Arguments
    /// * `json_string_args` - The JSON `String` to parse.
    ///
    /// # Returns
    /// A `HashMap<String, WrappedData>` containing the parsed arguments.
    ///
    /// # Panics
    /// Panics if the JSON structure does not match the expected argument types
    /// or if required arguments are missing or `null`.
    fn parse_json_args(&self, json_string_args: String) -> HashMap<String, WrappedData> {
        parse_json_args_internal(self.get_description(), json_string_args)
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

/// Internal helper function to parse JSON arguments for both synchronous and asynchronous tools.
///
/// This function takes the tool's definition and a JSON string of arguments,
/// then parses and validates them into a `HashMap<String, WrappedData>`.
/// It handles required and optional arguments, panicking if required arguments are missing or null,
/// or if type conversions fail.
///
/// # Arguments
/// * `tool_description` - The `IAToolDefinition` of the tool.
/// * `json_string_args` - The JSON `String` containing the arguments.
///
/// # Returns
/// A `HashMap<String, WrappedData>` containing the parsed arguments.
///
/// # Panics
/// Panics if the JSON string is invalid, if the JSON structure is not an object,
/// if required arguments are missing or null, or if argument type conversions fail.
fn parse_json_args_internal(tool_description: IAToolDefinition, json_string_args: String) -> HashMap<String, WrappedData> {
    let mut parsed_args = HashMap::new();

    let json_value: Value = serde_json::from_str(&json_string_args)
        .expect("Failed to parse JSON string for tool arguments.");

    let json_obj = json_value.as_object()
        .expect("JSON input for tool arguments must be an object.");

    for (arg_name, arg_def) in tool_description.arguments {
        let json_arg_value = json_obj.get(&arg_name);

        let wrapped_data = if arg_def.required {
            let value = json_arg_value
                .expect(&format!("Missing required argument '{}' in JSON input.", arg_name));
            if value.is_null() {
                panic!("Required argument '{}' cannot be null.", arg_name);
            }
            parse_single_arg(&arg_name, &arg_def.arg_type, value)
        } else {
            match json_arg_value {
                Some(value) if !value.is_null() => parse_single_arg(&arg_name, &arg_def.arg_type, value),
                _ => WrappedData::None, // Argument is optional and missing or null
            }
        };
        parsed_args.insert(arg_name, wrapped_data);
    }

    parsed_args
}

/// Parses a single JSON `Value` into a `WrappedData` enum based on the expected `ArgumentType`.
///
/// This helper function is used by `parse_json_args_internal` to convert individual argument
/// values from their JSON representation into the internal `WrappedData` format.
/// It handles various primitive types and vectors, performing necessary type conversions
/// and range checks for integer types.
///
/// # Arguments
/// * `arg_name` - The name of the argument being parsed (used for error messages).
/// * `arg_type` - The expected `ArgumentType` of the argument.
/// * `json_value` - The `serde_json::Value` representing the argument's value.
///
/// # Returns
/// A `WrappedData` enum containing the parsed value.
///
/// # Panics
/// Panics if the `json_value` cannot be converted to the specified `arg_type`,
/// if a number is out of range for its target integer type, or if a string
/// cannot be parsed into the target numeric or boolean type.
fn parse_single_arg(arg_name: &str, arg_type: &ArgumentType, json_value: &Value) -> WrappedData {
    match arg_type {
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
        ArgumentType::Vec(inner_type) => {
            let arr = json_value.as_array().expect(&format!("Argument '{}' expected an array, got {:?}", arg_name, json_value));
            let parsed_vec: Vec<WrappedData> = arr.iter()
                .map(|item| parse_single_arg(arg_name, inner_type, item))
                .collect();
            WrappedData::Vec(parsed_vec)
        }
    }
}

/// Defines the structured metadata for an AI tool.
///
/// This structure provides essential information about a tool, including its unique identifier,
/// a human-readable description, and a detailed specification of its arguments.
/// AI models use this definition to understand the tool's capabilities and how to invoke it correctly.
#[derive(Debug)]
pub struct IAToolDefinition {
    /// The unique name of the tool.
    pub name: String,
    /// A detailed description of the tool's purpose.
    pub description: String,
    /// A `HashMap` describing the arguments the tool accepts,
    /// where the key is the argument name and the value is its definition.
    pub arguments: HashMap<String, IAArgument>,
}

/// Defines the properties of an individual argument for an AI tool.
///
/// This structure specifies the `name`, `description`, expected `arg_type`,
/// and `required` status of a single argument.
/// It helps AI models understand what data to provide for each argument when calling a tool.
#[derive(Debug)]
pub struct IAArgument {
    /// The name of the argument.
    pub name: String,
    /// A description of what the argument represents.
    pub description: String,
    /// The type of data expected for the argument.
    pub arg_type: ArgumentType,
    /// Whether the argument is required (`true`) or optional (`false`).
    /// Optional arguments can be omitted from the JSON input or provided as `null`.
    pub required: bool,
}

/// Enumerates the supported data types for AI tool arguments.
///
/// This enum represents the various primitive types (integers, text, boolean, float)
/// and a vector type (`Vec`) that AI tools can accept as input.
/// The `Vec` variant allows for specifying homogeneous lists of other `ArgumentType`s.
#[derive(Debug, PartialEq)]
pub enum ArgumentType {
    I8, U8,
    I16, U16,
    I32, U32,
    I64, U64,
    Text,
    Boolean,
    Float,
    Vec(Box<ArgumentType>),
}

/// An enum that wraps different data types that can be passed
/// as arguments to AI tools.
///
/// Provides a unified way to handle various data types, including a `None` variant
/// for optional arguments that are missing or `null` in the JSON input.
#[derive(Debug, Clone)]
pub enum WrappedData {
    Number(WrappedInt),
    Text(String),
    Boolean(bool),
    Float(f64),
    Vec(Vec<WrappedData>),
    None, // Added for optional arguments
}

/// An enum that wraps different integer types.
///
/// Used within `WrappedData::Number` to represent integers of various sizes,
/// both signed and unsigned.
#[derive(Debug, Clone)]
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

// --- TryFrom<WrappedData> for PrimitiveType Implementations ---
impl TryFrom<WrappedData> for String {
    type Error = &'static str;
    fn try_from(wrapped: WrappedData) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedData::Text(s) => Ok(s),
            _ => Err("WrappedData is not a Text"),
        }
    }
}

impl TryFrom<WrappedData> for bool {
    type Error = &'static str;
    fn try_from(wrapped: WrappedData) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedData::Boolean(b) => Ok(b),
            _ => Err("WrappedData is not a Boolean"),
        }
    }
}

impl TryFrom<WrappedData> for f64 {
    type Error = &'static str;
    fn try_from(wrapped: WrappedData) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedData::Float(f) => Ok(f),
            _ => Err("WrappedData is not a Float"),
        }
    }
}

// --- TryFrom<WrappedData> for Vec<PrimitiveType> Implementations ---
impl TryFrom<WrappedData> for Vec<i8> {
    type Error = &'static str;
    fn try_from(wrapped: WrappedData) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedData::Vec(vec_wrapped_data) => {
                vec_wrapped_data.into_iter().map(|wd| {
                    if let WrappedData::Number(wi) = wd {
                        i8::try_from(wi)
                    } else {
                        Err("Element in WrappedData::Vec is not a Number for Vec<i8>")
                    }
                }).collect()
            },
            _ => Err("WrappedData is not a Vec for Vec<i8>"),
        }
    }
}

impl TryFrom<WrappedData> for Vec<u8> {
    type Error = &'static str;
    fn try_from(wrapped: WrappedData) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedData::Vec(vec_wrapped_data) => {
                vec_wrapped_data.into_iter().map(|wd| {
                    if let WrappedData::Number(wi) = wd {
                        u8::try_from(wi)
                    } else {
                        Err("Element in WrappedData::Vec is not a Number for Vec<u8>")
                    }
                }).collect()
            },
            _ => Err("WrappedData is not a Vec for Vec<u8>"),
        }
    }
}

impl TryFrom<WrappedData> for Vec<i16> {
    type Error = &'static str;
    fn try_from(wrapped: WrappedData) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedData::Vec(vec_wrapped_data) => {
                vec_wrapped_data.into_iter().map(|wd| {
                    if let WrappedData::Number(wi) = wd {
                        i16::try_from(wi)
                    } else {
                        Err("Element in WrappedData::Vec is not a Number for Vec<i16>")
                    }
                }).collect()
            },
            _ => Err("WrappedData is not a Vec for Vec<i16>"),
        }
    }
}

impl TryFrom<WrappedData> for Vec<u16> {
    type Error = &'static str;
    fn try_from(wrapped: WrappedData) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedData::Vec(vec_wrapped_data) => {
                vec_wrapped_data.into_iter().map(|wd| {
                    if let WrappedData::Number(wi) = wd {
                        u16::try_from(wi)
                    } else {
                        Err("Element in WrappedData::Vec is not a Number for Vec<u16>")
                    }
                }).collect()
            },
            _ => Err("WrappedData is not a Vec for Vec<u16>"),
        }
    }
}

impl TryFrom<WrappedData> for Vec<i32> {
    type Error = &'static str;
    fn try_from(wrapped: WrappedData) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedData::Vec(vec_wrapped_data) => {
                vec_wrapped_data.into_iter().map(|wd| {
                    if let WrappedData::Number(wi) = wd {
                        i32::try_from(wi)
                    } else {
                        Err("Element in WrappedData::Vec is not a Number for Vec<i32>")
                    }
                }).collect()
            },
            _ => Err("WrappedData is not a Vec for Vec<i32>"),
        }
    }
}

impl TryFrom<WrappedData> for Vec<u32> {
    type Error = &'static str;
    fn try_from(wrapped: WrappedData) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedData::Vec(vec_wrapped_data) => {
                vec_wrapped_data.into_iter().map(|wd| {
                    if let WrappedData::Number(wi) = wd {
                        u32::try_from(wi)
                    } else {
                        Err("Element in WrappedData::Vec is not a Number for Vec<u32>")
                    }
                }).collect()
            },
            _ => Err("WrappedData is not a Vec for Vec<u32>"),
        }
    }
}

impl TryFrom<WrappedData> for Vec<i64> {
    type Error = &'static str;
    fn try_from(wrapped: WrappedData) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedData::Vec(vec_wrapped_data) => {
                vec_wrapped_data.into_iter().map(|wd| {
                    if let WrappedData::Number(wi) = wd {
                        i64::try_from(wi)
                    } else {
                        Err("Element in WrappedData::Vec is not a Number for Vec<i64>")
                    }
                }).collect()
            },
            _ => Err("WrappedData is not a Vec for Vec<i64>"),
        }
    }
}

impl TryFrom<WrappedData> for Vec<u64> {
    type Error = &'static str;
    fn try_from(wrapped: WrappedData) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedData::Vec(vec_wrapped_data) => {
                vec_wrapped_data.into_iter().map(|wd| {
                    if let WrappedData::Number(wi) = wd {
                        u64::try_from(wi)
                    } else {
                        Err("Element in WrappedData::Vec is not a Number for Vec<u64>")
                    }
                }).collect()
            },
            _ => Err("WrappedData is not a Vec for Vec<u64>"),
        }
    }
}

impl TryFrom<WrappedData> for Vec<String> {
    type Error = &'static str;
    fn try_from(wrapped: WrappedData) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedData::Vec(vec_wrapped_data) => {
                vec_wrapped_data.into_iter().map(|wd| {
                    if let WrappedData::Text(s) = wd {
                        Ok(s)
                    } else {
                        Err("Element in WrappedData::Vec is not a Text for Vec<String>")
                    }
                }).collect()
            },
            _ => Err("WrappedData is not a Vec for Vec<String>"),
        }
    }
}

impl TryFrom<WrappedData> for Vec<bool> {
    type Error = &'static str;
    fn try_from(wrapped: WrappedData) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedData::Vec(vec_wrapped_data) => {
                vec_wrapped_data.into_iter().map(|wd| {
                    if let WrappedData::Boolean(b) = wd {
                        Ok(b)
                    } else {
                        Err("Element in WrappedData::Vec is not a Boolean for Vec<bool>")
                    }
                }).collect()
            },
            _ => Err("WrappedData is not a Vec for Vec<bool>"),
        }
    }
}

impl TryFrom<WrappedData> for Vec<f64> {
    type Error = &'static str;
    fn try_from(wrapped: WrappedData) -> Result<Self, Self::Error> {
        match wrapped {
            WrappedData::Vec(vec_wrapped_data) => {
                vec_wrapped_data.into_iter().map(|wd| {
                    if let WrappedData::Float(f) = wd {
                        Ok(f)
                    } else {
                        Err("Element in WrappedData::Vec is not a Float for Vec<f64>")
                    }
                }).collect()
            },
            _ => Err("WrappedData is not a Vec for Vec<f64>"),
        }
    }
}