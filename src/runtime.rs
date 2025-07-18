use rustyscript::Runtime;

use crate::JsWorkerOptions;

#[allow(clippy::result_large_err)]
pub fn init_runtime(options: JsWorkerOptions) -> Result<Runtime, rustyscript::Error> {
    let mut runtime = Runtime::new(rustyscript::RuntimeOptions {
        timeout: options.timeout,
        ..Default::default()
    })?;

    for f in options.functions {
        let executor = options.executor.clone();
        let name = f.name.clone();

        runtime.register_async_function(&name, move |args| {
            let name = f.name.clone();
            let executor = executor.clone();
            let args = args.to_vec();
            Box::pin(async move {
                executor
                    .execute(&name, args)
                    .await
                    .map_err(|e| rustyscript::Error::Runtime(e.to_string()))
            })
        })?;
    }

    let print_tx = options.print_tx.clone();
    runtime.register_async_function("print", move |args| {
        if let Some(print_tx) = print_tx.clone() {
            let print_tx = print_tx.clone();
            let val = args.iter().next().cloned().unwrap_or_default();
            Box::pin(async move {
                let _ = print_tx.send(val).await;
                Ok(serde_json::Value::Null)
            })
        } else {
            Box::pin(async move { Ok(serde_json::Value::Null) })
        }
    })?;

    Ok(runtime)
}
