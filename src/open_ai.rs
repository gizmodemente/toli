use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::{ArgumentType, IAArgument, IADescriptor, IAToolDefinition};

/// Helper function to convert an empty string to None
fn get_optional_description(desc: String) -> Option<String> {
    if desc.is_empty() {
        None
    } else {
        Some(desc)
    }
}

/// Represents a function tool in the OpenAI format, which has become a de facto standard
/// for Large Language Models (LLMs). This structure is used to describe functions
/// that an AI model can call, including their name, description, and parameters
/// defined using JSON Schema.
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIFunctionTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIToolDescriptor,
}

impl OpenAIFunctionTool {
    pub fn new(descriptor: OpenAIToolDescriptor) -> Self {
        OpenAIFunctionTool {
            tool_type: "function".to_string(),
            function: descriptor,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAIToolDescriptor {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<OpenAIParameterDescriptor>,
}

impl IADescriptor for OpenAIToolDescriptor {
    fn from_ai_tool_definition(ai_tool_definition: IAToolDefinition) -> Self {
        let parameters = if ai_tool_definition.arguments.is_empty() {
            None
        } else {
            Some(OpenAIParameterDescriptor::new(ai_tool_definition.arguments))
        };

        OpenAIToolDescriptor {
            name: ai_tool_definition.name,
            description: get_optional_description(ai_tool_definition.description),
            parameters,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OpenAIParameterDescriptor {
    #[serde(rename = "type")]
    parameter_type: String,
    properties: HashMap<String, OpenAIPropertyDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<Vec<String>>,
}

impl OpenAIParameterDescriptor {
    fn new(arguments: HashMap<String, IAArgument>) -> Self {
        let mut properties_map = HashMap::new();
        let mut required_args = Vec::new();

        for (arg_name, arg_def) in arguments {
            if arg_def.required {
                required_args.push(arg_name.clone());
            }
            properties_map.insert(arg_name.clone(), OpenAIPropertyDescriptor::new(arg_def));
        }

        OpenAIParameterDescriptor {
            parameter_type: "object".to_string(),
            properties: properties_map,
            required: if required_args.is_empty() { None } else { Some(required_args) },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OpenAIPropertyDescriptor {
    #[serde(rename = "type")]
    property_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<Box<OpenAIPropertyDescriptor>>,
}

impl OpenAIPropertyDescriptor {
    fn new(argument: IAArgument) -> Self {
        match argument.arg_type {
            ArgumentType::I8 | ArgumentType::U8 |
            ArgumentType::I16 | ArgumentType::U16 |
            ArgumentType::I32 | ArgumentType::U32 |
            ArgumentType::I64 | ArgumentType::U64 => OpenAIPropertyDescriptor {
                property_type: "integer".to_string(),
                description: get_optional_description(argument.description),
                items: None,
            },
            ArgumentType::Text => OpenAIPropertyDescriptor {
                property_type: "string".to_string(),
                description: get_optional_description(argument.description),
                items: None,
            },
            ArgumentType::Boolean => OpenAIPropertyDescriptor {
                property_type: "boolean".to_string(),
                description: get_optional_description(argument.description),
                items: None,
            },
            ArgumentType::Float => OpenAIPropertyDescriptor {
                property_type: "number".to_string(),
                description: get_optional_description(argument.description),
                items: None,
            },
            ArgumentType::Vec(inner_type) => {
                let inner_arg = IAArgument {
                    name: "".to_string(), // Name is not relevant for inner type
                    description: "".to_string(), // Description is not relevant for inner type
                    arg_type: *inner_type,
                    required: false, // Required is not relevant for inner type
                };
                OpenAIPropertyDescriptor {
                    property_type: "array".to_string(),
                    description: get_optional_description(argument.description), // This is the description of the array itself
                    items: Some(Box::new(OpenAIPropertyDescriptor::new(inner_arg))),
                }
            }
        }
    }
}