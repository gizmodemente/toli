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

/// Helper function to generate the conversion logic from WrappedData to the inner Rust type.
fn get_conversion_logic(inner_type: &Type, arg_name: &syn::Ident) -> proc_macro2::TokenStream {
    if let Type::Path(type_path) = inner_type {
        let type_name = type_path.path.segments.last().unwrap().ident.to_string();
        match type_name.as_str() {
            "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" => {
                quote! {
                    toli::WrappedData::Number(wrapped_int_val_ref) => {
                        let val: #inner_type = wrapped_int_val_ref.clone().try_into()
                            .expect(&format!("Integer conversion error for argument '{}': expected {}, got {:?}", stringify!(#arg_name), stringify!(#inner_type), wrapped_int_val_ref));
                        val
                    }, // Added comma
                }
            },
            "String" => {
                quote! {
                    toli::WrappedData::Text(val_ref) => val_ref.clone(), // Added comma
                }
            },
            "bool" => {
                quote! {
                    toli::WrappedData::Boolean(val_ref) => *val_ref, // Added comma
                }
            },
            "f64" => {
                quote! {
                    toli::WrappedData::Float(val_ref) => *val_ref, // Added comma
                }
            },
            _ => quote! { compile_error!("Unsupported inner type for Option") }, // This case should ideally be caught earlier
        }
    } else {
        quote! { compile_error!("Unsupported inner type for Option") } // This case should ideally be caught earlier
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
/// `String`, `bool`, `f64`, or `Option<T>` where `T` is one of the supported types.
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
/// ```ignore
/// /// This is a tool that greets a person.
/// ///
/// /// Parameters:
/// /// - name: The name of the person to greet.
/// /// - greeting: An optional custom greeting. Defaults to "Hello".
/// #[tool]
/// pub fn greet_person(name: String, greeting: Option<String>) -> String {
///     match greeting {
///         Some(g) => format!("{} {}", g, name),
///         None => format!("Hello {}", name),
///     }
/// }
/// ```
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

    // --- Convert snake_case function name to UpperCamelCamelCase for the struct name ---
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
    let tool_struct_name = format_ident!("{}Tool", tool_struct_name_str);
    // --- End conversion ---

    let original_fn_name = &input_fn.sig.ident;

    // --- Parse doc comments for function description and argument descriptions ---
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

    // Process lines for function description and argument descriptions
    for line in full_doc_comment.lines() {
        let trimmed_line = line.trim();

        // Check for ignorable headers
        if ignorable_headers.iter().any(|&header| trimmed_line.starts_with(header)) {
            current_section_is_ignorable = true;
            in_parameters_section = false; // If we hit an ignorable section, we are no longer in parameters
            continue; // Skip this header line
        }

        // Check for Parameters section
        if trimmed_line.starts_with("Parameters:") {
            in_parameters_section = true;
            current_section_is_ignorable = false; // Parameters section is not ignorable
            continue; // Skip this header line
        }

        // If we are in an ignorable section, skip the line
        if current_section_is_ignorable {
            continue;
        }

        // If we are in the parameters section, parse arguments
        if in_parameters_section {
            // Parse parameter line: "- argument1: Description of argument 1"
            if let Some(rest_after_dash) = trimmed_line.strip_prefix("- ") {
                if let Some(colon_idx) = rest_after_dash.find(':') {
                    let arg_name_str = rest_after_dash[..colon_idx].trim();
                    let arg_desc_str = rest_after_dash[colon_idx + 1..].trim();
                    arg_descriptions_map.insert(arg_name_str.to_string(), arg_desc_str.to_string());
                }
            }
        } else { // Not in parameters section, not in ignorable section, so it's function description
            if !trimmed_line.is_empty() {
                fn_description_lines.push(trimmed_line.to_string());
            }
        }
    }

    let fn_description = fn_description_lines.join(" ");
    // Generate default function description if empty
    let fn_description = if fn_description.is_empty() {
        format_name_for_description(&fn_name.to_string())
    } else {
        fn_description
    };
    let fn_description_literal = LitStr::new(&fn_description, proc_macro2::Span::call_site());
    // --- End doc comment parsing ---


    let mut args_map_creation = quote! {};
    let mut call_args = quote! {};
    let mut generated_arg_inserts = quote! {};

    for arg in &input_fn.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            let arg_name = if let Pat::Ident(pat_ident) = &*pat_type.pat {
                &pat_ident.ident
            } else {
                return syn::Error::new_spanned(pat_type, "Unsupported argument pattern").to_compile_error().into();
            };
            let original_arg_type = &pat_type.ty; // This is &Box<Type>
            let mut is_optional = false;
            // Initialize inner_type_for_parsing as a reference to the Type inside the Box
            let mut inner_type_for_parsing: &Type = original_arg_type;

            let arg_type_enum_variant;
            let arg_type_for_definition; // This will be the type used in the IAArgument definition

            // Check for Option<T>
            if let Type::Path(type_path) = &**original_arg_type { // Dereference Box to get Type
                if type_path.path.segments.len() == 1 && type_path.path.segments[0].ident == "Option" {
                    is_optional = true;
                    if let syn::PathArguments::AngleBracketed(angle_args) = &type_path.path.segments[0].arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = angle_args.args.first() {
                            inner_type_for_parsing = inner_ty; // Assign &Type directly
                        } else {
                            return syn::Error::new_spanned(original_arg_type, "Option must have a generic type argument, e.g., Option<String>").to_compile_error().into();
                        }
                    } else {
                        return syn::Error::new_spanned(original_arg_type, "Option must have angle bracketed arguments").to_compile_error().into();
                    }
                }
            }

            // Determine arg_type_enum_variant and arg_type_for_definition based on inner_type_for_parsing
            if let Type::Path(type_path) = &*inner_type_for_parsing {
                let type_name = type_path.path.segments.last().unwrap().ident.to_string();

                match type_name.as_str() {
                    "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" => {
                        let arg_type_ident = format_ident!("{}", {
                            let mut chars = type_name.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                            }
                        });
                        arg_type_enum_variant = quote! { toli::ArgumentType::#arg_type_ident };
                        arg_type_for_definition = inner_type_for_parsing;
                    },
                    "String" => {
                        arg_type_enum_variant = quote! { toli::ArgumentType::Text };
                        arg_type_for_definition = inner_type_for_parsing;
                    },
                    "bool" => {
                        arg_type_enum_variant = quote! { toli::ArgumentType::Boolean };
                        arg_type_for_definition = inner_type_for_parsing;
                    },
                    "f64" => {
                        arg_type_enum_variant = quote! { toli::ArgumentType::Float };
                        arg_type_for_definition = inner_type_for_parsing;
                    },
                    _ => return syn::Error::new_spanned(inner_type_for_parsing, &format!("Unsupported argument type '{}' for tool macro. Only integer types (i8..u64), String, bool, f64, and Option<T> of these types are supported.", type_name)).to_compile_error().into(),
                }
            } else {
                return syn::Error::new_spanned(inner_type_for_parsing, "Unsupported argument type for tool macro. Only integer types (i8..u64), String, bool, f64, and Option<T> of these types are supported.").to_compile_error().into();
            }

            let conversion_logic_for_some_wrapped_data = get_conversion_logic(inner_type_for_parsing, arg_name);

            let arg_extraction_code = if is_optional {
                quote! {
                    match args.get(stringify!(#arg_name)) {
                        Some(toli::WrappedData::None) | None => None,
                        Some(wrapped_data) => {
                            let converted_val = match wrapped_data {
                                #conversion_logic_for_some_wrapped_data
                                _ => panic!("Type mismatch for argument '{}'. Expected Option<{}>, got {:?}", stringify!(#arg_name), stringify!(#arg_type_for_definition), wrapped_data),
                            };
                            Some(converted_val)
                        },
                    }
                }
            } else {
                quote! {
                    match args.get(stringify!(#arg_name)) {
                        Some(toli::WrappedData::None) => panic!("Required argument '{}' cannot be null.", stringify!(#arg_name)),
                        Some(wrapped_data) => {
                            match wrapped_data {
                                #conversion_logic_for_some_wrapped_data
                                _ => panic!("Type mismatch for argument '{}'. Expected {}, got {:?}", stringify!(#arg_name), stringify!(#original_arg_type), wrapped_data),
                            }
                        },
                        None => panic!("Missing required argument '{}'", stringify!(#arg_name)),
                    }
                }
            };

            args_map_creation = quote! {
                #args_map_creation
                let #arg_name: #original_arg_type = {
                    use std::convert::TryInto; // Import TryInto for integer conversions
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

    // Determine the original return type of the function
    let original_return_type = match &input_fn.sig.output {
        ReturnType::Default => quote! { () }, // Unit type
        ReturnType::Type(_, ty) => quote! { #ty },
    };

    let expanded = quote! {
        #input_fn
        pub struct #tool_struct_name;

        impl toli::IATool for #tool_struct_name {
            type OriginalReturnType = #original_return_type; // Set the associated type

            fn call(&self,  json_string_args: String) -> Self::OriginalReturnType {
                let args = self.parse_json_args(json_string_args);
                #args_map_creation
                let result = #original_fn_name(#call_args);
                result // Directly return the result
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
    use super::format_name_for_description; // Import the helper function

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