use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::{ArgumentType, IAArgument, IADescriptor, IAToolDefinition};

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
    description: Option<String>,
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
            description: Some(ai_tool_definition.description),
            parameters,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OpenAIParameterDescriptor {
    #[serde(rename = "type")]
    parameter_type: String,
    properties: HashMap<String, OpenAIPropertyDescriptor>,
    required: Option<Vec<String>>,
}

impl OpenAIParameterDescriptor {
    fn new(arguments: HashMap<String, IAArgument>) -> Self {
        let mut properties_map = HashMap::new();
        let mut required_args = Vec::new();

        for (arg_name, arg_def) in arguments {
            properties_map.insert(arg_name.clone(), OpenAIPropertyDescriptor::new(arg_def));
            required_args.push(arg_name);
        }

        OpenAIParameterDescriptor {
            parameter_type: "object".to_string(),
            properties: properties_map,
            required: Some(required_args),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OpenAIPropertyDescriptor {
    #[serde(rename = "type")]
    property_type: String,
    description: Option<String>,
}

impl OpenAIPropertyDescriptor {
    fn new(argument: IAArgument) -> Self {
        let property_type = match argument.arg_type {
            ArgumentType::I8 | ArgumentType::U8 |
            ArgumentType::I16 | ArgumentType::U16 |
            ArgumentType::I32 | ArgumentType::U32 |
            ArgumentType::I64 | ArgumentType::U64 => "integer".to_string(),
            ArgumentType::Text => "string".to_string(),
            ArgumentType::Boolean => "boolean".to_string(),
            ArgumentType::Float => "number".to_string(),
        };

        OpenAIPropertyDescriptor {
            property_type,
            description: Some(argument.description),
        }
    }
}