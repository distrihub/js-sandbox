use rustyscript::{
    worker::{InnerWorker, Worker},
    Error, Runtime, Module,
};

use crate::{runtime::init_runtime, JsWorkerError, JsWorkerOptions, JsWorkerResult};

pub struct JsWorker(Worker<JsWorker>, JsWorkerOptions);

impl JsWorker {
    /// Create a new instance of the worker
    pub fn new(options: JsWorkerOptions) -> JsWorkerResult<Self> {
        Ok(Self(
            Worker::new(options.clone()).map_err(|e| JsWorkerError::JsError(e.to_string()))?,
            options,
        ))
    }

    // Make them available just by their names
    pub fn append_functions(&self, code: &str) -> String {
        let mut str = String::new();
        self.1.functions.iter().for_each(|f| {
            let line = if f.is_async {
                format!(
                    " let {} = rustyscript.async_functions['{}'];",
                    f.name, f.name
                )
            } else {
                format!(" let {} = rustyscript.functions['{}'];", f.name, f.name)
            };
            str += &line;
        });
        str.push_str(code);
        str
    }

    pub fn wrap_async_block(&self, code: &str) -> String {
        format!(
            r#"
(async () => {{
    try {{
        {}
    }} catch (error) {{
        console.log('Error in code execution:', error);
        throw error;
    }}
}})()
"#,
            code
        )
    }

    /// Execute a snippet of JS/TS code on our threaded worker
    pub fn execute<T>(&self, code: &str) -> JsWorkerResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let res = match self
            .0
            .send_and_await(JsWorkerMessage::Execute(code.to_string()))
            .map_err(|e| JsWorkerError::JsError(e.to_string()))?
        {
            JsWorkerMessage::Value(v) => Ok(serde_json::from_value(v)?),
            JsWorkerMessage::Error(e) => Err(JsWorkerError::JsError(e.to_string())),
            _ => Err(JsWorkerError::Other("Unexpected response".to_string())),
        };

        res
    }
    fn ensure_return_last_expression(code: &str) -> String {
        let lines: Vec<&str> = code.trim().lines().collect();
        if lines.is_empty() {
            return code.to_string();
        }

        let mut result = String::new();
        for (i, line) in lines.iter().enumerate() {
            if i == lines.len() - 1 && !line.trim().is_empty() {
                let trimmed = line.trim();
                // If last line is an expression (ends with ;), convert to return
                if trimmed.ends_with(';') && !trimmed.starts_with("return") 
                   && !trimmed.starts_with("const") && !trimmed.starts_with("let") 
                   && !trimmed.starts_with("var") && !trimmed.starts_with("function")
                   && !trimmed.starts_with("class") && !trimmed.starts_with("interface")
                   && !trimmed.starts_with("type") && !trimmed.starts_with("if")
                   && !trimmed.starts_with("for") && !trimmed.starts_with("while")
                   && !trimmed.starts_with("switch") {
                    // Remove semicolon and add return
                    let expr = &trimmed[..trimmed.len()-1];
                    result.push_str(&format!("return {};\n", expr));
                } else {
                    result.push_str(&format!("{}\n", line));
                }
            } else {
                result.push_str(&format!("{}\n", line));
            }
        }
        result
    }
}

/// The messages we will use to communicate with the worker
pub enum JsWorkerMessage {
    Execute(String),
    Error(Error),
    Value(serde_json::Value),
}
impl InnerWorker for JsWorker {
    type Query = JsWorkerMessage;
    type Response = JsWorkerMessage;
    type RuntimeOptions = JsWorkerOptions;
    type Runtime = Runtime;

    /// Initialize the runtime using the options provided
    fn init_runtime(options: Self::RuntimeOptions) -> Result<Self::Runtime, Error> {
        init_runtime(options)
    }

    /// Handle all possible queries
    fn handle_query(runtime: &mut Self::Runtime, query: Self::Query) -> Self::Response {
        match query {
            JsWorkerMessage::Execute(code) => {
                // Wrap user code in a module with default export, handling last expression
                let processed_code = Self::ensure_return_last_expression(code);
                let user_module_code = format!(
                    r#"
export default function main() {{
    {}
}}
"#,
                    processed_code
                );
                
                // Create TypeScript module to enable transpilation
                let user_module = Module::new("user_module.ts", &user_module_code);
                
                // Load the user module
                match runtime.load_modules(&user_module, vec![]) {
                    Ok(module_handle) => {
                        // Call the main function from the loaded module
                        match runtime.call_function(Some(&module_handle), "default", rustyscript::json_args![]) {
                            Ok(value) => JsWorkerMessage::Value(value),
                            Err(e) => JsWorkerMessage::Error(e),
                        }
                    }
                    Err(e) => JsWorkerMessage::Error(e),
                }
            }

            JsWorkerMessage::Error(e) => JsWorkerMessage::Error(e),
            JsWorkerMessage::Value(v) => JsWorkerMessage::Value(v),
        }
    }
}
