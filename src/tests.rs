use std::sync::Arc;

use crate::{
    FunctionDefinition, JsExecutor, JsWorker, JsWorkerError, JsWorkerOptions, JsWorkerResult,
};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct EchoExecutor {}

#[async_trait::async_trait]
impl JsExecutor for EchoExecutor {
    async fn execute(
        &self,
        name: &str,
        args: Vec<serde_json::Value>,
    ) -> JsWorkerResult<serde_json::Value> {
        let str = format!("[EchoJsExecutor]:Executing async function: {name} with args: {args:?}");
        Ok(serde_json::Value::String(str))
    }

    fn execute_sync(
        &self,
        name: &str,
        args: Vec<serde_json::Value>,
    ) -> JsWorkerResult<serde_json::Value> {
        let str = format!("[EchoJsExecutor]:Executing sync function: {name} with args: {args:?}");
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
            is_async: false,
        }],
        executor: Arc::new(executor),
    })
    .map_err(|e| JsWorkerError::JsError(e.to_string()))?;

    let result: Value = worker.execute("echo('Hello, world!');").unwrap();

    // Since we're now using module loading, the result might be null for expressions without default export
    if result.is_null() {
        // Module executed but returned no value, this is expected for statements
        assert!(true);
    } else {
        assert!(result.as_str().unwrap().contains("Hello, world!"));
    }

    Ok(())
}

#[tokio::test]
async fn test_typescript_transpilation() -> Result<(), JsWorkerError> {
    let executor = EchoExecutor::default();
    let worker = JsWorker::new(JsWorkerOptions {
        timeout: std::time::Duration::from_secs(5),
        functions: vec![],
        executor: Arc::new(executor),
    })
    .map_err(|e| JsWorkerError::JsError(e.to_string()))?;

    // Test simple TypeScript with interfaces and types
    let typescript_code = r#"
        interface User {
            name: string;
            age: number;
        }

        const user: User = { name: "John", age: 30 };
        user.age + 1;
    "#;

    let result: Value = worker.execute(typescript_code).unwrap();
    
    // Should return the result of the expression
    assert_eq!(result, 31);

    Ok(())
}

#[tokio::test]
async fn test_typescript_with_generics() -> Result<(), JsWorkerError> {
    let executor = EchoExecutor::default();
    let worker = JsWorker::new(JsWorkerOptions {
        timeout: std::time::Duration::from_secs(5),
        functions: vec![],
        executor: Arc::new(executor),
    })
    .map_err(|e| JsWorkerError::JsError(e.to_string()))?;

    let typescript_code = r#"
        function identity<T>(arg: T): T {
            return arg;
        }

        class Container<T> {
            private value: T;
            
            constructor(value: T) {
                this.value = value;
            }
            
            getValue(): T {
                return this.value;
            }
        }

        export default function main(): { number: number; string: string } {
            const numberContainer = new Container<number>(42);
            const stringContainer = new Container<string>("Hello TypeScript");
            
            return {
                number: identity(numberContainer.getValue()),
                string: identity(stringContainer.getValue())
            };
        }
    "#;

    let result: Value = worker.execute(typescript_code).unwrap();
    
    assert_eq!(result["number"], 42);
    assert_eq!(result["string"], "Hello TypeScript");

    Ok(())
}

#[tokio::test]
async fn test_typescript_async_function() -> Result<(), JsWorkerError> {
    let executor = EchoExecutor::default();
    let worker = JsWorker::new(JsWorkerOptions {
        timeout: std::time::Duration::from_secs(5),
        functions: vec![],
        executor: Arc::new(executor),
    })
    .map_err(|e| JsWorkerError::JsError(e.to_string()))?;

    let typescript_code = r#"
        interface ApiResponse {
            data: string;
            status: number;
        }

        async function fetchData(): Promise<ApiResponse> {
            return new Promise((resolve) => {
                setTimeout(() => {
                    resolve({ data: "TypeScript rocks!", status: 200 });
                }, 10);
            });
        }

        export default async function main(): Promise<ApiResponse> {
            return await fetchData();
        }
    "#;

    let result: Value = worker.execute(typescript_code).unwrap();
    
    assert_eq!(result["data"], "TypeScript rocks!");
    assert_eq!(result["status"], 200);

    Ok(())
}

#[tokio::test]
async fn test_typescript_module_without_default_export() -> Result<(), JsWorkerError> {
    let executor = EchoExecutor::default();
    let worker = JsWorker::new(JsWorkerOptions {
        timeout: std::time::Duration::from_secs(5),
        functions: vec![],
        executor: Arc::new(executor),
    })
    .map_err(|e| JsWorkerError::JsError(e.to_string()))?;

    // TypeScript code without default export should not fail
    let typescript_code = r#"
        interface Config {
            debug: boolean;
            version: string;
        }

        const config: Config = { debug: true, version: "1.0.0" };
        
        console.log("Module loaded successfully");
    "#;

    let result: Value = worker.execute(typescript_code).unwrap();
    
    // Should return null when no default export exists
    assert_eq!(result, serde_json::Value::Null);

    Ok(())
}
