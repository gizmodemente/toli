//! # macro_toli
//!
//! `macro_toli` is a procedural macro crate that provides the `#[tool]` attribute macro.
//! This macro simplifies the process of defining AI tools by automatically generating
//! the necessary `IATool` implementation for a given Rust function.
//!
//! It handles:
//! - Parsing function signatures to determine tool arguments and their types.
//! - Extracting documentation comments to use as tool and argument descriptions.
//! - Generating a struct that implements the `toli::IATool` trait.
//! - Managing the conversion between a JSON `String` and native Rust types
//!   for function calls and return values.
//!
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemFn, FnArg, Pat, Type, ReturnType, LitStr, Expr, Lit, Meta};
use proc_macro::TokenStream;
use std::collections::HashMap;

// Helper function to format snake_case to "Capitalized words with spaces"
fn format_name_for_description(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + {
            c.map(|l| if l == '_' {
                ' '
            } else {
                l
            })
        }.collect::<String>().as_str(),
    }
}

/// Generates the UpperCamelCase struct name from a snake_case function name.
fn generate_tool_struct_name(fn_name: &syn::Ident) -> syn::Ident {
    let tool_struct_name_str = fn_name.to_string()
        .split('_')
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<String>();
    format_ident!("{}Tool", tool_struct_name_str)
}

/// Parses function documentation comments to extract the function description and argument descriptions.
fn parse_function_docs(input_fn: &ItemFn, fn_name: &syn::Ident) -> (LitStr, HashMap<String, String>) {
    let mut full_doc_comment = String::new();
    for attr in &input_fn.attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(nv) = &attr.meta {
                if let Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(lit_str) = &expr_lit.lit {
                        full_doc_comment.push_str(&lit_str.value());
                        full_doc_comment.push('\n');
                    }
                }
            }
        }
    }

    let mut fn_description_lines = Vec::new();
    let mut arg_descriptions_map: HashMap<String, String> = HashMap::new();

    let ignorable_headers = [
        "# Examples",
        "# Panics",
        "# Errors",
        "# Safety",
    ];

    let mut current_section_is_ignorable = false;
    let mut in_parameters_section = false;

    for line in full_doc_comment.lines() {
        let trimmed_line = line.trim();

        if ignorable_headers.iter().any(|&header| trimmed_line.starts_with(header)) {
            current_section_is_ignorable = true;
            in_parameters_section = false;
            continue;
        }

        if trimmed_line.starts_with("Parameters:") {
            in_parameters_section = true;
            current_section_is_ignorable = false;
            continue;
        }

        if current_section_is_ignorable {
            continue;
        }

        if in_parameters_section {
            if let Some(rest_after_dash) = trimmed_line.strip_prefix("- ") {
                if let Some(colon_idx) = rest_after_dash.find(':') {
                    let arg_name_str = rest_after_dash[..colon_idx].trim();
                    let arg_desc_str = rest_after_dash[colon_idx + 1..].trim();
                    arg_descriptions_map.insert(arg_name_str.to_string(), arg_desc_str.to_string());
                }
            }
        } else {
            if !trimmed_line.is_empty() {
                fn_description_lines.push(trimmed_line.to_string());
            }
        }
    }

    let fn_description = fn_description_lines.join(" ");
    let fn_description = if fn_description.is_empty() {
        format_name_for_description(&fn_name.to_string())
    } else {
        fn_description
    };
    let fn_description_literal = LitStr::new(&fn_description, proc_macro2::Span::call_site());

    (fn_description_literal, arg_descriptions_map)
}

/// Processes function arguments to generate code for argument extraction and tool definition.
fn process_function_arguments(
    input_fn: &ItemFn,
    arg_descriptions_map: &HashMap<String, String>,
    macro_name: &str, // Added macro_name for better error messages
) -> Result<(proc_macro2::TokenStream, proc_macro2::TokenStream, proc_macro2::TokenStream), TokenStream> {
    let mut args_map_creation = quote! {};
    let mut call_args = quote! {};
    let mut generated_arg_inserts = quote! {};

    for arg in &input_fn.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            let arg_name = if let Pat::Ident(pat_ident) = &*pat_type.pat {
                &pat_ident.ident
            } else {
                return Err(syn::Error::new_spanned(pat_type, "Unsupported argument pattern").to_compile_error().into());
            };
            let original_arg_type = &pat_type.ty;
            let mut is_optional = false;
            let mut is_vec = false;
            let mut type_for_arg_type_enum: &Type = original_arg_type; // This will be the innermost type for ArgumentType enum
            let mut type_for_conversion: &Type = original_arg_type; // This is the type WrappedData converts into

            // Check for Option<T>
            if let Type::Path(type_path) = &**original_arg_type {
                if type_path.path.segments.len() == 1 {
                    let segment = &type_path.path.segments[0];
                    if segment.ident == "Option" {
                        is_optional = true;
                        if let syn::PathArguments::AngleBracketed(angle_args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = angle_args.args.first() {
                                type_for_conversion = inner_ty; // If Option<T>, WrappedData converts to T
                                type_for_arg_type_enum = inner_ty; // Start checking T for Vec<U>
                            } else {
                                return Err(syn::Error::new_spanned(original_arg_type, "Option must have a generic type argument, e.g., Option<String>").to_compile_error().into());
                            }
                        } else {
                            return Err(syn::Error::new_spanned(original_arg_type, "Option must have angle bracketed arguments").to_compile_error().into());
                        }
                    }
                }
            }

            // Check if the *effective* type (after unwrapping Option) is Vec<U>
            if let Type::Path(type_path) = type_for_arg_type_enum {
                if type_path.path.segments.len() == 1 && type_path.path.segments[0].ident == "Vec" {
                    is_vec = true;
                    if let syn::PathArguments::AngleBracketed(angle_args) = &type_path.path.segments[0].arguments {
                        if let Some(syn::GenericArgument::Type(vec_inner_ty)) = angle_args.args.first() {
                            type_for_arg_type_enum = vec_inner_ty; // Now type_for_arg_type_enum is U from Vec<U>
                        } else {
                            return Err(syn::Error::new_spanned(original_arg_type, "Vec must have a generic type argument, e.g., Vec<String>").to_compile_error().into());
                        }
                    } else {
                        return Err(syn::Error::new_spanned(original_arg_type, "Vec must have angle bracketed arguments").to_compile_error().into());
                    }
                }
            }

            let arg_type_enum_variant;

            if let Type::Path(type_path) = &*type_for_arg_type_enum {
                let type_name = type_path.path.segments.last().unwrap().ident.to_string();

                let base_arg_type_enum = match type_name.as_str() {
                    "i8" => quote! { toli::ArgumentType::I8 },
                    "u8" => quote! { toli::ArgumentType::U8 },
                    "i16" => quote! { toli::ArgumentType::I16 },
                    "u16" => quote! { toli::ArgumentType::U16 },
                    "i32" => quote! { toli::ArgumentType::I32 },
                    "u32" => quote! { toli::ArgumentType::U32 },
                    "i64" => quote! { toli::ArgumentType::I64 },
                    "u64" => quote! { toli::ArgumentType::U64 },
                    "String" => quote! { toli::ArgumentType::Text },
                    "bool" => quote! { toli::ArgumentType::Boolean },
                    "f64" => quote! { toli::ArgumentType::Float },
                    _ => return Err(syn::Error::new_spanned(type_for_arg_type_enum, &format!("Unsupported argument type '{}' for {} macro. Only integer types (i8..u64), String, bool, f64, Vec<T>, and Option<T> of these types are supported.", type_name, macro_name)).to_compile_error().into()),
                };

                if is_vec {
                    arg_type_enum_variant = quote! { toli::ArgumentType::Vec(Box::new(#base_arg_type_enum)) };
                } else {
                    arg_type_enum_variant = base_arg_type_enum;
                }
            } else {
                return Err(syn::Error::new_spanned(type_for_arg_type_enum, &format!("Unsupported argument type for {} macro. Only integer types (i8..u64), String, bool, f64, Vec<T>, and Option<T> of these types are supported.", macro_name)).to_compile_error().into());
            }

            // Determine if the target type for try_from is a primitive integer
            let is_primitive_integer = if let Type::Path(type_path) = type_for_conversion {
                let type_name = type_path.path.segments.last().unwrap().ident.to_string();
                matches!(type_name.as_str(), "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64")
            } else {
                false
            };

            let conversion_code = if is_primitive_integer {
                quote! {
                    match wrapped_data.clone() {
                        toli::WrappedData::Number(wrapped_int_val) => {
                            let wrapped_int_for_err = wrapped_int_val.clone();
                            <#type_for_conversion as std::convert::TryFrom<toli::WrappedInt>>::try_from(wrapped_int_val)
                                .expect(&format!("Integer conversion error for argument '{}': expected {}, got {:?}", stringify!(#arg_name), stringify!(#type_for_conversion), wrapped_int_for_err))
                        },
                        _ => panic!("Type mismatch for argument '{}'. Expected {}, got {:?}", stringify!(#arg_name), stringify!(#original_arg_type), wrapped_data),
                    }
                }
            } else {
                quote! {
                    <#type_for_conversion as std::convert::TryFrom<toli::WrappedData>>::try_from(wrapped_data.clone())
                        .expect(&format!("Type mismatch for argument '{}'. Expected {}, got {:?}", stringify!(#arg_name), stringify!(#original_arg_type), wrapped_data))
                }
            };

            let arg_extraction_code = if is_optional {
                quote! {
                    match args.get(stringify!(#arg_name)) {
                        Some(toli::WrappedData::None) | None => None,
                        Some(wrapped_data) => {
                            Some(#conversion_code)
                        },
                    }
                }
            } else {
                quote! {
                    match args.get(stringify!(#arg_name)) {
                        Some(toli::WrappedData::None) => panic!("Required argument '{}' cannot be null.", stringify!(#arg_name)),
                        Some(wrapped_data) => {
                            #conversion_code
                        },
                        None => panic!("Missing required argument '{}'", stringify!(#arg_name)),
                    }
                }
            };

            args_map_creation = quote! {
                #args_map_creation
                let #arg_name: #original_arg_type = {
                    #arg_extraction_code
                };
            };
            call_args = quote! { #call_args #arg_name, };

            let arg_desc_for_this_arg = arg_descriptions_map.get(&arg_name.to_string());
            let arg_desc_literal = if let Some(desc) = arg_desc_for_this_arg {
                LitStr::new(desc, proc_macro2::Span::call_site())
            } else {
                let default_desc = format_name_for_description(&arg_name.to_string());
                LitStr::new(&default_desc, proc_macro2::Span::call_site())
            };

            let required_literal = if is_optional { quote! { false } } else { quote! { true } };

            generated_arg_inserts = quote! {
                #generated_arg_inserts
                arguments.insert(stringify!(#arg_name).to_string(), toli::IAArgument {
                    name: stringify!(#arg_name).to_string(),
                    description: #arg_desc_literal.to_string(),
                    arg_type: #arg_type_enum_variant,
                    required: #required_literal,
                });
            };
        }
    }
    Ok((args_map_creation, call_args, generated_arg_inserts))
}

/// Determines the original return type of the function.
fn get_original_return_type(input_fn: &ItemFn) -> proc_macro2::TokenStream {
    match &input_fn.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    }
}

/// Attribute macro to transform a standard Rust function into an AI tool.
///
/// This macro automatically generates a new struct that implements the `toli::IATool` trait,
/// allowing the function to be exposed and called by AI models.
///
/// # Usage
/// Apply `#[tool]` to a public function. The function's arguments and return type
/// must be one of the supported types: `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`,
/// `String`, `bool`, `f64`, `Vec<T>` (where `T` is one of the primitive types above),
/// or `Option<T>` where `T` is one of the supported types.
///
/// Documentation comments (`///`) on the function will be used as the tool's description.
/// Argument descriptions can be provided within the function's doc comment under a
/// "Parameters:" section, e.g.:
///
/// ```ignore
/// /// This is a tool that adds two numbers.
/// ///
/// /// Parameters:
/// /// - a: The first number to add.
/// /// - b: The second number to add.
/// #[tool]
/// pub fn add_numbers(a: i32, b: i32) -> i32 {
///     a + b
/// }
/// ```
///
/// For optional arguments, use `Option<T>` in the function signature. The `required`
/// field in the tool's argument description will automatically be set to `false`.
///
/// # Generated Code
/// For a function like `add_numbers(a: i32, b: i32) -> i32`, the macro will generate:
/// - The original `add_numbers` function.
/// - A new struct named `AddNumbersTool`.
/// - An `impl toli::IATool for AddNumbersTool` block, which:
///   - Implements `call` to parse a JSON `String` into native types,
///     call `add_numbers`, and returns the result.
///   - Implements `get_description` to provide `toli::IAToolDefinition` based on
///     the function's name, doc comments, and argument types.
///
/// # Panics
/// - If an argument type or return type is not supported.
/// - If a required argument is missing during a `call`.
/// - If a type mismatch occurs during argument conversion in a `call`.
#[proc_macro_attribute]
pub fn tool(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let tool_struct_name = generate_tool_struct_name(fn_name);
    let original_fn_name = &input_fn.sig.ident;

    let (fn_description_literal, arg_descriptions_map) = parse_function_docs(&input_fn, fn_name);

    let (args_map_creation, call_args, generated_arg_inserts) = match process_function_arguments(&input_fn, &arg_descriptions_map, "tool") {
        Ok(val) => val,
        Err(e) => return e,
    };

    let original_return_type = get_original_return_type(&input_fn);

    let expanded = quote! {
        #input_fn
        pub struct #tool_struct_name;

        impl toli::IATool for #tool_struct_name {
            type OriginalReturnType = #original_return_type;

            fn call(&self,  json_string_args: String) -> Self::OriginalReturnType {
                let args = self.parse_json_args(json_string_args);
                use std::convert::TryInto; // Ensure TryInto is in scope
                #args_map_creation
                let result = #original_fn_name(#call_args);
                result
            }

            fn get_description(&self) -> toli::IAToolDefinition {
                let mut arguments = std::collections::HashMap::new();
                #generated_arg_inserts

                toli::IAToolDefinition {
                    name: stringify!(#fn_name).to_string(),
                    description: #fn_description_literal.to_string(),
                    arguments,
                }
            }
        }
    };

    expanded.into()
}

/// Attribute macro to transform an asynchronous Rust function into an AI tool.
///
/// This macro automatically generates a new struct that implements the `toli::IAAsyncTool` trait,
/// allowing the async function to be exposed and called by AI models.
///
/// # Usage
/// Apply `#[async_tool]` to a public `async` function. The function's arguments and return type
/// must be one of the supported types: `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`,
/// `String`, `bool`, `f64`, `Vec<T>` (where `T` is one of the primitive types above),
/// or `Option<T>` where `T` is one of the supported types.
///
/// Documentation comments (`///`) on the function will be used as the tool's description.
/// Argument descriptions can be provided within the function's doc comment under a
/// "Parameters:" section, e.g.:
///
/// ```ignore
/// /// This is an async tool that fetches data.
/// ///
/// /// Parameters:
/// /// - query: The search query.
/// #[async_tool]
/// pub async fn fetch_data(query: String) -> String {
///     // Simulate an async operation
///     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
///     format!("Data for: {}", query)
/// }
/// ```
///
/// For optional arguments, use `Option<T>` in the function signature. The `required`
/// field in the tool's argument description will automatically be set to `false`.
///
/// # Generated Code
/// For an async function like `fetch_data(query: String) -> String`, the macro will generate:
/// - The original `fetch_data` async function.
/// - A new struct named `FetchDataTool`.
/// - An `impl toli::IAAsyncTool for FetchDataTool` block, which:
///   - Implements `call` to parse a JSON `String` into native types,
///     `await` the call to `fetch_data`, and returns the result.
///   - Implements `get_description` to provide `toli::IAToolDefinition` based on
///     the function's name, doc comments, and argument types.
///
/// # Panics
/// - If an argument type or return type is not supported.
/// - If a required argument is missing during a `call`.
/// - If a type mismatch occurs during argument conversion in a `call`.
#[proc_macro_attribute]
pub fn async_tool(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let tool_struct_name = generate_tool_struct_name(fn_name);
    let original_fn_name = &input_fn.sig.ident;

    let (fn_description_literal, arg_descriptions_map) = parse_function_docs(&input_fn, fn_name);

    let (args_map_creation, call_args, generated_arg_inserts) = match process_function_arguments(&input_fn, &arg_descriptions_map, "async_tool") {
        Ok(val) => val,
        Err(e) => return e,
    };

    let original_return_type = get_original_return_type(&input_fn);

    let expanded = quote! {
        #input_fn
        pub struct #tool_struct_name;

        #[toli::async_trait]
        impl toli::IAAsyncTool for #tool_struct_name {
            type OriginalReturnType = #original_return_type;

            async fn call(&self,  json_string_args: String) -> Self::OriginalReturnType {
                let args = self.parse_json_args(json_string_args);
                use std::convert::TryInto; // Ensure TryInto is in scope
                #args_map_creation
                let result = #original_fn_name(#call_args).await;
                result
            }

            fn get_description(&self) -> toli::IAToolDefinition {
                let mut arguments = std::collections::HashMap::new();
                #generated_arg_inserts

                toli::IAToolDefinition {
                    name: stringify!(#fn_name).to_string(),
                    description: #fn_description_literal.to_string(),
                    arguments,
                }
            }
        }
    };

    expanded.into()
}

#[cfg(test)]
mod tests {
    use super::format_name_for_description;

    #[test]
    fn test_format_name_for_description_simple_snake_case() {
        assert_eq!(format_name_for_description("add_values"), "Add values");
    }

    #[test]
    fn test_format_name_for_description_single_word() {
        assert_eq!(format_name_for_description("hello"), "Hello");
    }

    #[test]
    fn test_format_name_for_description_multiple_underscores() {
        assert_eq!(format_name_for_description("some_long_name_with_numbers_123"), "Some long name with numbers 123");
    }

    #[test]
    fn test_format_name_for_description_leading_trailing_underscores() {
        assert_eq!(format_name_for_description("_test_function_"), "_test function ");
    }

    #[test]
    fn test_format_name_for_description_empty_string() {
        assert_eq!(format_name_for_description(""), "");
    }

    #[test]
    fn test_format_name_for_description_already_formatted() {
        assert_eq!(format_name_for_description("Already formatted"), "Already formatted");
    }
}