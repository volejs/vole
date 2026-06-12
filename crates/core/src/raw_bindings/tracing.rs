use rquickjs::{
    module::{Declarations, Exports, ModuleDef},
    Ctx, Function, Result,
};

fn tracing_info(msg: String) {
    println!("[INFO] {}", msg);
}

fn tracing_warn(msg: String) {
    eprintln!("[WARN] {}", msg);
}

fn tracing_error(msg: String) {
    eprintln!("[ERROR] {}", msg);
}

fn tracing_debug(msg: String) {
    println!("[DEBUG] {}", msg);
}

fn tracing_trace(msg: String) {
    println!("[TRACE] {}", msg);
}

/// Native module exposing the low-level tracing operations used by the `std:tracing` JS shim.
pub struct TracingRawModule;

impl ModuleDef for TracingRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("__std_tracing_info")?;
        decl.declare("__std_tracing_warn")?;
        decl.declare("__std_tracing_error")?;
        decl.declare("__std_tracing_debug")?;
        decl.declare("__std_tracing_trace")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export("__std_tracing_info", Function::new(ctx.clone(), tracing_info))?;
        exports.export("__std_tracing_warn", Function::new(ctx.clone(), tracing_warn))?;
        exports.export("__std_tracing_error", Function::new(ctx.clone(), tracing_error))?;
        exports.export("__std_tracing_debug", Function::new(ctx.clone(), tracing_debug))?;
        exports.export("__std_tracing_trace", Function::new(ctx.clone(), tracing_trace))?;
        Ok(())
    }
}

/// Legacy global init is now a no-op for `std:tracing`.
/// All tracing helpers are provided through the native module `std:raw:tracing`.
pub fn init(_ctx: &Ctx) -> Result<()> {
    Ok(())
}
