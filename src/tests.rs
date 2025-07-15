use std::sync::Arc;

use rustyscript::Error;

use crate::{FunctionDefinition, JsExecutor, JsWorker, JsWorkerOptions};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct EchoExecutor {}

#[async_trait::async_trait]
impl JsExecutor for EchoExecutor {
    async fn execute(
        &self,
        name: &str,
        args: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value, rustyscript::Error> {
        let str = format!("[EchoJsExecutor]:Executing function: {name} with args: {args:?}");
        Ok(serde_json::Value::String(str))
    }
}

#[tokio::test]
async fn test_echo_async() -> Result<(), Error> {
    let executor = EchoExecutor::default();
    let worker = JsWorker::new(JsWorkerOptions {
        timeout: std::time::Duration::from_secs(1),
        functions: vec![FunctionDefinition {
            name: "echo".to_string(),
            description: Some("Echo a message".to_string()),
            parameters: serde_json::json!({}),
            returns: Some("The echoed message".to_string()),
        }],
        executor: Arc::new(executor),
    })?;

    let result: Value = worker.execute("echo('Hello, world!');").unwrap();

    assert_eq!(result, "Hello, world!");

    Ok(())
}
