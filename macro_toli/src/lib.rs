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

#[proc_macro_attribute]
pub fn tool(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;

    // --- Convert snake_case function name to UpperCamelCase for the struct name ---
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

    let fn_description = fn_description_lines.join("\n");
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
            let arg_type = &pat_type.ty;

            let arg_type_enum_variant;
            let wrapped_data_extraction;

            if let Type::Path(type_path) = &**arg_type {
                let type_name = type_path.path.segments.last().unwrap().ident.to_string();

                match type_name.as_str() {
                    "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" => {
                        // Convert "i8" -> "I8", "u32" -> "U32" for ArgumentType variant
                        let arg_type_ident = format_ident!("{}", {
                            let mut chars = type_name.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                            }
                        });
                        arg_type_enum_variant = quote! { toli::ArgumentType::#arg_type_ident };
                        wrapped_data_extraction = quote! {
                            Some(toli::WrappedData::Number(wrapped_int_val_ref)) => {
                                // Clone wrapped_int_val_ref to pass by value to try_into()
                                let val: #arg_type = wrapped_int_val_ref.clone().try_into()
                                    .expect(&format!("Integer conversion error for argument '{}': expected {}, got {:?}", stringify!(#arg_name), stringify!(#arg_type), wrapped_int_val_ref));
                                val
                            },
                        };
                    },
                    "String" => {
                        arg_type_enum_variant = quote! { toli::ArgumentType::Text };
                        wrapped_data_extraction = quote! {
                            Some(toli::WrappedData::Text(val_ref)) => val_ref.clone(),
                        };
                    },
                    "bool" => {
                        arg_type_enum_variant = quote! { toli::ArgumentType::Boolean };
                        wrapped_data_extraction = quote! {
                            Some(toli::WrappedData::Boolean(val_ref)) => *val_ref,
                        };
                    },
                    "f64" => {
                        arg_type_enum_variant = quote! { toli::ArgumentType::Float };
                        wrapped_data_extraction = quote! {
                            Some(toli::WrappedData::Float(val_ref)) => *val_ref,
                        };
                    },
                    _ => return syn::Error::new_spanned(arg_type, &format!("Unsupported argument type '{}' for tool macro. Only integer types (i8..u64), String, bool, f64 are supported.", type_name)).to_compile_error().into(),
                }
            } else {
                return syn::Error::new_spanned(arg_type, "Unsupported argument type for tool macro. Only integer types (i8..u64), String, bool, f64 are supported.").to_compile_error().into();
            }


            args_map_creation = quote! {
                #args_map_creation
                let #arg_name: #arg_type = {
                    use std::convert::TryInto; // Import TryInto for integer conversions
                    match args.get(stringify!(#arg_name)) {
                        #wrapped_data_extraction
                        Some(other) => panic!("Type mismatch for argument '{}'. Expected {}, got {:?}", stringify!(#arg_name), stringify!(#arg_type), other),
                        None => panic!("Missing argument '{}'", stringify!(#arg_name)),
                    }
                };
            };
            call_args = quote! { #call_args #arg_name, };

            let arg_desc_for_this_arg = arg_descriptions_map.get(&arg_name.to_string());
            // Generate default argument description if empty
            let arg_desc_literal = if let Some(desc) = arg_desc_for_this_arg {
                LitStr::new(desc, proc_macro2::Span::call_site())
            } else {
                let default_desc = format_name_for_description(&arg_name.to_string());
                LitStr::new(&default_desc, proc_macro2::Span::call_site())
            };


            generated_arg_inserts = quote! {
                #generated_arg_inserts
                arguments.insert(stringify!(#arg_name).to_string(), toli::IAArgument {
                    name: stringify!(#arg_name).to_string(),
                    description: #arg_desc_literal.to_string(),
                    arg_type: #arg_type_enum_variant,
                    required: true,
                });
            };
        }
    }

    let output_type = &input_fn.sig.output;
    let return_conversion = match output_type {
        ReturnType::Default => quote! { toli::WrappedData::Text("".to_string()) },
        ReturnType::Type(_, ty) => {
            if let Type::Path(type_path) = &**ty {
                let type_name = type_path.path.segments.last().unwrap().ident.to_string();
                match type_name.as_str() {
                    "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" => {
                        // Use .into() which leverages From<PrimitiveType> for WrappedInt
                        quote! { toli::WrappedData::Number(result.into()) }
                    },
                    "String" => quote! { toli::WrappedData::Text(result) },
                    "bool" => quote! { toli::WrappedData::Boolean(result) },
                    "f64" => quote! { toli::WrappedData::Float(result) },
                    _ => return syn::Error::new_spanned(ty, &format!("Unsupported return type '{}' for tool macro. Only integer types (i8..u64), String, bool, f64 are supported.", type_name)).to_compile_error().into(),
                }
            } else {
                return syn::Error::new_spanned(ty, "Unsupported return type for tool macro. Only integer types (i8..u64), String, bool, f64 are supported.").to_compile_error().into();
            }
        }
    };

    let expanded = quote! {
        #input_fn
        pub struct #tool_struct_name;

        impl toli::IATool for #tool_struct_name {
            fn call(&self, args: std::collections::HashMap<String, toli::WrappedData>) -> toli::WrappedData {
                #args_map_creation
                let result = #original_fn_name(#call_args);
                #return_conversion
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