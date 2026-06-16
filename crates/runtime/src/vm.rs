use rquickjs::context::EvalOptions;
use rquickjs::{AsyncContext, AsyncRuntime, Ctx, FromJs, Function, Promise, Value};
use std::path::Path;
use std::time::Duration;
use vole_core::until::{UntilRegistry, WaitUntilError};
use vole_module::{ModuleLoader, ModuleResolver};

pub struct Vm {
    pub runtime: AsyncRuntime,
    pub ctx: AsyncContext,
    pub until_registry: UntilRegistry,
}

#[derive(Debug, thiserror::Error)]
pub enum VmError {
    #[error("Failed to create VM: {0}")]
    CreationFailed(String),
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error("Type conversion error: {0}")]
    ConversionError(String),
    #[error("Lifecycle error: {0}")]
    LifecycleError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl Vm {
    pub async fn new() -> Result<Self, VmError> {
        vole_raw_env::load_dotenv();

        let runtime = AsyncRuntime::new().map_err(|e| VmError::CreationFailed(e.to_string()))?;

        let base_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
        let resolver = ModuleResolver::new(base_dir);
        let mut loader = ModuleLoader::new();
        loader.add_native_provider(Box::new(crate::native_modules::provider()));

        runtime.set_loader(resolver, loader).await;

        let ctx = AsyncContext::full(&runtime)
            .await
            .map_err(|e| VmError::CreationFailed(e.to_string()))?;

        let until_registry = UntilRegistry::new();
        let registry_for_ctx = until_registry.clone();
        ctx.with(move |ctx| {
            let _ = ctx.store_userdata(registry_for_ctx);
            Ok::<_, rquickjs::Error>(())
        })
        .await
        .map_err(|e| VmError::CreationFailed(e.to_string()))?;

        // Initialize all bindings: global APIs, raw module backends, and shims.
        crate::bindings::init(&ctx)
            .await
            .map_err(|e| VmError::CreationFailed(e.to_string()))?;

        // Hook unhandled Promise rejections so they are dispatched as
        // "unhandledrejection" / "rejectionhandled" events on globalThis.
        runtime
            .set_host_promise_rejection_tracker(Some(Box::new(
                move |ctx: Ctx<'_>, promise: Value<'_>, reason: Value<'_>, is_handled: bool| {
                    if let Ok(dispatch) =
                        ctx.globals().get::<_, Function>("__voleDispatchRejection")
                    {
                        let _ = dispatch.call::<_, bool>((promise, reason, is_handled));
                    }
                },
            )))
            .await;

        Ok(Vm {
            runtime,
            ctx,
            until_registry,
        })
    }

    pub async fn eval(&self, code: &str) -> Result<String, VmError> {
        let output = self
            .ctx
            .with(|ctx| match ctx.eval::<rquickjs::Value, _>(code) {
                Ok(result) => Ok::<_, rquickjs::Error>(format_js_value(&result)),
                Err(e) => {
                    let err = ctx.catch();
                    let _ = vole_global_events::dispatch_error_event(&ctx, err);
                    Err(e)
                }
            })
            .await
            .map_err(|e| VmError::ExecutionError(e.to_string()))?;
        self.drain_managed_until(Duration::from_millis(30_000))
            .await?;
        Ok(output)
    }

    pub async fn eval_file(&self, path: &Path) -> Result<String, VmError> {
        let source = std::fs::read_to_string(path)?;
        let code = if is_typescript_file(path) {
            let transpiled =
                vole_module::transpile_typescript(&source, path.to_str().unwrap_or("")).map_err(
                    |e| VmError::ExecutionError(format!("TypeScript transpile error: {}", e)),
                )?;
            String::from_utf8(transpiled)
                .map_err(|e| VmError::ExecutionError(format!("UTF-8 error: {}", e)))?
        } else {
            source
        };
        let output = self
            .ctx
            .async_with(async |ctx| {
                let module_name = path.to_string_lossy().to_string();
                let promise = rquickjs::Module::evaluate(ctx.clone(), module_name, code)?;
                let result: rquickjs::Value = match promise.into_future().await {
                    Ok(v) => v,
                    Err(e) => {
                        let err = ctx.catch();
                        let _ = vole_global_events::dispatch_error_event(&ctx, err);
                        return Err(e);
                    }
                };

                // Run event loop to process spawned timers
                for _ in 0..3 {
                    tokio::task::yield_now().await;
                }
                let mut waited = 0u64;
                while !vole_global_timers::is_idle() && waited < 30_000 {
                    ctx.execute_pending_job();
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    waited += 10;
                }

                Ok::<_, rquickjs::Error>(format_js_value(&result))
            })
            .await
            .map_err(|e| VmError::ExecutionError(e.to_string()))?;
        self.drain_managed_until(Duration::from_millis(30_000))
            .await?;
        Ok(output)
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

    pub async fn get_global<T: for<'js> FromJs<'js> + Send>(
        &self,
        name: &str,
    ) -> Result<T, VmError> {
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

    pub fn managed_until_pending_count(&self) -> usize {
        self.until_registry.pending_count()
    }

    pub fn managed_until_scope_count(&self) -> usize {
        self.until_registry.scope_count()
    }

    pub fn managed_until_is_idle(&self) -> bool {
        self.until_registry.is_idle()
    }

    pub fn close_managed_until(&self) {
        self.until_registry.close();
    }

    pub async fn drain_managed_until(&self, timeout: Duration) -> Result<(), VmError> {
        if self.until_registry.is_idle() {
            return Ok(());
        }

        let started_at = tokio::time::Instant::now();
        while !self.until_registry.is_idle() {
            if started_at.elapsed() >= timeout {
                return Err(wait_until_error(WaitUntilError::Timeout));
            }

            self.ctx
                .with(|ctx| {
                    let _ = ctx.execute_pending_job();
                    Ok::<_, rquickjs::Error>(())
                })
                .await
                .map_err(|e| VmError::LifecycleError(e.to_string()))?;

            tokio::task::yield_now().await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        Ok(())
    }

    pub async fn run_tests(&self, files: &[std::path::PathBuf]) -> Result<String, VmError> {
        for file in files {
            self.eval_file(file).await?;
        }

        let code = r#"import { run } from "vole:test"; await run();"#;
        let output = self
            .ctx
            .async_with(async |ctx| {
                let mut options = EvalOptions::default();
                options.global = false;
                options.promise = true;
                let promise: Promise = ctx.eval_with_options(code, options)?;
                let result: rquickjs::Value = match promise.into_future().await {
                    Ok(v) => v,
                    Err(e) => {
                        let err = ctx.catch();
                        let _ = vole_global_events::dispatch_error_event(&ctx, err);
                        return Err(e);
                    }
                };

                for _ in 0..3 {
                    tokio::task::yield_now().await;
                }
                let mut waited = 0u64;
                while !vole_global_timers::is_idle() && waited < 60_000 {
                    ctx.execute_pending_job();
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    waited += 10;
                }

                Ok::<_, rquickjs::Error>(format_js_value(&result))
            })
            .await
            .map_err(|e| VmError::ExecutionError(e.to_string()))?;
        self.drain_managed_until(Duration::from_millis(60_000))
            .await?;
        Ok(output)
    }
}

fn wait_until_error(err: WaitUntilError) -> VmError {
    match err {
        WaitUntilError::Closed => VmError::LifecycleError("WaitUntil registry is closed".into()),
        WaitUntilError::Timeout => {
            VmError::LifecycleError("WaitUntil registry drain timed out".into())
        }
    }
}

fn is_typescript_file(path: &Path) -> bool {
    let path_lower = path.to_string_lossy().to_lowercase();
    path_lower.ends_with(".ts") || path_lower.ends_with(".mts") || path_lower.ends_with(".tsx")
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
