# toli - A Rust Procedural Macro for Tool Generation
[![Crates.io](https://img.shields.io/crates/v/toli)](https://crates.io/crates/toli)


`toli` is a Rust library designed to simplify the creation of callable tools, particularly useful for integrating with Large Language Models (LLMs) or other systems that require structured function definitions and dynamic invocation. It provides a procedural macro `#[tool]` that automatically generates a `Tool` implementation for any annotated function, extracting its description and arguments directly from Rust's doc comments.

## Features

*   **Automatic Tool Definition:** Transform any Rust function into a structured tool definition with a simple attribute macro.
*   **Doc Comment Parsing:** Extracts function descriptions and argument details from standard Rust doc comments.
*   **Intelligent Description Generation:** Automatically generates default descriptions for functions and arguments if not explicitly provided, by formatting `snake_case` names into human-readable strings (e.g., `add_values` -> "Add values").
*   **Ignorable Doc Sections:** Automatically ignores common Rust doc sections like `# Examples`, `# Panics`, `# Errors`, and `# Safety` when extracting descriptions.
*   **Robust JSON Argument Parsing:** The generated tools can receive arguments as a JSON string. The `parse_json_args` method converts JSON values to the expected Rust types, including attempting to parse string representations of numbers and booleans.
*   **Direct Return Types:** The `call` method of the generated tool directly returns the `OriginalReturnType` of the wrapped function, ensuring type safety and avoiding unnecessary conversions.

## How to Add `toli` to Your Project

To use `toli` in your Rust project, add it as a dependency in your `Cargo.toml` file.

```toml
[dependencies]
toli = "0.1.0"
```

## How to Use the `#[tool]` Macro

The `#[tool]` macro can be applied to any `fn` definition. It will automatically generate a unit struct named `[FunctionName]Tool` (e.g., `my_function` -> `MyFunctionTool`) that implements the `toli::IATool` trait.

### Function Annotation Example

Annotate your function with `#[tool]`. The function's documentation comments will be parsed to extract its description and argument details.

```rust
use toli::tool;
use toli::IATool;

/// This is a comprehensive description of my awesome tool.
/// It performs a calculation based on two numbers and returns a string.
///
/// This description can span multiple lines.
///
/// Parameters:
/// - first_number: The initial integer value for the calculation.
/// - second_number: The second integer value to be added.
/// - operation_type: The type of operation to perform (e.g., "add", "subtract").
/// - enable_logging: Whether to enable verbose logging for this operation.
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
fn my_awesome_tool(
    first_number: i64,
    second_number: i64,
    operation_type: String,
    enable_logging: bool,
) -> String {
    let log_status = if enable_logging { "with logging" } else { "without logging" };
    match operation_type.as_str() {
        "add" => format!("Result: {} {} ({})", first_number + second_number, log_status, operation_type),
        "subtract" => format!("Result: {} {} ({})", first_number - second_number, log_status, operation_type),
        _ => panic!("Unsupported operation type: {}", operation_type),
    }
}

// Define a custom struct that can be returned by a tool
#[derive(Debug, PartialEq)]
pub struct CalculationSummary {
    pub operation: String,
    pub operands: (i64, i64),
    pub result: i64,
    pub logged: bool,
}

/// A tool that performs an operation and returns a structured summary.
///
/// Parameters:
/// - val1: The first operand.
/// - val2: The second operand.
/// - op: The operation to perform (e.g., "multiply", "divide").
/// - log: Whether the operation was logged.
#[tool]
fn perform_calculation(val1: i64, val2: i64, op: String, log: bool) -> CalculationSummary {
    let res = match op.as_str() {
        "multiply" => val1 * val2,
        "divide" => val1 / val2,
        _ => panic!("Unsupported calculation operation: {}", op),
    };
    CalculationSummary {
        operation: op,
        operands: (val1, val2),
        result: res,
        logged: log,
    }
}


fn main() {
    // --- Example 1: Using my_awesome_tool ---
    let awesome_tool_instance = MyAwesomeToolTool; // The macro generates this struct

    // Get the tool's description
    let description = awesome_tool_instance.get_description();
    println!("--- Tool Definition: {} ---", description.name);
    println!("Description: {}", description.description);
    for (arg_name, arg_details) in description.arguments {
        println!(
            "  - {}: (Type: {:?}, Required: {}, Description: '{}')",
            arg_name, arg_details.arg_type, arg_details.required, arg_details.description
        );
    }

    // Call the tool, providing arguments as a JSON string
    let json_args_string = "{ \
        \"first_number\": 20, \
        \"second_number\": \"10\", \
        \"operation_type\": \"subtract\", \
        \"enable_logging\": \"true\" \
    }".to_string();

    let result: String = awesome_tool_instance.call(json_args_string);
    println!("Tool Call Result (my_awesome_tool): {}", result);
    // Expected Output: "Tool Call Result (my_awesome_tool): Result: 10 with logging (subtract)"


    // --- Example 2: Using perform_calculation with custom struct return ---
    let calc_tool_instance = PerformCalculationTool;

    let json_calc_args = "{ \
        \"val1\": 50, \
        \"val2\": 5, \
        \"op\": \"divide\", \
        \"log\": false \
    }".to_string();

    let calc_summary: CalculationSummary = calc_tool_instance.call(json_calc_args);
    println!("Tool Call Result (perform_calculation): {:?}", calc_summary);
    // Expected Output: "Tool Call Result (perform_calculation): CalculationSummary { operation: "divide", operands: (50, 5), result: 10, logged: false }"
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

The macro supports the following types for function arguments.

*   All integer primitives: `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64` (arguments are mapped from `WrappedData::Number(WrappedInt::...)`)
*   `String` (arguments are mapped from `WrappedData::Text`)
*   `bool` (arguments are mapped from `WrappedData::Boolean`)
*   `f64` (arguments are mapped from `WrappedData::Float`)

Conversions between `WrappedInt` and primitive integer types are handled automatically using `From` and `TryFrom` implementations provided by the `toli` crate, ensuring type safety and proper error handling for out-of-range conversions when extracting arguments. Additionally, `parse_json_args` will attempt to parse string representations of numbers and booleans if the JSON value is a string.

### Function Returns

The `IATool::call` method will return the exact type specified by the original function's signature including any user-defined `struct` (as long as it's declared public).

## License

This project is licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.