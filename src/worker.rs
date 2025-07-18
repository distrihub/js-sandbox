use rustyscript::{
    worker::{InnerWorker, Worker},
    Error, Runtime,
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
            str += &format!(
                " let {} = rustyscript.async_functions['{}'];",
                f.name, f.name
            );
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

    /// Execute a snippet of JS code on our threaded worker
    pub fn execute<T>(&self, code: &str) -> JsWorkerResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let code = self.append_functions(code);
        let code = self.wrap_async_block(&code);
        let res = match self
            .0
            .send_and_await(JsWorkerMessage::Execute(code))
            .map_err(|e| JsWorkerError::JsError(e.to_string()))?
        {
            JsWorkerMessage::Value(v) => Ok(serde_json::from_value(v)?),
            JsWorkerMessage::Error(e) => Err(JsWorkerError::JsError(e.to_string())),
            _ => Err(JsWorkerError::Other("Unexpected response".to_string())),
        };

        res
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
            JsWorkerMessage::Execute(code) => match runtime.eval::<serde_json::Value>(&code) {
                Ok(value) => JsWorkerMessage::Value(value),
                Err(e) => JsWorkerMessage::Error(e),
            },

            JsWorkerMessage::Error(e) => JsWorkerMessage::Error(e),
            JsWorkerMessage::Value(v) => JsWorkerMessage::Value(v),
        }
    }
}
