mod worker;

use std::sync::Arc;

pub use worker::*;
mod runtime;

pub type JsWorkerResult<T> = Result<T, JsWorkerError>;

#[derive(Debug, Clone, Default)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: Option<String>,
    pub parameters: serde_json::Value,
    pub is_async: bool,
}

/// The runtime options for our worker
#[derive(Clone)]
pub struct JsWorkerOptions {
    pub timeout: std::time::Duration,
    pub functions: Vec<FunctionDefinition>,
    pub executor: Arc<dyn JsExecutor>,
}

#[async_trait::async_trait]
pub trait JsExecutor: Send + Sync {
    async fn execute(
        &self,
        name: &str,
        args: Vec<serde_json::Value>,
    ) -> JsWorkerResult<serde_json::Value>;
    fn execute_sync(
        &self,
        name: &str,
        args: Vec<serde_json::Value>,
    ) -> JsWorkerResult<serde_json::Value>;
}

#[derive(Debug, thiserror::Error)]
pub enum JsWorkerError {
    #[error("JsError: {0}")]
    JsError(String),
    #[error("SerdeError: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Timeout")]
    Timeout,
    #[error("Other: {0}")]
    Other(String),
}
#[cfg(test)]
mod tests;
