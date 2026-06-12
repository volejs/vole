use rquickjs::{Context, Runtime as JsRuntime};
use std::sync::Arc;

pub struct Runtime {
    rt: Arc<JsRuntime>,
}

impl Runtime {
    pub fn new() -> Result<Self, RuntimeError> {
        let rt = JsRuntime::new().map_err(|_| RuntimeError::CreationFailed)?;
        Ok(Runtime {
            rt: Arc::new(rt),
        })
    }

    pub fn eval(&self, code: &str) -> Result<String, RuntimeError> {
        let ctx = Context::full(&self.rt).map_err(|_| RuntimeError::CreationFailed)?;
        
        ctx.with(|ctx| {
            // Add console.log support using a simple JavaScript implementation
            // Since we can't easily add Rust callbacks, we'll just skip console.log for now
            // and let the user handle it in their JavaScript code
            
            let result: rquickjs::Value = ctx.eval(code)?;
            Ok(format_js_value(&result))
        }).map_err(|e: rquickjs::Error| RuntimeError::ExecutionError(e.to_string()))
    }

    pub fn global_object(&self) -> String {
        // rquickjs doesn't expose a direct global object accessor
        "undefined".to_string()
    }
}

impl Clone for Runtime {
    fn clone(&self) -> Self {
        Runtime {
            rt: Arc::clone(&self.rt),
        }
    }
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
        value.as_string()
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

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Failed to create runtime")]
    CreationFailed,
    #[error("Conversion error")]
    ConversionError,
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error("Not implemented")]
    NotImplemented,
}
