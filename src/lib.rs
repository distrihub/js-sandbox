mod worker;

use std::sync::Arc;

pub use worker::*;
mod runtime;

#[derive(Debug, Clone, Default)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: Option<String>,
    pub parameters: serde_json::Value,
    pub returns: Option<String>,
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
    ) -> Result<serde_json::Value, rustyscript::Error>;
}

#[cfg(test)]
mod tests;
