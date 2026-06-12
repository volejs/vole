use rquickjs::{Ctx, Result};

pub fn init(ctx: &Ctx) -> Result<()> {
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

    class EventTarget {
        constructor() {
            this._listeners = new Map();
        }
        addEventListener(type, callback, options = {}) {
            if (typeof callback !== "function") return;
            const opts = typeof options === "boolean" ? { capture: options } : options;
            const key = type + (opts.capture ? "~capture" : "");
            if (!this._listeners.has(key)) {
                this._listeners.set(key, []);
            }
            const list = this._listeners.get(key);
            if (!list.some(l => l.callback === callback && l.capture === !!opts.capture)) {
                list.push({ callback, capture: !!opts.capture, once: opts.once ?? false });
            }
        }
        removeEventListener(type, callback, options = {}) {
            if (typeof callback !== "function") return;
            const opts = typeof options === "boolean" ? { capture: options } : options;
            const key = type + (opts.capture ? "~capture" : "");
            const list = this._listeners.get(key);
            if (!list) return;
            const idx = list.findIndex(l => l.callback === callback && l.capture === !!opts.capture);
            if (idx >= 0) list.splice(idx, 1);
        }
        dispatchEvent(event) {
            event.target = this;
            const key = event.type;
            const list = this._listeners.get(key);
            if (list) {
                const snapshot = [...list];
                for (const listener of snapshot) {
                    if (listener.capture) continue;
                    event.currentTarget = this;
                    listener.callback.call(this, event);
                    if (listener.once) {
                        this.removeEventListener(event.type, listener.callback);
                    }
                    if (event._stopImmediatePropagation) break;
                }
            }
            return !event.defaultPrevented;
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
})();
"#)?;
    Ok(())
}
