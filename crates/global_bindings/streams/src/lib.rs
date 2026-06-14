use rquickjs::{Class, Ctx, Result};

mod queuing_strategy;
mod readable;
mod transform;
mod utils;
mod writable;

use queuing_strategy::{ByteLengthQueuingStrategy, CountQueuingStrategy};
use readable::{
    ReadableByteStreamController, ReadableStream, ReadableStreamBYOBReader,
    ReadableStreamDefaultController, ReadableStreamDefaultReader,
};
use transform::{TransformStream, TransformStreamDefaultController};
use writable::{WritableStream, WritableStreamDefaultController, WritableStreamDefaultWriter};

pub fn init(ctx: &Ctx) -> Result<()> {
    let globals = ctx.globals();

    // Helpers used by Rust code to avoid re-entrancy when resolving/rejecting
    // promises from inside native callbacks.
    ctx.eval::<(), _>(
        r#"
        globalThis.__vole_promise_then = (promise, onFulfilled, onRejected) =>
            Promise.prototype.then.call(promise, onFulfilled, onRejected);
        globalThis.__vole_resolve_later = (resolve, value) =>
            Promise.resolve(undefined).then(() => resolve(value));
        globalThis.__vole_reject_later = (reject, reason) =>
            Promise.resolve(undefined).then(() => reject(reason));
        globalThis.__vole_schedule_task = (fn) =>
            (typeof queueMicrotask === "function" ? queueMicrotask : setTimeout)(fn, 0);
        globalThis.__vole_byte_length = (chunk) => {
            if (chunk instanceof ArrayBuffer) return chunk.byteLength;
            if (ArrayBuffer.isView(chunk)) return chunk.byteLength;
            throw new TypeError('Chunk must be an ArrayBuffer or ArrayBufferView');
        };
        globalThis.__vole_byob_copy = (chunk, view) => {
            let src;
            if (chunk instanceof ArrayBuffer) {
                src = new Uint8Array(chunk);
            } else if (ArrayBuffer.isView(chunk)) {
                src = new Uint8Array(chunk.buffer, chunk.byteOffset, chunk.byteLength);
            } else {
                throw new TypeError('Chunk must be an ArrayBuffer or ArrayBufferView');
            }
            const dest = new Uint8Array(view.buffer, view.byteOffset, view.byteLength);
            const len = Math.min(src.length, dest.length);
            dest.set(src.subarray(0, len));
            const remainder = src.length > len ? src.subarray(len) : undefined;
            return { consumed: len, remainder };
        };
        globalThis.__vole_byob_respond_view = (view, bytesWritten) => {
            const Ctor = view.constructor;
            return new Ctor(view.buffer, view.byteOffset, bytesWritten);
        };
        globalThis.__vole_byob_same_buffer = (a, b) => a.buffer === b.buffer;
    "#,
    )?;

    Class::<ReadableStream>::define(&globals)?;
    Class::<ReadableStreamDefaultController>::define(&globals)?;
    Class::<ReadableStreamDefaultReader>::define(&globals)?;
    Class::<ReadableByteStreamController>::define(&globals)?;
    Class::<ReadableStreamBYOBReader>::define(&globals)?;

    Class::<WritableStream>::define(&globals)?;
    Class::<WritableStreamDefaultController>::define(&globals)?;
    Class::<WritableStreamDefaultWriter>::define(&globals)?;

    Class::<TransformStream>::define(&globals)?;
    Class::<TransformStreamDefaultController>::define(&globals)?;

    Class::<ByteLengthQueuingStrategy>::define(&globals)?;
    Class::<CountQueuingStrategy>::define(&globals)?;

    // TextEncoderStream/TextDecoderStream are implemented as wrappers around the
    // native TransformStream, which is simpler than reimplementing the streaming
    // encoder state in Rust.
    ctx.eval::<(), _>(
        r#"
        (function() {
            class TextEncoderStream {
                constructor() {
                    this._encoder = new TextEncoder();
                    const self = this;
                    this._transform = new TransformStream({
                        transform(chunk, controller) {
                            const encoded = self._encoder.encode(chunk);
                            controller.enqueue(encoded);
                        },
                    });
                }
                get readable() { return this._transform.readable; }
                get writable() { return this._transform.writable; }
                get encoding() { return "utf-8"; }
            }

            class TextDecoderStream {
                constructor(label = "utf-8", options = {}) {
                    this._decoder = new TextDecoder(label, options);
                    const self = this;
                    this._transform = new TransformStream({
                        transform(chunk, controller) {
                            const decoded = self._decoder.decode(chunk, { stream: true });
                            if (decoded) controller.enqueue(decoded);
                        },
                        flush(controller) {
                            const decoded = self._decoder.decode(new Uint8Array(0), { stream: false });
                            if (decoded) controller.enqueue(decoded);
                        },
                    });
                }
                get readable() { return this._transform.readable; }
                get writable() { return this._transform.writable; }
                get encoding() { return this._decoder.encoding; }
            }

            globalThis.TextEncoderStream = TextEncoderStream;
            globalThis.TextDecoderStream = TextDecoderStream;
        })();

        // Polyfill higher-level stream operations that are easier to implement
        // in JS on top of the native primitives.
        (function() {
            const RSProto = ReadableStream.prototype;
            const asyncIteratorSymbol = typeof Symbol !== 'undefined' && Symbol.asyncIterator
                ? Symbol.asyncIterator
                : '__asyncIterator__';

            RSProto.tee = function() {
                const reader = this.getReader();
                let c1, c2;
                let canceled1 = false;
                let canceled2 = false;
                let reason1, reason2;

                function maybeCancel() {
                    if (canceled1 && canceled2) {
                        reader.cancel(reason1 !== undefined ? reason1 : reason2);
                    }
                }

                async function pump() {
                    try {
                        while (true) {
                            if (canceled1 && canceled2) {
                                reader.cancel(reason1 !== undefined ? reason1 : reason2);
                                return;
                            }
                            const { done, value } = await reader.read();
                            if (done) {
                                if (!canceled1) c1.close();
                                if (!canceled2) c2.close();
                                return;
                            }
                            if (!canceled1) c1.enqueue(value);
                            if (!canceled2) c2.enqueue(value);
                        }
                    } catch (err) {
                        if (!canceled1) { try { c1.error(err); } catch (_) {} }
                        if (!canceled2) { try { c2.error(err); } catch (_) {} }
                    }
                }

                const branch1 = new ReadableStream({
                    start(c) { c1 = c; },
                    pull() {},
                    cancel(reason) {
                        canceled1 = true;
                        reason1 = reason;
                        maybeCancel();
                    }
                });

                const branch2 = new ReadableStream({
                    start(c) { c2 = c; },
                    pull() {},
                    cancel(reason) {
                        canceled2 = true;
                        reason2 = reason;
                        maybeCancel();
                    }
                });

                pump();
                return [branch1, branch2];
            };

            RSProto.pipeTo = async function(dest, options = {}) {
                const {
                    preventClose = false,
                    preventAbort = false,
                    preventCancel = false,
                    signal
                } = options;
                const reader = this.getReader();
                const writer = dest.getWriter();
                let aborted = false;
                let abortListener;

                function isAborted() {
                    return aborted || (signal && signal.aborted);
                }

                function release() {
                    if (signal && abortListener) {
                        signal.removeEventListener('abort', abortListener);
                    }
                    reader.releaseLock();
                    writer.releaseLock();
                }

                async function doAbort(reason) {
                    await Promise.all([
                        preventCancel ? Promise.resolve() : reader.cancel(reason).catch(() => {}),
                        preventAbort ? Promise.resolve() : writer.abort(reason).catch(() => {})
                    ]);
                }

                if (signal) {
                    if (signal.aborted) {
                        aborted = true;
                    } else {
                        abortListener = () => { aborted = true; };
                        signal.addEventListener('abort', abortListener);
                    }
                }

                try {
                    while (true) {
                        if (isAborted()) {
                            const reason = signal ? signal.reason : undefined;
                            await doAbort(reason);
                            throw reason;
                        }
                        let done, value;
                        try {
                            ({ done, value } = await reader.read());
                        } catch (err) {
                            await doAbort(err);
                            throw err;
                        }
                        if (isAborted()) {
                            const reason = signal ? signal.reason : undefined;
                            await doAbort(reason);
                            throw reason;
                        }
                        if (done) {
                            if (!preventClose) await writer.close();
                            return;
                        }
                        try {
                            await writer.write(value);
                        } catch (err) {
                            if (!preventCancel) await reader.cancel(err).catch(() => {});
                            throw err;
                        }
                    }
                } catch (err) {
                    if (!preventAbort) await writer.abort(err).catch(() => {});
                    throw err;
                } finally {
                    release();
                }
            };

            RSProto.pipeThrough = function(transform, options) {
                this.pipeTo(transform.writable, options).catch(() => {});
                return transform.readable;
            };

            RSProto.getIterator = function() {
                const reader = this.getReader();
                const iterator = {
                    next() {
                        return reader.read();
                    },
                    return() {
                        reader.releaseLock();
                        return Promise.resolve({ done: true, value: undefined });
                    },
                    [asyncIteratorSymbol]() {
                        return iterator;
                    }
                };
                return iterator;
            };

            RSProto[asyncIteratorSymbol] = RSProto.getIterator;
        })();
        "#,
    )?;

    Ok(())
}
