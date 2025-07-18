use std::sync::Arc;

use crate::{
    FunctionDefinition, JsExecutor, JsWorker, JsWorkerError, JsWorkerOptions, JsWorkerResult,
};
use serde_json::Value;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Default)]
pub struct EchoExecutor {}

#[async_trait::async_trait]
impl JsExecutor for EchoExecutor {
    async fn execute(
        &self,
        name: &str,
        args: Vec<serde_json::Value>,
    ) -> JsWorkerResult<serde_json::Value> {
        let str = format!("[EchoJsExecutor]:Executing function: {name} with args: {args:?}");
        Ok(serde_json::Value::String(str))
    }
}

#[tokio::test]
async fn test_echo_async() -> Result<(), JsWorkerError> {
    let executor = EchoExecutor::default();
    let worker = JsWorker::new(JsWorkerOptions {
        timeout: std::time::Duration::from_secs(1),
        functions: vec![FunctionDefinition {
            name: "echo".to_string(),
            description: Some("Echo a message".to_string()),
            parameters: serde_json::json!({}),
            returns: Some("The echoed message".to_string()),
        }],
        print_tx: None,
        executor: Arc::new(executor),
    })
    .map_err(|e| JsWorkerError::JsError(e.to_string()))?;

    let result: Value = worker.execute("echo('Hello, world!');").unwrap();

    assert!(result.as_str().unwrap().contains("Hello, world!"));

    Ok(())
}

#[tokio::test]
async fn print_test() -> Result<(), JsWorkerError> {
    let executor = EchoExecutor::default();
    let (tx, mut rx) = mpsc::channel::<Value>(1);
    let worker = JsWorker::new(JsWorkerOptions {
        timeout: std::time::Duration::from_secs(1),
        functions: vec![],
        executor: Arc::new(executor),
        print_tx: Some(Arc::new(tx)),
    })
    .map_err(|e| JsWorkerError::JsError(e.to_string()))?;

    let handle = tokio::spawn(async move {
        let result = rx.recv().await.unwrap();
        result
    });
    let result: Value = worker.execute("console.log(1);").unwrap();

    assert!(result.is_null());

    let result = handle.await.unwrap();
    assert_eq!(result, serde_json::Value::Number(1.into()));

    Ok(())
}
