pub mod abort_adapter;
pub mod abort_controller;
pub mod abort_signal;
pub mod custom_event;
pub mod event;
pub mod event_target;
pub mod message_channel;
pub mod message_event;
pub mod message_port;

use rquickjs::{Ctx, Function, Result, class::Class};

pub use abort_adapter::{AbortControllerRef, AbortSignalRef};
pub use abort_controller::AbortController;
pub use abort_signal::AbortSignal;
pub use custom_event::CustomEvent;
pub use event::Event;
pub use event_target::EventTarget;
pub use message_channel::MessageChannel;
pub use message_event::MessageEvent;
pub use message_port::MessagePort;

pub fn dispatch_error_event<'js>(ctx: &Ctx<'js>, error: rquickjs::Value<'js>) -> Result<bool> {
    let dispatch: Function = ctx.globals().get("__voleDispatchError")?;
    dispatch.call((error,))
}

pub fn dispatch_rejection_event<'js>(
    ctx: &Ctx<'js>,
    promise: rquickjs::Value<'js>,
    reason: rquickjs::Value<'js>,
    is_handled: bool,
) -> Result<bool> {
    let dispatch: Function = ctx.globals().get("__voleDispatchRejection")?;
    dispatch.call((promise, reason, is_handled))
}

pub fn init(ctx: &Ctx) -> Result<()> {
    let globals = ctx.globals();
    Class::<EventTarget>::define(&globals)?;
    Class::<Event>::define(&globals)?;
    Class::<CustomEvent>::define(&globals)?;
    Class::<MessageEvent>::define(&globals)?;
    Class::<AbortSignal>::define(&globals)?;
    Class::<AbortController>::define(&globals)?;
    Class::<MessagePort>::define(&globals)?;
    Class::<MessageChannel>::define(&globals)?;

    // Adjust native prototypes to feel like real Web APIs: turn methods that
    // return property values into getters, add missing setters, and implement
    // static helpers that are easier to express on the JS side.
    ctx.eval::<(), _>(
        r#"
(function() {
    const wrapGetter = (proto, name) => {
        const fn = proto[name];
        Object.defineProperty(proto, name, {
            get: function() { return fn.call(this); },
            enumerable: true,
            configurable: true
        });
    };
    const defineAccessor = (proto, name, getter, setter) => {
        Object.defineProperty(proto, name, {
            get: getter,
            set: setter,
            enumerable: true,
            configurable: true
        });
    };

    wrapGetter(Event.prototype, 'type');
    wrapGetter(Event.prototype, 'bubbles');
    wrapGetter(Event.prototype, 'cancelable');
    wrapGetter(Event.prototype, 'composed');
    wrapGetter(Event.prototype, 'defaultPrevented');
    wrapGetter(Event.prototype, 'timeStamp');
    wrapGetter(Event.prototype, 'eventPhase');
    wrapGetter(Event.prototype, 'isTrusted');
    wrapGetter(Event.prototype, 'target');
    wrapGetter(Event.prototype, 'currentTarget');

    wrapGetter(CustomEvent.prototype, 'type');
    wrapGetter(CustomEvent.prototype, 'detail');

    wrapGetter(MessageEvent.prototype, 'type');
    wrapGetter(MessageEvent.prototype, 'data');
    wrapGetter(MessageEvent.prototype, 'origin');
    wrapGetter(MessageEvent.prototype, 'lastEventId');
    wrapGetter(MessageEvent.prototype, 'source');
    wrapGetter(MessageEvent.prototype, 'ports');
    wrapGetter(MessageEvent.prototype, 'target');
    wrapGetter(MessageEvent.prototype, 'currentTarget');

    wrapGetter(CustomEvent.prototype, 'target');
    wrapGetter(CustomEvent.prototype, 'currentTarget');

    wrapGetter(AbortSignal.prototype, 'aborted');
    wrapGetter(AbortSignal.prototype, 'reason');
    defineAccessor(AbortSignal.prototype, 'onabort',
        function() { return this._onabort === undefined ? null : this._onabort; },
        function(v) { this._onabort = v; }
    );

    AbortSignal.prototype.throwIfAborted = function() {
        if (this.aborted) {
            throw this.reason;
        }
    };

    AbortSignal.timeout = function(milliseconds) {
        const signal = new AbortSignal();
        setTimeout(() => {
            if (signal.aborted) return;
            signal._aborted = true;
            signal._reason = new DOMException("The operation timed out.", "TimeoutError");
            const ev = new Event("abort");
            signal.dispatchEvent(ev);
            if (typeof signal.onabort === "function") {
                signal.onabort(ev);
            }
        }, milliseconds);
        return signal;
    };

    AbortSignal.any = function(signals) {
        const composite = new AbortSignal();
        for (const signal of signals) {
            if (signal.aborted) {
                composite._aborted = true;
                composite._reason = signal.reason;
                return composite;
            }
            signal.addEventListener("abort", () => {
                if (composite.aborted) return;
                composite._aborted = true;
                composite._reason = signal.reason;
                const ev = new Event("abort");
                composite.dispatchEvent(ev);
                if (typeof composite.onabort === "function") {
                    composite.onabort(ev);
                }
            });
        }
        return composite;
    };

    wrapGetter(AbortController.prototype, 'signal');

    defineAccessor(MessagePort.prototype, 'onmessage',
        function() { return this._onmessage === undefined ? null : this._onmessage; },
        function(v) { this._onmessage = v; }
    );

    wrapGetter(MessageChannel.prototype, 'port1');
    wrapGetter(MessageChannel.prototype, 'port2');
})();
    "#,
    )?;

    // Global error/rejection dispatch helpers and EventTarget-style globalThis methods.
    ctx.eval::<(), _>(
        r#"
(function() {
    "use strict";

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
    "#,
    )?;

    Ok(())
}
