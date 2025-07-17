#![allow(clippy::result_large_err)]
use rustyscript::{
    worker::{InnerWorker, Worker},
    Error, Runtime,
};

use crate::{runtime::init_runtime, JsWorkerOptions};

pub struct JsWorker(Worker<JsWorker>, JsWorkerOptions);

impl JsWorker {
    /// Create a new instance of the worker
    pub fn new(options: JsWorkerOptions) -> Result<Self, rustyscript::Error> {
        Ok(Self(Worker::new(options.clone())?, options))
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

    /// Execute a snippet of JS code on our threaded worker
    pub fn execute<T>(&self, code: &str) -> Result<T, rustyscript::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let code = self.append_functions(code);
        match self.0.send_and_await(JsWorkerMessage::Execute(code))? {
            JsWorkerMessage::Value(v) => Ok(serde_json::from_value(v)?),
            JsWorkerMessage::Error(e) => Err(e),
            _ => Err(rustyscript::Error::Runtime(
                "Unexpected response".to_string(),
            )),
        }
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
