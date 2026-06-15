use std::time::Duration;

use rquickjs::{
    class::{JsClass, Trace, Tracer, Writable},
    function::{Constructor, Opt},
    module::{Declarations, Exports, ModuleDef},
    Class, Ctx, Exception, Function, Object, Promise, Result,
};
use vole_core::until::{UntilRegistry, WaitUntil, WaitUntilError, WaitUntilGuard};

#[derive(Debug)]
struct WaitUntilHandle {
    until: WaitUntil,
    managed: bool,
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for WaitUntilHandle {
    type Changed<'to> = WaitUntilHandle;
}

impl<'js> Trace<'js> for WaitUntilHandle {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> JsClass<'js> for WaitUntilHandle {
    const NAME: &'static str = "WaitUntilHandle";
    type Mutable = Writable;

    fn prototype(_ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        Ok(None)
    }

    fn constructor(_ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        Ok(None)
    }
}

#[derive(Debug)]
struct WaitUntilGuardHandle {
    guard: Option<WaitUntilGuard>,
}

unsafe impl<'js> rquickjs::JsLifetime<'js> for WaitUntilGuardHandle {
    type Changed<'to> = WaitUntilGuardHandle;
}

impl<'js> Trace<'js> for WaitUntilGuardHandle {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> JsClass<'js> for WaitUntilGuardHandle {
    const NAME: &'static str = "WaitUntilGuardHandle";
    type Mutable = Writable;

    fn prototype(_ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        Ok(None)
    }

    fn constructor(_ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        Ok(None)
    }
}

fn until_error<'js>(ctx: &Ctx<'js>, err: WaitUntilError) -> rquickjs::Error {
    match err {
        WaitUntilError::Closed => Exception::throw_type(ctx, "WaitUntil is closed"),
        WaitUntilError::Timeout => Exception::throw_message(ctx, "WaitUntil drain timed out"),
    }
}

fn registry_for_ctx<'js>(ctx: &Ctx<'js>) -> UntilRegistry {
    if let Some(registry) = ctx.userdata::<UntilRegistry>() {
        return (*registry).clone();
    }

    let registry = UntilRegistry::new();
    let _ = ctx.store_userdata(registry.clone());
    registry
}

fn until_create<'js>(
    ctx: Ctx<'js>,
    managed: Opt<bool>,
) -> Result<Class<'js, WaitUntilHandle>> {
    let managed = managed.0.unwrap_or(true);
    let until = if managed {
        registry_for_ctx(&ctx).create_scope()
    } else {
        WaitUntil::new()
    };

    Class::instance(ctx, WaitUntilHandle { until, managed })
}

fn until_begin<'js>(
    ctx: Ctx<'js>,
    handle: Class<'js, WaitUntilHandle>,
) -> Result<Class<'js, WaitUntilGuardHandle>> {
    let guard = handle
        .try_borrow()?
        .until
        .begin()
        .map_err(|err| until_error(&ctx, err))?;
    Class::instance(ctx, WaitUntilGuardHandle { guard: Some(guard) })
}

fn until_finish<'js>(guard: Class<'js, WaitUntilGuardHandle>) -> Result<()> {
    if let Some(guard) = guard.try_borrow_mut()?.guard.take() {
        guard.finish();
    }
    Ok(())
}

fn until_pending_count<'js>(handle: Class<'js, WaitUntilHandle>) -> Result<usize> {
    Ok(handle.try_borrow()?.until.pending_count())
}

fn until_is_idle<'js>(handle: Class<'js, WaitUntilHandle>) -> Result<bool> {
    Ok(handle.try_borrow()?.until.is_idle())
}

fn until_is_closed<'js>(handle: Class<'js, WaitUntilHandle>) -> Result<bool> {
    Ok(handle.try_borrow()?.until.is_closed())
}

fn until_is_managed<'js>(handle: Class<'js, WaitUntilHandle>) -> Result<bool> {
    Ok(handle.try_borrow()?.managed)
}

fn until_close<'js>(handle: Class<'js, WaitUntilHandle>) -> Result<()> {
    handle.try_borrow()?.until.close();
    Ok(())
}

fn until_drain<'js>(
    ctx: Ctx<'js>,
    handle: Class<'js, WaitUntilHandle>,
    timeout_ms: Opt<u64>,
) -> Result<Promise<'js>> {
    let until = handle.try_borrow()?.until.clone();
    let timeout = Duration::from_millis(timeout_ms.0.unwrap_or(30_000));
    let (promise, resolve, reject) = Promise::new(&ctx)?;

    ctx.spawn(async move {
        match until.drain(timeout).await {
            Ok(()) => {
                let _ = resolve.call::<_, ()>(());
            }
            Err(err) => {
                let message = match err {
                    WaitUntilError::Closed => "WaitUntil is closed",
                    WaitUntilError::Timeout => "WaitUntil drain timed out",
                };
                let _ = reject.call::<_, ()>((message,));
            }
        }
    });

    Ok(promise)
}

fn until_registry_pending_count<'js>(ctx: Ctx<'js>) -> Result<usize> {
    Ok(registry_for_ctx(&ctx).pending_count())
}

fn until_registry_scope_count<'js>(ctx: Ctx<'js>) -> Result<usize> {
    Ok(registry_for_ctx(&ctx).scope_count())
}

fn until_registry_is_idle<'js>(ctx: Ctx<'js>) -> Result<bool> {
    Ok(registry_for_ctx(&ctx).is_idle())
}

fn until_registry_close<'js>(ctx: Ctx<'js>) -> Result<()> {
    registry_for_ctx(&ctx).close();
    Ok(())
}

fn until_registry_drain<'js>(ctx: Ctx<'js>, timeout_ms: Opt<u64>) -> Result<Promise<'js>> {
    let registry = registry_for_ctx(&ctx);
    let timeout = Duration::from_millis(timeout_ms.0.unwrap_or(30_000));
    let (promise, resolve, reject) = Promise::new(&ctx)?;

    ctx.spawn(async move {
        match registry.drain(timeout).await {
            Ok(()) => {
                let _ = resolve.call::<_, ()>(());
            }
            Err(err) => {
                let message = match err {
                    WaitUntilError::Closed => "WaitUntil registry is closed",
                    WaitUntilError::Timeout => "WaitUntil registry drain timed out",
                };
                let _ = reject.call::<_, ()>((message,));
            }
        }
    });

    Ok(promise)
}

pub fn init(_ctx: &Ctx) -> Result<()> {
    Ok(())
}

pub struct UntilRawModule;

impl ModuleDef for UntilRawModule {
    fn declare(declare: &Declarations) -> Result<()> {
        declare.declare("__until_create")?;
        declare.declare("__until_begin")?;
        declare.declare("__until_finish")?;
        declare.declare("__until_pending_count")?;
        declare.declare("__until_is_idle")?;
        declare.declare("__until_is_closed")?;
        declare.declare("__until_is_managed")?;
        declare.declare("__until_close")?;
        declare.declare("__until_drain")?;
        declare.declare("__until_registry_pending_count")?;
        declare.declare("__until_registry_scope_count")?;
        declare.declare("__until_registry_is_idle")?;
        declare.declare("__until_registry_close")?;
        declare.declare("__until_registry_drain")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        let globals = ctx.globals();
        Class::<WaitUntilHandle>::define(&globals)?;
        Class::<WaitUntilGuardHandle>::define(&globals)?;
        let _ = globals.remove("WaitUntilHandle");
        let _ = globals.remove("WaitUntilGuardHandle");

        exports.export("__until_create", Function::new(ctx.clone(), until_create)?)?;
        exports.export("__until_begin", Function::new(ctx.clone(), until_begin)?)?;
        exports.export("__until_finish", Function::new(ctx.clone(), until_finish)?)?;
        exports.export(
            "__until_pending_count",
            Function::new(ctx.clone(), until_pending_count)?,
        )?;
        exports.export("__until_is_idle", Function::new(ctx.clone(), until_is_idle)?)?;
        exports.export("__until_is_closed", Function::new(ctx.clone(), until_is_closed)?)?;
        exports.export("__until_is_managed", Function::new(ctx.clone(), until_is_managed)?)?;
        exports.export("__until_close", Function::new(ctx.clone(), until_close)?)?;
        exports.export("__until_drain", Function::new(ctx.clone(), until_drain)?)?;
        exports.export(
            "__until_registry_pending_count",
            Function::new(ctx.clone(), until_registry_pending_count)?,
        )?;
        exports.export(
            "__until_registry_scope_count",
            Function::new(ctx.clone(), until_registry_scope_count)?,
        )?;
        exports.export(
            "__until_registry_is_idle",
            Function::new(ctx.clone(), until_registry_is_idle)?,
        )?;
        exports.export(
            "__until_registry_close",
            Function::new(ctx.clone(), until_registry_close)?,
        )?;
        exports.export(
            "__until_registry_drain",
            Function::new(ctx.clone(), until_registry_drain)?,
        )?;
        Ok(())
    }
}
