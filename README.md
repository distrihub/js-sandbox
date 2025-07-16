## Distri JS Sandbox

**Distri JS Sandbox** enables you to register LLM function definitions and runs them in a sandbox. This utility is designed to let LLMs (Large Language Models) generate plans in code format and execute them primarily designed for distri Code Agent.

### ✨ Features
- **LLM-Driven Execution:** Designed for scenarios where LLMs generate code plans that need to be executed in a controlled, sandboxed environment.
- **Sandboxed & Safe:** Built on top of [rustyscript](https://github.com/rscarson/rustyscript) and Deno for secure, isolated execution.

### 🚀 Example Usage

```rust


#[tokio::main]
async fn main() {
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
    }).unwrap();

    let result: Value = worker.execute("echo('Hello, world!');").unwrap();
    println!("{}", result); // Output: "Hello, world!"
}
```

### 📦 Built With

- [rustyscript](https://github.com/rscarson/rustyscript) — Rust bindings for Deno, enabling JS execution.
- [Deno](https://deno.com/) — Secure JavaScript/TypeScript runtime.
- [tokio](https://tokio.rs/) — Async runtime for Rust.

### 🔗 See Also

- **[distri](https://github.com/your-org/distri)** — The broader project for Agent building & orchestration..

---

