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

        if f.is_async {
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
        } else {
            runtime.register_function(&name, move |args| {
                let name = f.name.clone();
                let executor = executor.clone();
                let args = args.to_vec();
                executor
                    .execute_sync(&name, args)
                    .map_err(|e| rustyscript::Error::Runtime(e.to_string()))
            })?;
        }
    }
    Ok(runtime)
}
