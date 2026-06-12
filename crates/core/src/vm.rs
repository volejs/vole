use rquickjs::{AsyncContext, AsyncRuntime, Ctx, FromJs};
use rquickjs::context::EvalOptions;
use vole_module::{ModuleLoader, ModuleResolver};
use std::path::Path;
use std::time::Duration;

pub struct Vm {
    pub runtime: AsyncRuntime,
    pub ctx: AsyncContext,
}

#[derive(Debug, thiserror::Error)]
pub enum VmError {
    #[error("Failed to create VM: {0}")]
    CreationFailed(String),
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error("Type conversion error: {0}")]
    ConversionError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl Vm {
    pub async fn new() -> Result<Self, VmError> {
        crate::raw_bindings::env::load_dotenv();

        let runtime = AsyncRuntime::new().map_err(|e| VmError::CreationFailed(e.to_string()))?;

        let base_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
        let resolver = ModuleResolver::new(base_dir);
        let mut loader = ModuleLoader::new();
        loader.add_native_provider(Box::new(crate::native_modules::provider()));

        runtime.set_loader(resolver, loader).await;

        let ctx = AsyncContext::full(&runtime)
            .await
            .map_err(|e| VmError::CreationFailed(e.to_string()))?;

        // Initialize all bindings: global APIs, raw module backends, and shims.
        crate::bindings::init(&ctx)
            .await
            .map_err(|e| VmError::CreationFailed(e.to_string()))?;

        Ok(Vm { runtime, ctx })
    }

    pub async fn eval(&self, code: &str) -> Result<String, VmError> {
        self.ctx
            .with(|ctx| {
                let result: rquickjs::Value = ctx.eval(code)?;
                Ok::<_, rquickjs::Error>(format_js_value(&result))
            })
            .await
            .map_err(|e| VmError::ExecutionError(e.to_string()))
    }

    pub async fn eval_file(&self, path: &Path) -> Result<String, VmError> {
        let source = std::fs::read_to_string(path)?;
        let code = if is_typescript_file(path) {
            let transpiled = vole_module::transpile_typescript(&source, path.to_str().unwrap_or(""))
                .map_err(|e| VmError::ExecutionError(format!("TypeScript transpile error: {}", e)))?;
            String::from_utf8(transpiled).map_err(|e| VmError::ExecutionError(format!("UTF-8 error: {}", e)))?
        } else {
            source
        };
        self.ctx
            .async_with(async |ctx| {
                let module_name = path.to_string_lossy().to_string();
                let promise = rquickjs::Module::evaluate(ctx.clone(), module_name, code)?;
                let result: rquickjs::Value = promise.into_future().await?;

                // Run event loop to process spawned timers
                for _ in 0..3 {
                    tokio::task::yield_now().await;
                }
                let mut waited = 0u64;
                while !crate::global_bindings::timers::is_idle() && waited < 30_000 {
                    ctx.execute_pending_job();
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    waited += 10;
                }

                Ok::<_, rquickjs::Error>(format_js_value(&result))
            })
            .await
            .map_err(|e| VmError::ExecutionError(e.to_string()))
    }

    pub async fn set_global_string(&self, name: &str, value: &str) -> Result<(), VmError> {
        self.ctx
            .with(|ctx| {
                let global = ctx.globals();
                global.set(name, value)?;
                Ok::<_, rquickjs::Error>(())
            })
            .await
            .map_err(|e| VmError::ConversionError(e.to_string()))
    }

    pub async fn set_global_number(&self, name: &str, value: f64) -> Result<(), VmError> {
        self.ctx
            .with(|ctx| {
                let global = ctx.globals();
                global.set(name, value)?;
                Ok::<_, rquickjs::Error>(())
            })
            .await
            .map_err(|e| VmError::ConversionError(e.to_string()))
    }

    pub async fn set_global_bool(&self, name: &str, value: bool) -> Result<(), VmError> {
        self.ctx
            .with(|ctx| {
                let global = ctx.globals();
                global.set(name, value)?;
                Ok::<_, rquickjs::Error>(())
            })
            .await
            .map_err(|e| VmError::ConversionError(e.to_string()))
    }

    pub async fn get_global<T: for<'js> FromJs<'js> + Send>(&self, name: &str) -> Result<T, VmError> {
        self.ctx
            .with(|ctx| {
                let global = ctx.globals();
                let value: T = global.get(name)?;
                Ok::<_, rquickjs::Error>(value)
            })
            .await
            .map_err(|e| VmError::ConversionError(e.to_string()))
    }

    pub async fn with_context<F, R>(&self, f: F) -> Result<R, VmError>
    where
        F: for<'js> FnOnce(Ctx<'js>) -> Result<R, rquickjs::Error> + Send,
        R: Send,
    {
        self.ctx
            .with(f)
            .await
            .map_err(|e| VmError::ExecutionError(e.to_string()))
    }

    pub async fn run_gc(&self) {
        self.runtime.run_gc().await;
    }

    pub async fn idle(&self) {
        self.runtime.idle().await;
    }

    pub async fn run_tests(&self, files: &[std::path::PathBuf]) -> Result<String, VmError> {
        for file in files {
            self.eval_file(file).await?;
        }

        let code = r#"import { run } from "std:test"; await run();"#;
        self.ctx
            .async_with(async |ctx| {
                let mut options = EvalOptions::default();
                options.global = false;
                options.promise = true;
                let result: rquickjs::Value = ctx.eval_with_options(code, options)?;

                for _ in 0..3 {
                    tokio::task::yield_now().await;
                }
                let mut waited = 0u64;
                while !crate::global_bindings::timers::is_idle() && waited < 60_000 {
                    ctx.execute_pending_job();
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    waited += 10;
                }

                Ok::<_, rquickjs::Error>(format_js_value(&result))
            })
            .await
            .map_err(|e| VmError::ExecutionError(e.to_string()))
    }
}

fn is_typescript_file(path: &Path) -> bool {
    let path_lower = path.to_string_lossy().to_lowercase();
    path_lower.ends_with(".ts")
        || path_lower.ends_with(".mts")
        || path_lower.ends_with(".tsx")
}

fn format_js_value(value: &rquickjs::Value) -> String {
    if value.is_undefined() {
        "undefined".to_string()
    } else if value.is_null() {
        "null".to_string()
    } else if value.is_bool() {
        value.as_bool().unwrap().to_string()
    } else if value.is_number() {
        value.as_number().unwrap().to_string()
    } else if value.is_string() {
        value
            .as_string()
            .and_then(|s| s.to_string().ok())
            .unwrap_or_else(|| "string".to_string())
    } else if value.is_array() {
        "[array]".to_string()
    } else if value.is_object() {
        "[object Object]".to_string()
    } else {
        "unknown".to_string()
    }
}
