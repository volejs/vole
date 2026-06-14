use rquickjs::{Ctx, Function, Result};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static ACTIVE_IDS: OnceLock<Arc<Mutex<HashSet<u64>>>> = OnceLock::new();
static ACTIVE_TASKS: AtomicUsize = AtomicUsize::new(0);

pub fn init_registry() {
    ACTIVE_IDS.set(Arc::new(Mutex::new(HashSet::new()))).ok();
}

pub fn task_begin() {
    ACTIVE_TASKS.fetch_add(1, Ordering::SeqCst);
}

pub fn task_end() {
    ACTIVE_TASKS.fetch_sub(1, Ordering::SeqCst);
}

pub fn is_idle() -> bool {
    let timers_idle = ACTIVE_IDS
        .get()
        .map(|c| c.lock().unwrap().is_empty())
        .unwrap_or(true);
    let tasks_idle = ACTIVE_TASKS.load(Ordering::SeqCst) == 0;
    timers_idle && tasks_idle
}

fn add_id(id: u64) {
    if let Some(c) = ACTIVE_IDS.get() {
        c.lock().unwrap().insert(id);
    }
}

fn remove_id(id: u64) {
    if let Some(c) = ACTIVE_IDS.get() {
        c.lock().unwrap().remove(&id);
    }
}

fn is_active(id: u64) -> bool {
    ACTIVE_IDS
        .get()
        .map(|c| c.lock().unwrap().contains(&id))
        .unwrap_or(false)
}

pub fn init(ctx: &Ctx) -> Result<()> {
    let globals = ctx.globals();
    globals.set("_setTimeout", Function::new(ctx.clone(), set_timeout_impl)?)?;
    globals.set(
        "_clearTimeout",
        Function::new(ctx.clone(), clear_timeout_impl)?,
    )?;
    globals.set(
        "_setInterval",
        Function::new(ctx.clone(), set_interval_impl)?,
    )?;
    globals.set(
        "_clearInterval",
        Function::new(ctx.clone(), clear_interval_impl)?,
    )?;
    globals.set(
        "queueMicrotask",
        Function::new(ctx.clone(), queue_microtask_impl)?,
    )?;

    ctx.eval::<(), _>(
        r#"
        (function() {
            globalThis.setTimeout = function(callback, delay, ...args) {
                if (typeof callback !== 'function') {
                    throw new TypeError('setTimeout callback must be a function');
                }
                return _setTimeout(args.length > 0
                    ? function() { callback(...args); }
                    : callback, delay);
            };
            globalThis.clearTimeout = function(id) {
                _clearTimeout(id);
            };
            globalThis.setInterval = function(callback, delay, ...args) {
                if (typeof callback !== 'function') {
                    throw new TypeError('setInterval callback must be a function');
                }
                return _setInterval(args.length > 0
                    ? function() { callback(...args); }
                    : callback, delay);
            };
            globalThis.clearInterval = function(id) {
                _clearInterval(id);
            };
        })();
    "#,
    )?;
    Ok(())
}

fn set_timeout_impl<'js>(ctx: Ctx<'js>, callback: Function<'js>, delay: Option<u64>) -> u64 {
    let delay = delay.unwrap_or(0);
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    add_id(id);
    let cb = callback.clone();
    ctx.spawn(async move {
        tokio::time::sleep(Duration::from_millis(delay)).await;
        if is_active(id) {
            let _ = cb.call::<_, ()>(());
        }
        remove_id(id);
    });
    id
}

fn clear_timeout_impl(id: u64) {
    remove_id(id);
}

fn set_interval_impl<'js>(ctx: Ctx<'js>, callback: Function<'js>, delay: Option<u64>) -> u64 {
    let delay = delay.unwrap_or(0).max(1);
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    add_id(id);
    let cb = callback.clone();
    ctx.spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(delay));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        while is_active(id) {
            interval.tick().await;
            if !is_active(id) {
                break;
            }
            let _ = cb.call::<_, ()>(());
        }
        remove_id(id);
    });
    id
}

fn clear_interval_impl(id: u64) {
    remove_id(id);
}

fn queue_microtask_impl<'js>(ctx: Ctx<'js>, callback: Function<'js>) {
    let cb = callback.clone();
    ctx.spawn(async move {
        let _ = cb.call::<_, ()>(());
    });
}
