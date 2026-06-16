use rquickjs::{
    function::Opt,
    module::{Declarations, Exports, ModuleDef},
    Class, Ctx, Function, Result,
};

mod registry;
mod send;
mod spec;
mod stream;

pub use send::send_signal;
pub use spec::{is_terminating_signal, parse_signal, SignalSpec};

use spec::{is_supported_signal, signal_list};
use stream::SignalStream;

fn signal_send_js(pid: u32, signal: Opt<String>) -> Result<bool> {
    Ok(send_signal(pid, parse_signal(signal.0.as_deref())))
}

fn signal_is_supported(raw: String) -> bool {
    is_supported_signal(&raw)
}

/// Native module for system signal helpers.
///
/// 只关注信号语义与发送，不承担进程生命周期管理。
pub struct OsSignalRawModule;

impl ModuleDef for OsSignalRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> Result<()> {
        decl.declare("__std_signal_send")?;
        decl.declare("__std_signal_list")?;
        decl.declare("__std_signal_is_supported")?;
        decl.declare("SignalStream")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        let globals = ctx.globals();
        Class::<SignalStream>::define(&globals)?;

        exports.export(
            "__std_signal_send",
            Function::new(ctx.clone(), signal_send_js)?,
        )?;
        exports.export(
            "__std_signal_list",
            Function::new(ctx.clone(), signal_list)?,
        )?;
        exports.export(
            "__std_signal_is_supported",
            Function::new(ctx.clone(), signal_is_supported)?,
        )?;

        let signal_stream_ctor: Function<'js> = globals.get("SignalStream")?;
        exports.export("SignalStream", signal_stream_ctor)?;
        let _ = globals.remove("SignalStream");

        Ok(())
    }
}
