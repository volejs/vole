use rquickjs::{
    class::{JsClass, Trace, Tracer, Writable},
    function::{Opt, This},
    Array, Class, Constructor, Ctx, Function, Object, Result, Value,
};

// --- Native EventTarget implementation ---
//
// The EventTarget class is implemented in Rust, but listener state is stored
// in a hidden JS property `_listeners` on each instance. This avoids lifetime
// issues with storing JS Function values in Rust structs and keeps the
// implementation simple while still being a native class.

#[derive(Clone, Debug)]
pub struct EventTarget;

unsafe impl<'js> rquickjs::JsLifetime<'js> for EventTarget {
    type Changed<'to> = EventTarget;
}

impl<'js> Trace<'js> for EventTarget {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

fn ensure_listeners<'js>(
    ctx: &Ctx<'js>,
    this: &This<Class<'js, EventTarget>>,
) -> Result<Object<'js>> {
    let obj = this
        .as_object()
        .expect("This<Class> should always have an object");
    if !obj.contains_key("_listeners")? {
        obj.set("_listeners", Object::new(ctx.clone())?)?;
    }
    obj.get::<&str, Object>("_listeners")
}

fn parse_listener_options(options: Option<Value<'_>>) -> (bool, bool) {
    let mut capture = false;
    let mut once = false;
    if let Some(options) = options {
        if let Some(c) = options.as_bool() {
            capture = c;
        } else if let Some(obj) = options.as_object() {
            if let Ok(v) = obj.get::<&str, bool>("capture") {
                capture = v;
            }
            if let Ok(v) = obj.get::<&str, bool>("once") {
                once = v;
            }
        }
    }
    (capture, once)
}

fn listener_key(type_: &str, capture: bool) -> String {
    if capture {
        format!("{}~capture", type_)
    } else {
        type_.to_string()
    }
}

fn add_event_listener<'js>(
    ctx: Ctx<'js>,
    this: This<Class<'js, EventTarget>>,
    type_: String,
    callback: Function<'js>,
    options: Opt<Value<'js>>,
) -> Result<()> {
    let (capture, once) = parse_listener_options(options.0);
    let listeners = ensure_listeners(&ctx, &this)?;
    let key = listener_key(&type_, capture);
    let list: Array<'js> = if listeners.contains_key(&key)? {
        listeners.get(&key)?
    } else {
        let arr = Array::new(ctx.clone())?;
        listeners.set(key.as_str(), arr.clone())?;
        arr
    };

    let len = list.len();
    for i in 0..len {
        let entry: Object<'js> = match list.get(i)? {
            Some(entry) => entry,
            None => continue,
        };
        let existing: Function<'js> = entry.get("callback")?;
        let existing_capture: bool = entry.get("capture")?;
        if existing == callback && existing_capture == capture {
            return Ok(());
        }
    }

    let entry = Object::new(ctx.clone())?;
    entry.set("callback", callback)?;
    entry.set("capture", capture)?;
    entry.set("once", once)?;
    list.set(len, entry)?;
    Ok(())
}

fn remove_event_listener<'js>(
    ctx: Ctx<'js>,
    this: This<Class<'js, EventTarget>>,
    type_: String,
    callback: Function<'js>,
    options: Opt<Value<'js>>,
) -> Result<()> {
    let (capture, _) = parse_listener_options(options.0);
    let listeners = ensure_listeners(&ctx, &this)?;
    let key = listener_key(&type_, capture);
    if !listeners.contains_key(&key)? {
        return Ok(());
    }
    let list: Array<'js> = listeners.get(&key)?;
    let len = list.len();
    for i in (0..len).rev() {
        let entry: Object<'js> = match list.get(i)? {
            Some(entry) => entry,
            None => continue,
        };
        let existing: Function<'js> = entry.get("callback")?;
        let existing_capture: bool = entry.get("capture")?;
        if existing == callback && existing_capture == capture {
            list.set(i, Value::new_undefined(ctx.clone()))?;
        }
    }
    Ok(())
}

fn dispatch_event<'js>(
    ctx: Ctx<'js>,
    this: This<Class<'js, EventTarget>>,
    event: Object<'js>,
) -> Result<bool> {
    let type_: String = event.get::<&str, String>("type")?;
    event.set("target", this.clone())?;

    let listeners = ensure_listeners(&ctx, &this)?;
    let key = listener_key(&type_, false);
    if listeners.contains_key(&key)? {
        let list: Array<'js> = listeners.get(&key)?;
        let len = list.len();
        let mut to_remove = Vec::new();
        for i in 0..len {
            let entry: Object<'js> = match list.get(i)? {
                Some(entry) => entry,
                None => continue,
            };
            let callback: Function<'js> = entry.get("callback")?;
            let once: bool = entry.get("once")?;
            event.set("currentTarget", this.clone())?;
            callback.call::<_, ()>((event.clone(),))?;
            if once {
                to_remove.push(i);
            }
            let stop_immediate: bool = event.get::<&str, bool>("_stopImmediatePropagation")?;
            if stop_immediate {
                break;
            }
        }
        for i in to_remove.into_iter().rev() {
            list.set(i, Value::new_undefined(ctx.clone()))?;
        }
    }

    let default_prevented: bool = event.get::<&str, bool>("defaultPrevented")?;
    Ok(!default_prevented)
}

fn event_target_ctor<'js>(_ctx: Ctx<'js>) -> Result<EventTarget> {
    Ok(EventTarget)
}

impl<'js> rquickjs::IntoJs<'js> for EventTarget {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(Class::instance(ctx.clone(), self)?.into_value())
    }
}

impl<'js> JsClass<'js> for EventTarget {
    const NAME: &'static str = "EventTarget";
    type Mutable = Writable;

    fn prototype(ctx: &Ctx<'js>) -> Result<Option<Object<'js>>> {
        let proto = Object::new(ctx.clone())?;
        proto.set(
            "addEventListener",
            Function::new(ctx.clone(), add_event_listener)?,
        )?;
        proto.set(
            "removeEventListener",
            Function::new(ctx.clone(), remove_event_listener)?,
        )?;
        proto.set("dispatchEvent", Function::new(ctx.clone(), dispatch_event)?)?;
        Ok(Some(proto))
    }

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        let constr = Constructor::new_class::<EventTarget, _, _>(ctx.clone(), event_target_ctor)?;
        Ok(Some(constr))
    }
}

pub fn dispatch_error_event<'js>(ctx: &Ctx<'js>, error: Value<'js>) -> Result<bool> {
    let dispatch: Function = ctx.globals().get("__voleDispatchError")?;
    dispatch.call((error,))
}

pub fn dispatch_rejection_event<'js>(
    ctx: &Ctx<'js>,
    promise: Value<'js>,
    reason: Value<'js>,
    is_handled: bool,
) -> Result<bool> {
    let dispatch: Function = ctx.globals().get("__voleDispatchRejection")?;
    dispatch.call((promise, reason, is_handled))
}

pub fn init(ctx: &Ctx) -> Result<()> {
    Class::<EventTarget>::define(&ctx.globals())?;

    // JS-side event classes. They extend the native EventTarget.
    ctx.eval::<(), _>(r#"
(function() {
    "use strict";

    class Event {
        constructor(type, options = {}) {
            this.type = String(type);
            this.bubbles = options.bubbles ?? false;
            this.cancelable = options.cancelable ?? false;
            this.composed = options.composed ?? false;
            this.defaultPrevented = false;
            this.timeStamp = typeof performance !== "undefined" ? performance.now() : 0;
            this.target = null;
            this.currentTarget = null;
            this.eventPhase = 0;
            this.isTrusted = false;
            this._stopPropagation = false;
            this._stopImmediatePropagation = false;
        }
        preventDefault() {
            if (this.cancelable) {
                this.defaultPrevented = true;
            }
        }
        stopPropagation() {
            this._stopPropagation = true;
        }
        stopImmediatePropagation() {
            this._stopPropagation = true;
            this._stopImmediatePropagation = true;
        }
        composedPath() {
            return [];
        }
    }

    class CustomEvent extends Event {
        constructor(type, options = {}) {
            super(type, options);
            this.detail = options.detail ?? null;
        }
    }

    class AbortSignal extends EventTarget {
        constructor() {
            super();
            this.aborted = false;
            this.reason = undefined;
            this.onabort = null;
        }
        throwIfAborted() {
            if (this.aborted) {
                throw this.reason;
            }
        }
        static abort(reason) {
            const signal = new AbortSignal();
            signal.aborted = true;
            signal.reason = reason;
            return signal;
        }
        static timeout(milliseconds) {
            const signal = new AbortSignal();
            setTimeout(() => {
                const reason = new DOMException("The operation timed out.", "TimeoutError");
                signal.aborted = true;
                signal.reason = reason;
                signal.dispatchEvent(new Event("abort"));
                if (typeof signal.onabort === "function") {
                    signal.onabort(new Event("abort"));
                }
            }, milliseconds);
            return signal;
        }
        static any(signals) {
            const composite = new AbortSignal();
            for (const signal of signals) {
                if (signal.aborted) {
                    composite.aborted = true;
                    composite.reason = signal.reason;
                    return composite;
                }
                signal.addEventListener("abort", () => {
                    if (composite.aborted) return;
                    composite.aborted = true;
                    composite.reason = signal.reason;
                    composite.dispatchEvent(new Event("abort"));
                    if (typeof composite.onabort === "function") {
                        composite.onabort(new Event("abort"));
                    }
                });
            }
            return composite;
        }
    }

    class AbortController {
        constructor() {
            this.signal = new AbortSignal();
        }
        abort(reason) {
            if (this.signal.aborted) return;
            this.signal.aborted = true;
            this.signal.reason = reason ?? new DOMException("The operation was aborted.", "AbortError");
            const event = new Event("abort");
            this.signal.dispatchEvent(event);
            if (typeof this.signal.onabort === "function") {
                this.signal.onabort(event);
            }
        }
    }

    class MessageEvent extends Event {
        constructor(type, options = {}) {
            super(type, options);
            this.data = options.data ?? null;
            this.ports = options.ports ?? [];
            this.origin = options.origin ?? "";
            this.lastEventId = options.lastEventId ?? "";
            this.source = options.source ?? null;
        }
    }

    class MessagePort extends EventTarget {
        constructor() {
            super();
            this._pair = null;
            this._started = false;
            this._closed = false;
        }
        postMessage(data, transfer = []) {
            if (this._closed || !this._pair) return;
            const ports = transfer.filter(p => p instanceof MessagePort);
            if (this._started) {
                setTimeout(() => {
                    if (this._pair._closed) return;
                    this._pair.dispatchEvent(new MessageEvent("message", { data, ports }));
                }, 0);
            }
        }
        start() {
            this._started = true;
        }
        close() {
            this._closed = true;
            if (this._pair) {
                this._pair._closed = true;
            }
        }
    }

    class MessageChannel {
        constructor() {
            this.port1 = new MessagePort();
            this.port2 = new MessagePort();
            this.port1._pair = this.port2;
            this.port2._pair = this.port1;
        }
    }

    globalThis.Event = Event;
    globalThis.EventTarget = EventTarget;
    globalThis.CustomEvent = CustomEvent;
    globalThis.AbortController = AbortController;
    globalThis.AbortSignal = AbortSignal;
    globalThis.MessageEvent = MessageEvent;
    globalThis.MessagePort = MessagePort;
    globalThis.MessageChannel = MessageChannel;

    // Runtime-level event dispatch helpers used by Rust-side hooks.
    const fallbackReportError = globalThis.reportError;

    function dispatchError(error) {
        const ev = new Event("error", { cancelable: true });
        ev.error = error;
        const canceled = !globalThis.dispatchEvent(ev);
        if (!canceled && fallbackReportError) {
            fallbackReportError(error);
        }
        return canceled;
    }

    function dispatchRejection(promise, reason, isHandled) {
        const type = isHandled ? "rejectionhandled" : "unhandledrejection";
        const ev = new Event(type, { cancelable: true });
        ev.promise = promise;
        ev.reason = reason;
        const canceled = !globalThis.dispatchEvent(ev);
        if (!canceled && type === "unhandledrejection" && fallbackReportError) {
            fallbackReportError(reason);
        }
        return canceled;
    }

    function defineHidden(name, value) {
        Object.defineProperty(globalThis, name, {
            value,
            writable: true,
            enumerable: false,
            configurable: true,
        });
    }

    defineHidden("__voleDispatchError", dispatchError);
    defineHidden("__voleDispatchRejection", dispatchRejection);
    defineHidden("reportError", function(error) {
        dispatchError(error);
    });

    // Make globalThis behave like an EventTarget. The real global object is a plain
    // JS object, so we install method wrappers that delegate to a hidden native
    // EventTarget instance while keeping EventTarget.prototype in the prototype chain.
    const _globalEventTarget = new EventTarget();
    defineHidden("_globalEventTarget", _globalEventTarget);
    defineHidden("addEventListener", function(type, listener, options) {
        return _globalEventTarget.addEventListener(type, listener, options);
    });
    defineHidden("removeEventListener", function(type, listener, options) {
        return _globalEventTarget.removeEventListener(type, listener, options);
    });
    defineHidden("dispatchEvent", function(event) {
        return _globalEventTarget.dispatchEvent(event);
    });
    Object.setPrototypeOf(globalThis, EventTarget.prototype);
})();
    "#)?;
    Ok(())
}
