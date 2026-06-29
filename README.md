# toli - A Rust Procedural Macro for Tool Generation

`toli` is a Rust library designed to simplify the creation of callable tools, particularly useful for integrating with Large Language Models (LLMs) or other systems that require structured function definitions and dynamic invocation. It provides a procedural macro `#[tool]` that automatically generates a `Tool` implementation for any annotated function, extracting its description and arguments directly from Rust's doc comments.

## Features

*   **Automatic Tool Definition:** Transform any Rust function into a structured tool definition with a simple attribute macro.
*   **Doc Comment Parsing:** Extracts function descriptions and argument details from standard Rust doc comments.
*   **Intelligent Description Generation:** Automatically generates default descriptions for functions and arguments if not explicitly provided, by formatting `snake_case` names into human-readable strings.
*   **Ignorable Doc Sections:** Automatically ignores common Rust doc sections like `# Examples`, `# Panics`, `# Errors`, and `# Safety` when extracting descriptions.
*   **Flexible Data Handling:** Uses `WrappedData` and `WrappedInt` enums to handle various primitive types (integers, strings, booleans, floats) for function arguments, enabling dynamic type-safe invocation.

## How to Add `toli` to Your Project

To use `toli` in your Rust project, add it as a dependency in your `Cargo.toml` file. Since `toli` re-exports its procedural macro, you only need to add `toli` itself.

```toml
[dependencies]
toli = "0.1.0"
```

## How to Use the `#[tool]` Macro

The `#[tool]` macro can be applied to any `fn` definition. It will automatically generate a unit struct named `[FunctionName]Tool` (e.g., `my_function` -> `MyFunctionTool`) that implements the `toli::IATool` trait.

### Example Function Annotation

Annotate your function with `#[tool]`. The function's documentation comments will be parsed to extract its description and argument details.

```rust
use toli::{IATool, WrappedData, WrappedInt, ArgumentType};
use std::collections::HashMap;
use toli::tool; // Import the macro from 'toli'

/// This is a comprehensive description of my awesome tool.
/// It performs a calculation based on two numbers and returns a string.
///
/// This description can span multiple lines.
///
/// Parameters:
/// - first_number: The initial integer value for the calculation.
/// - second_number: The second integer value to be added.
/// - operation_type: The type of operation to perform (e.g., "add", "subtract").
///
/// # Examples
/// ```
/// // Example usage of the tool
/// let tool_instance = MyAwesomeToolTool;
/// let mut args = HashMap::new();
/// args.insert("first_number".to_string(), WrappedData::Number(WrappedInt::I64(10)));
/// args.insert("second_number".to_string(), WrappedData::Number(WrappedInt::I64(5)));
/// args.insert("operation_type".to_string(), WrappedData::Text("add".to_string()));
/// let result = tool_instance.call(args);
/// assert_eq!(result, "Result: 15".to_string());
/// ```
///
/// # Panics
/// This function will panic if `operation_type` is not recognized.
///
/// # Errors
/// This function does not return explicit errors, but panics on invalid input.
///
/// # Safety
/// This function is safe to call.
#[tool]
fn my_awesome_tool(first_number: i64, second_number: i64, operation_type: String) -> String {
    match operation_type.as_str() {
        "add" => format!("Result: {}", first_number + second_number),
        "subtract" => format!("Result: {}", first_number - second_number),
        _ => panic!("Unsupported operation type: {}", operation_type),
    }
}

// You can then use the generated tool struct:
fn main() {
    let tool_instance = MyAwesomeToolTool; // The macro generates this struct

    // Get the tool's description
    let description = tool_instance.get_description();
    println!("Tool Name: {}", description.name);
    println!("Tool Description: {}", description.description);
    for (arg_name, arg_details) in description.arguments {
        println!(
            "  - {}: (Type: {:?}, Required: {}, Description: '{}')",
            arg_name, arg_details.arg_type, arg_details.required, arg_details.description
        );
    }

    // Call the tool
    let mut args = HashMap::new();
    args.insert("first_number".to_string(), WrappedData::Number(WrappedInt::I64(20)));
    args.insert("second_number".to_string(), WrappedData::Number(WrappedInt::I64(10)));
    args.insert("operation_type".to_string(), WrappedData::Text("subtract".to_string()));

    let result = tool_instance.call(args);
    if let s = result {
        println!("Tool Call Result: {}", s); // Output: "Tool Call Result: Result: 10"
    }
}
```

### Doc Comment Parsing Rules

The `#[tool]` macro parses doc comments (`///`) with the following rules:

*   **Function Description:** All lines before the `Parameters:` section (and not part of an ignorable section) are collected as the function's description. If no description is found, a default one is generated from the function's `snake_case` name (e.g., `my_awesome_tool` -> "My awesome tool").
*   **Argument Descriptions:**
    *   Argument descriptions are extracted from a dedicated `Parameters:` section.
    *   Each argument should be listed as: `- argument_name: Description of argument.`
    *   The `argument_name` must exactly match the function parameter's name.
    *   If an argument is listed in `Parameters:` but the function does not have that argument, it will be ignored.
    *   If a function parameter is not listed in the `Parameters:` section, a default description is generated from its `snake_case` name (e.g., `first_number` -> "First number").
*   **Ignored Sections:** The macro automatically ignores content within the following standard Rust doc comment sections:
    *   `# Examples`
    *   `# Panics`
    *   `# Errors`
    *   `# Safety`

These sections and their content will not be included in the extracted `description` field of the `IAToolDefinition`.

### Supported Argument

The macro supports the following types for function arguments, which are mapped to `toli::WrappedData` variants:

*   All integer primitives: `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64` (mapped to `WrappedData::Number(WrappedInt::...)`)
*   `String` (mapped to `WrappedData::Text`)
*   `bool` (mapped to `WrappedData::Boolean`)
*   `f64` (mapped to `WrappedData::Float`)

Conversions between `WrappedInt` and primitive integer types are handled automatically using `From` and `TryFrom` implementations provided by the `toli` crate, ensuring type safety and proper error handling for out-of-range conversions.

### Returning Custom Types

The macro supports functions returning user defined structs. To allow the macro returning this type the struct must be declared as public. 

```rust
    #[derive(Debug, PartialEq)]
    pub struct MyCustomResult {
        id: u32,
        message: String,
        is_success: bool,
    }

    /// A tool that generates a custom result struct.
    ///
    /// Parameters:
    /// - input_id: An identifier for the result.
    /// - input_message: A message to include in the result.
    /// - success_status: Whether the operation was successful.
    #[tool]
    fn get_custom_result(input_id: u32, input_message: String, success_status: bool) -> MyCustomResult {
        MyCustomResult {
            id: input_id,
            message: format!("Processed: {}", input_message),
            is_success: success_status,
        }
    }
```
