use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use crate::{IAAsyncTool, IATool, IAToolDefinition};

/// A type alias for a boxed closure that handles a synchronous tool call.
///
/// The closure takes a `serde_json::Value` (representing the arguments)
/// and returns a `Result<String, String>` where `Ok(json_string)` is the
/// serialized result of the tool, and `Err(error_message)` if something went wrong.
pub type ToolHandler = Box<dyn Fn(Value) -> Result<String, String> + Send + Sync + 'static>;

/// A type alias for a boxed closure that handles an asynchronous tool call.
///
/// The closure takes a `serde_json::Value` (representing the arguments)
/// and returns a `Pin<Box<dyn Future<Output = Result<String, String>> + Send>>`
/// which resolves to the serialized result of the tool, or an error message.
pub type AsyncToolHandler =
Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>> + Send + Sync + 'static>;

/// Serializes a tool's result into a JSON string.
///
/// If serialization fails, it returns a predefined error message.
///
/// # Arguments
/// * `result` - The value to be serialized, must implement `serde::Serialize`.
///
/// # Returns
/// A `Result<String, String>` containing the JSON string on success,
/// or an error message on failure.
pub fn serialize_tool_result<T: Serialize>(result: T) -> Result<String, String> {
    match serde_json::to_string(&result) {
        Ok(json_string) => Ok(json_string),
        Err(_) => Err("{ \"error\": \"Cannot serialize tool result\"}".to_string()),
    }
}

/// Creates a `ToolHandler` for a given synchronous tool type.
///
/// This function abstracts the creation of the closure that will call a specific tool
/// and serialize its result.
///
/// # Type Parameters
/// * `T` - The type of the tool. It must implement `IATool` and have a `'static` lifetime.
///         Its original return type (`T::OriginalReturnType`) must also implement `serde::Serialize`.
///
/// # Arguments
/// * `tool_instance` - An instance of the tool `T`. This instance will be moved into the handler closure.
///
/// # Returns
/// A tuple containing the tool's name (`String`) and its `ToolHandler`.
pub fn create_tool_handler<T>(tool_instance: T) -> (String, ToolHandler)
where
    T: IATool + 'static + Send + Sync,
    T::OriginalReturnType: Serialize,
{
    let name = tool_instance.get_description().name.clone();

    let handler = Box::new(move |args: Value| {
        let result = tool_instance.call(args.to_string());
        serialize_tool_result(result)
    });

    (name, handler)
}

/// Creates an `AsyncToolHandler` for a given asynchronous tool type.
///
/// This function abstracts the creation of the closure that will call a specific async tool
/// and serialize its result.
///
/// # Type Parameters
/// * `T` - The type of the async tool. It must implement `IAAsyncTool` and have a `'static` lifetime.
///         Its original return type (`T::OriginalReturnType`) must also implement `serde::Serialize`.
///
/// # Arguments
/// * `tool_instance` - An instance of the async tool `T`. This instance will be moved into the handler closure.
///
/// # Returns
/// A tuple containing the tool's name (`String`) and its `AsyncToolHandler`.
pub fn create_async_tool_handler<T>(tool_instance: T) -> (String, AsyncToolHandler)
where
    T: IAAsyncTool + 'static + Send + Sync,
    T::OriginalReturnType: Serialize,
{
    let name = tool_instance.get_description().name.clone();
    let arc_tool_instance = Arc::new(tool_instance); // Wrap in Arc

    let handler = Box::new(move |args: Value| {
        // Clone the Arc for each call, then move the clone into the async block
        let cloned_arc_tool_instance = arc_tool_instance.clone();
        let future = async move {
            let result = cloned_arc_tool_instance.call(args.to_string()).await;
            serialize_tool_result(result)
        };
        // Explicitly box the `impl Future` as a `dyn Future` trait object, then pin it.
        let boxed_dyn_future: Box<dyn Future<Output = Result<String, String>> + Send> = Box::new(future);
        Pin::from(boxed_dyn_future)
    });

    (name, handler)
}

pub trait IADescriptor: Serialize + Send + Sync + 'static + Clone {
    fn from_ai_tool_definition(definition: IAToolDefinition) -> Self;
}

/// `ToolSet` manages a collection of both synchronous and asynchronous callable tools.
///
/// It stores `ToolHandler` and `AsyncToolHandler` closures in separate `HashMap`s,
/// allowing dynamic dispatch of tool calls based on their names and execution type.
pub struct ToolSet<D: IADescriptor> {
    tool_descriptors: HashMap<String, D>,
    sync_handlers: HashMap<String, ToolHandler>,
    async_handlers: HashMap<String, AsyncToolHandler>,
}

impl <D: IADescriptor> ToolSet<D> {
    /// Creates a new, empty `ToolSet`.
    pub fn new() -> Self {
        ToolSet {
            tool_descriptors: HashMap::new(),
            sync_handlers: HashMap::new(),
            async_handlers: HashMap::new(),
        }
    }

    /// Adds a new synchronous tool to the `ToolSet`.
    ///
    /// # Arguments
    /// * `tool_instance` - An instance of the synchronous tool.
    pub fn add_tool<T>(&mut self, tool_instance: T)
    where
        T: IATool + 'static + Send + Sync,
        T::OriginalReturnType: Serialize,
    {
        let definition = tool_instance.get_description();
        let (name, handler) = create_tool_handler(tool_instance);
        self.sync_handlers.insert(name.clone(), handler);
        self.tool_descriptors.insert(name, D::from_ai_tool_definition(definition));
    }

    /// Adds a new asynchronous tool to the `ToolSet`.
    ///
    /// # Arguments
    /// * `tool_instance` - An instance of the asynchronous tool.
    pub fn add_async_tool<T>(&mut self, tool_instance: T)
    where
        T: IAAsyncTool + 'static + Send + Sync,
        T::OriginalReturnType: Serialize,
    {
        let definition = tool_instance.get_description();
        let (name, handler) = create_async_tool_handler(tool_instance);
        self.async_handlers.insert(name.clone(), handler);
        self.tool_descriptors.insert(name, D::from_ai_tool_definition(definition));
    }

    /// Dispatches a tool call by its name and arguments.
    ///
    /// It first looks up a synchronous handler, and if not found,
    /// then looks for an asynchronous handler.
    ///
    /// # Arguments
    /// * `function_name` - The name of the tool to call.
    /// * `args` - The arguments for the tool, as a `serde_json::Value`.
    ///
    /// # Returns
    /// A `Result<String, String>` containing the serialized tool result on success, or a predefined error message if the tool is
    /// not found or the handler fails.
    pub async fn dispatch(&self, function_name: String, args: Value) -> Result<String, String> {
        if let Some(handler) = self.sync_handlers.get(&function_name) {
            // Found a synchronous handler
            handler(args)
        } else if let Some(handler) = self.async_handlers.get(&function_name) {
            // Found an asynchronous handler
            handler(args).await
        } else {
            Err("{ \"error\": \"Tool not found or cannot be dispatched\"}".to_string())
        }
    }

    /// Retrieves all stored tool descriptors.
    pub fn get_tools(&self) -> Vec<D> {
        self.tool_descriptors.values().cloned().collect()
    }
}