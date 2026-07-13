# toli - A Rust Procedural Macro for Tool Generation
[![Crates.io](https://img.shields.io/crates/v/toli)](https://crates.io/crates/toli)


`toli` is a Rust library designed to simplify the creation of callable tools, particularly useful for integrating with Large Language Models (LLMs) or other systems that require structured function definitions and dynamic invocation. It provides procedural macros `#[tool]` and `#[async_tool]` that automatically generate `Tool` implementations for any annotated function, extracting its description and arguments directly from Rust's doc comments.

## Features

*   **Automatic Tool Definition:** Transform any Rust function (sync or async) into a structured tool definition with a simple attribute macro.
*   **Doc Comment Parsing:** Extracts function descriptions and argument details from standard Rust doc comments.
*   **Intelligent Description Generation:** Automatically generates default descriptions for functions and arguments if not explicitly provided, by formatting `snake_case` names into human-readable strings (e.g., `add_values` -> "Add values").
*   **Ignorable Doc Sections:** Automatically ignores common Rust doc sections like `# Examples`, `# Panics`, `# Errors`, and `# Safety` when extracting descriptions.
*   **Robust JSON Argument Parsing:** The generated tools can receive arguments as a JSON string. The `parse_json_args` method converts JSON values to the expected Rust types, including attempting to parse string representations of numbers and booleans. It also correctly handles optional arguments.
*   **Direct Return Types:** The `call` method of the generated tool directly returns the `OriginalReturnType` of the wrapped function, ensuring type safety and avoiding unnecessary conversions.

## How to Add `toli` to Your Project

To use `toli` in your Rust project, add it as a dependency in your `Cargo.toml` file.

```toml
[dependencies]
toli = "0.3.1"
```

## How to Use the `#[tool]` Macro (Synchronous Functions)

The `#[tool]` macro can be applied to any `fn` definition. It will automatically generate a unit struct named `[FunctionName]Tool` (e.g., `my_function` -> `MyFunctionTool`) that implements the `toli::IATool` trait.

### Function Annotation Example (Synchronous)

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
```

## How to Use the `#[async_tool]` Macro (Asynchronous Functions)

The `#[async_tool]` macro is designed for `async fn` definitions. It will automatically generate a unit struct named `[FunctionName]Tool` (e.g., `fetch_data` -> `FetchDataTool`) that implements the `toli::IAAsyncTool` trait.

### Function Annotation Example (Asynchronous)

Annotate your `async fn` with `#[async_tool]`. The function's documentation comments will be parsed to extract its description and argument details. You will need an async runtime (like `tokio`) to execute these tools.

```rust
use toli::{async_tool, IAAsyncTool};

/// An async tool that simulates fetching data from a remote source.
///
/// Parameters:
/// - query: The search query to use for fetching data.
/// - delay_ms: An optional delay in milliseconds to simulate network latency.
#[async_tool]
pub async fn fetch_remote_data(query: String, delay_ms: Option<u64>) -> String {
    if let Some(delay) = delay_ms {
        // In a real application, you'd use a proper async runtime's sleep
        // For example, with tokio: tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        // For this example, we'll just acknowledge the delay.
        println!("Simulating a {}ms delay for query: '{}'", delay, query);
    }
    format!("Fetched data for '{}' after {}ms delay.", query, delay_ms.unwrap_or(0))
}

#[tokio::main] // Requires tokio runtime for execution
async fn main() {
    let async_tool_instance = FetchRemoteDataTool; // The macro generates this struct

    // Get the tool's description
    let description = async_tool_instance.get_description();
    println!("--- Async Tool Definition: {} ---", description.name);
    println!("Description: {}", description.description);
    for (arg_name, arg_details) in description.arguments {
        println!(
            "  - {}: (Type: {:?}, Required: {}, Description: '{}')",
            arg_name, arg_details.arg_type, arg_details.required, arg_details.description
        );
    }

    // Call the async tool
    let json_args_string = r#"{"query": "Rust async programming", "delay_ms": 200}"#.to_string();
    let result: String = async_tool_instance.call(json_args_string).await;
    println!("Async Tool Call Result: {}", result);
    // Expected Output: "Async Tool Call Result: Fetched data for 'Rust async programming' after 200ms delay."
}
```

### Doc Comment Parsing Rules

The `#[tool]` and `#[async_tool]` macros parse doc comments (`///`) with the following rules:

*   **Function Description:** All lines before the `Parameters:` section (and not part of an ignorable section) are collected as the function's description. If no description is found, a default one is generated from the function's `snake_case` name (e.g., `my_awesome_tool` -> "My awesome tool").
*   **Argument Descriptions:**
    *   Argument descriptions are extracted from a dedicated `Parameters:` section.
    *   Each argument should be listed as: `- argument_name: Description of argument.`
    *   The `argument_name` must exactly match the function parameter's name.
    *   If an argument is listed in `Parameters:` but the function does not have that argument, it will be ignored.
    *   If a function parameter is not listed in the `Parameters:` section, a default description is generated from its `snake_case` name (e.g., `first_number` -> "First number").
*   **Ignored Sections:** The macros automatically ignore content within the following standard Rust doc comment sections:
    *   `# Examples`
    *   `# Panics`
    *   `# Errors`
    *   `# Safety`

These sections and their content will not be included in the extracted `description` field of the `IAToolDefinition`.

### Supported Argument Types

The macros support the following types for function arguments:

*   All integer primitives: `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64` (arguments are mapped from `WrappedData::Number(WrappedInt::...)`)
*   `String` (arguments are mapped from `WrappedData::Text`)
*   `bool` (arguments are mapped from `WrappedData::Boolean`)
*   `f64` (arguments are mapped from `WrappedData::Float`)
*   **Optional Arguments**: `Option<T>`, where `T` is one of the supported types listed above. If an `Option<T>` argument is missing from the JSON input or its value is `null`, `None` will be passed to the function.

Conversions between `WrappedInt` and primitive integer types are handled automatically using `From` and `TryFrom` implementations provided by the `toli` crate, ensuring type safety and proper error handling for out-of-range conversions when extracting arguments. Additionally, `parse_json_args` will attempt to parse string representations of numbers and booleans if the JSON value is a string.

### Function Returns

The `IATool::call` and `IAAsyncTool::call` methods will return the exact type specified by the original function's signature including any user-defined `struct` (as long as it's declared public).

## Using the `ToolSet` for Dynamic Tool Management

The `ToolSet` struct provides a way to manage and dispatch multiple synchronous and asynchronous tools dynamically. It's particularly useful when you need to select and execute tools based on their names at runtime, for example, when integrating with an LLM that decides which tool to call.

### 1. Define a Descriptor Type

`ToolSet` is generic over a descriptor type `D` which must implement `IADescriptor`. This trait allows `ToolSet` to store tool definitions in a format suitable for your application. Implementing this trait your struct can follow the scheme for the LLM or application that have to use the tool. 

```rust
use toli::{IAToolDefinition, ToolSet};

// For simplicity, we'll use IAToolDefinition as our descriptor type.
// You can define your own struct that implements IADescriptor if you need
// to store additional metadata with your tool definitions.
type MyToolSet = ToolSet<IAToolDefinition>;

// Create a new ToolSet instance
let mut tool_set = MyToolSet::new();
```

### 2. Add Synchronous Tools

Use the `add_tool` method to register synchronous tools generated by the `#[tool]` macro.

```rust
use toli::{tool, IATool};
// ... (my_awesome_tool definition from above) ...

// Add the synchronous tool to the ToolSet
tool_set.add_tool(MyAwesomeTool); // MyAwesomeTool is the generated struct
```

### 3. Add Asynchronous Tools

Use the `add_async_tool` method to register asynchronous tools generated by the `#[async_tool]` macro.

```rust
use toli::{async_tool, IAAsyncTool};
// ... (fetch_remote_data definition from above) ...

// Add the asynchronous tool to the ToolSet
tool_set.add_async_tool(FetchRemoteDataTool); // FetchRemoteDataTool is the generated struct
```

### 4. Dispatch Tools

The `dispatch` method allows you to call a tool by its name, passing arguments as a `serde_json::Value`. It automatically handles whether the tool is synchronous or asynchronous.

```rust
use serde_json::json;

#[tokio::main] // Required for dispatching async tools
async fn main() {
    // ... (tool_set creation and adding tools) ...

    // Dispatch a synchronous tool
    let sync_args = json!({
        "first_number": 10,
        "second_number": 5,
        "operation_type": "add",
        "enable_logging": true
    });
    let sync_result = tool_set.dispatch("my_awesome_tool".to_string(), sync_args).await;
    println!("Synchronous tool result: {:?}", sync_result);
    // Expected: Ok("Result: 15 with logging (add)")

    // Dispatch an asynchronous tool
    let async_args = json!({
        "query": "Rust programming",
        "delay_ms": 100
    });
    let async_result = tool_set.dispatch("fetch_remote_data".to_string(), async_args).await;
    println!("Asynchronous tool result: {:?}", async_result);
    // Expected: Ok("Fetched data for 'Rust programming' after 100ms delay.")

    // Attempt to dispatch a non-existent tool
    let non_existent_result = tool_set.dispatch("non_existent_tool".to_string(), json!({})).await;
    println!("Non-existent tool result: {:?}", non_existent_result);
    // Expected: Err("{ \"error\": \"Tool not found or cannot be dispatched\"}")
}
```

### 5. Retrieve Tool Descriptors

You can get a list of all registered tool descriptors using the `get_tools` method. This is useful for providing an LLM with a list of available tools and their definitions.

```rust
// ... (tool_set creation and adding tools) ...

let available_tools = tool_set.get_tools();
println!("\n--- Available Tools ---");
for tool_def in available_tools {
    println!("Name: {}", tool_def.name);
    println!("Description: {}", tool_def.description);
    println!("Arguments: {:?}", tool_def.arguments);
    println!("-----------------------");
}
```

## License

This project is licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.