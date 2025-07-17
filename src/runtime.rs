#![allow(clippy::result_large_err)]
use rustyscript::Runtime;

use crate::JsWorkerOptions;

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
                let output = executor.execute(&name, args).await?;
                Ok::<_, rustyscript::Error>(output)
            })
        })?;
    }
    Ok(runtime)
}
