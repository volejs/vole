use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicU32, AtomicUsize, Ordering},
        Mutex, OnceLock,
    },
    time::Duration,
};

use rquickjs::{
    function::{Opt, Rest},
    Coerced, Ctx, Function, Object, Result, Value,
};
use vole_core::error::ErrorExtensions;

static NEXT_ID: AtomicU32 = AtomicU32::new(1);
static ACTIVE_IDS: OnceLock<Mutex<HashSet<u32>>> = OnceLock::new();
static ACTIVE_TASKS: AtomicUsize = AtomicUsize::new(0);

fn active_ids() -> &'static Mutex<HashSet<u32>> {
    ACTIVE_IDS.get_or_init(|| Mutex::new(HashSet::new()))
}

pub fn init_registry() {
    let _ = active_ids();
}

pub fn task_begin() {
    ACTIVE_TASKS.fetch_add(1, Ordering::SeqCst);
}

pub fn task_end() {
    ACTIVE_TASKS.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |count| {
        Some(count.saturating_sub(1))
    }).ok();
}

pub fn has_pending_tasks() -> bool {
    ACTIVE_TASKS.load(Ordering::SeqCst) > 0
}

pub fn is_idle() -> bool {
    !has_pending_tasks()
}

fn next_timer_id() -> u32 {
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

fn register_timer(id: u32) {
    active_ids().lock().unwrap().insert(id);
    task_begin();
}

fn clear_registered_timer(id: u32) {
    if active_ids().lock().unwrap().remove(&id) {
        task_end();
    }
}

fn is_timer_active(id: u32) -> bool {
    active_ids().lock().unwrap().contains(&id)
}

fn normalize_delay(value: Option<Value<'_>>) -> Result<u64> {
    let Some(value) = value else {
        return Ok(0);
    };
    if value.is_undefined() {
        return Ok(0);
    }

    let delay = value.get::<Coerced<f64>>()?.0;
    if !delay.is_finite() || delay <= 0.0 {
        return Ok(0);
    }

    Ok(delay.min(i32::MAX as f64) as u64)
}

fn timer_id_from_value(value: Option<Value<'_>>) -> Result<Option<u32>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_undefined() || value.is_null() {
        return Ok(None);
    }

    let id = value.get::<Coerced<f64>>()?.0;
    if !id.is_finite() || id < 0.0 {
        return Ok(None);
    }
    Ok(Some(id as u32))
}

fn call_callback<'js>(ctx: &Ctx<'js>, callback: &Function<'js>, args: &[Value<'js>]) {
    if let Err(error) = callback.call::<_, ()>((Rest(args.to_vec()),)) {
        report_callback_error(ctx, error);
    }
}

fn report_callback_error<'js>(ctx: &Ctx<'js>, error: rquickjs::Error) {
    let Ok(value) = error.into_value(ctx) else {
        return;
    };
    if let Ok(report_error) = ctx.globals().get::<_, Function>("reportError") {
        let _ = report_error.call::<_, ()>((value,));
    }
}

fn set_timeout<'js>(
    ctx: Ctx<'js>,
    callback: Function<'js>,
    delay: Opt<Value<'js>>,
    args: Rest<Value<'js>>,
) -> Result<u32> {
    let delay = normalize_delay(delay.0)?;
    let id = next_timer_id();
    register_timer(id);

    ctx.clone().spawn(async move {
        tokio::time::sleep(Duration::from_millis(delay)).await;
        if !is_timer_active(id) {
            return;
        }
        clear_registered_timer(id);
        call_callback(&ctx, &callback, &args.0);
    });

    Ok(id)
}

fn set_interval<'js>(
    ctx: Ctx<'js>,
    callback: Function<'js>,
    delay: Opt<Value<'js>>,
    args: Rest<Value<'js>>,
) -> Result<u32> {
    let delay = normalize_delay(delay.0)?.max(1);
    let id = next_timer_id();
    register_timer(id);

    ctx.clone().spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(delay)).await;
            if !is_timer_active(id) {
                return;
            }
            call_callback(&ctx, &callback, &args.0);
        }
    });

    Ok(id)
}

fn set_immediate<'js>(
    ctx: Ctx<'js>,
    callback: Function<'js>,
    args: Rest<Value<'js>>,
) -> Result<u32> {
    let id = next_timer_id();
    register_timer(id);

    ctx.clone().spawn(async move {
        tokio::task::yield_now().await;
        if !is_timer_active(id) {
            return;
        }
        clear_registered_timer(id);
        call_callback(&ctx, &callback, &args.0);
    });

    Ok(id)
}

fn clear_timer(id: Opt<Value<'_>>) -> Result<()> {
    if let Some(id) = timer_id_from_value(id.0)? {
        clear_registered_timer(id);
    }
    Ok(())
}

fn queue_microtask(callback: Function<'_>) -> Result<()> {
    callback.defer(())
}

pub fn init(ctx: &Ctx) -> Result<()> {
    let globals = ctx.globals();
    let timers = Object::new(ctx.clone())?;

    let set_timeout_fn = Function::new(ctx.clone(), set_timeout)?;
    let clear_timeout_fn = Function::new(ctx.clone(), clear_timer)?;
    let set_interval_fn = Function::new(ctx.clone(), set_interval)?;
    let clear_interval_fn = Function::new(ctx.clone(), clear_timer)?;
    let set_immediate_fn = Function::new(ctx.clone(), set_immediate)?;
    let clear_immediate_fn = Function::new(ctx.clone(), clear_timer)?;
    let queue_microtask_fn = Function::new(ctx.clone(), queue_microtask)?;

    globals.set("setTimeout", set_timeout_fn.clone())?;
    globals.set("clearTimeout", clear_timeout_fn.clone())?;
    globals.set("setInterval", set_interval_fn.clone())?;
    globals.set("clearInterval", clear_interval_fn.clone())?;
    globals.set("setImmediate", set_immediate_fn.clone())?;
    globals.set("clearImmediate", clear_immediate_fn.clone())?;
    globals.set("queueMicrotask", queue_microtask_fn.clone())?;

    timers.set("setTimeout", set_timeout_fn)?;
    timers.set("clearTimeout", clear_timeout_fn)?;
    timers.set("setInterval", set_interval_fn)?;
    timers.set("clearInterval", clear_interval_fn)?;
    timers.set("setImmediate", set_immediate_fn)?;
    timers.set("clearImmediate", clear_immediate_fn)?;
    timers.set("queueMicrotask", queue_microtask_fn)?;
    globals.set("timers", timers)?;

    Ok(())
}
