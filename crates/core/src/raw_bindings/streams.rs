use rquickjs::{Ctx, Function, Result, TypedArray};
use rquickjs::module::{Declarations, Exports, ModuleDef};
use std::io::Write;

pub fn init(ctx: &Ctx) -> Result<()> {
    ctx.eval::<(), _>(r#"
(function() {
    "use strict";

    class ReadableStreamDefaultController {
        constructor(stream, strategy = {}) {
            this._stream = stream;
            this._highWaterMark = strategy.highWaterMark ?? 1;
            this._size = strategy.size ?? (() => 1);
        }
        _queueSize() {
            const stream = this._stream;
            if (!stream) return 0;
            let total = 0;
            for (const chunk of stream._chunks) {
                total += this._size(chunk);
            }
            return total;
        }
        _shouldPull() {
            const stream = this._stream;
            if (!stream || stream._closed || stream._errored || stream._pulling) return false;
            return this.desiredSize > 0;
        }
        enqueue(chunk) {
            const stream = this._stream;
            if (!stream || stream._closed || stream._errored) return;
            stream._chunks.push(chunk);
            if (stream._pendingRead) {
                const { resolve } = stream._pendingRead;
                stream._pendingRead = null;
                resolve({ done: false, value: stream._chunks.shift() });
            }
        }
        close() {
            const stream = this._stream;
            if (!stream) return;
            stream._closed = true;
            if (stream._pendingRead) {
                const { resolve } = stream._pendingRead;
                stream._pendingRead = null;
                resolve({ done: true, value: undefined });
            }
            if (stream._closedResolver) {
                stream._closedResolver();
                stream._closedResolver = null;
            }
        }
        error(e) {
            const stream = this._stream;
            if (!stream) return;
            stream._closed = true;
            stream._errored = true;
            stream._error = e;
            if (stream._pendingRead) {
                const { reject } = stream._pendingRead;
                stream._pendingRead = null;
                reject(e);
            }
            if (stream._closedResolver) {
                stream._closedResolver();
                stream._closedResolver = null;
            }
        }
        get desiredSize() {
            return this._highWaterMark - this._queueSize();
        }
    }

    class ReadableStreamDefaultReader {
        constructor(stream) {
            this._stream = stream;
        }
        read() {
            const stream = this._stream;
            if (!stream) {
                return Promise.reject(new TypeError("This reader has been released"));
            }
            if (stream._errored) {
                return Promise.reject(stream._error);
            }
            if (stream._chunks.length > 0) {
                const value = stream._chunks.shift();
                const result = Promise.resolve({ done: false, value });
                if (stream._controller._shouldPull()) {
                    this._tryPull();
                }
                return result;
            }
            if (stream._closed) {
                return Promise.resolve({ done: true, value: undefined });
            }
            if (stream._pendingRead) {
                return Promise.reject(new TypeError("A read is already pending"));
            }
            return new Promise((resolve, reject) => {
                stream._pendingRead = { resolve, reject };
                this._tryPull();
            });
        }
        _tryPull() {
            const stream = this._stream;
            if (!stream || !stream._controller._shouldPull()) return;
            const source = stream._underlyingSource;
            if (typeof source.pull === "function") {
                stream._pulling = true;
                try {
                    const result = source.pull(stream._controller);
                    if (result && typeof result.then === "function") {
                        result.then(
                            () => {
                                stream._pulling = false;
                                if (stream._controller._shouldPull() && !stream._pendingRead) {
                                    this._tryPull();
                                }
                            },
                            (e) => { stream._pulling = false; stream._controller.error(e); }
                        );
                    } else {
                        stream._pulling = false;
                    }
                } catch (e) {
                    stream._pulling = false;
                    stream._controller.error(e);
                }
            }
        }
        releaseLock() {
            if (this._stream) {
                if (this._stream._pendingRead) {
                    const { reject } = this._stream._pendingRead;
                    this._stream._pendingRead = null;
                    reject(new TypeError("Reader released"));
                }
                this._stream._reader = null;
            }
            this._stream = null;
        }
        get closed() {
            const stream = this._stream;
            if (!stream) return Promise.resolve(undefined);
            if (stream._closed || stream._errored) {
                return Promise.resolve(undefined);
            }
            return new Promise(resolve => {
                stream._closedResolver = resolve;
            });
        }
        cancel(reason) {
            const stream = this._stream;
            if (stream) {
                stream._closed = true;
                if (stream._pendingRead) {
                    const { reject } = stream._pendingRead;
                    stream._pendingRead = null;
                    reject(new TypeError("Cancelled"));
                }
                if (stream._closedResolver) {
                    stream._closedResolver();
                    stream._closedResolver = null;
                }
                stream._reader = null;
                this._stream = null;
            }
            return Promise.resolve(undefined);
        }
    }

    class ReadableStream {
        constructor(underlyingSource = {}, strategy = {}) {
            this._underlyingSource = underlyingSource;
            this._controller = new ReadableStreamDefaultController(this, strategy);
            this._reader = null;
            this._chunks = [];
            this._closed = false;
            this._errored = false;
            this._error = undefined;
            this._pendingRead = null;
            this._pulling = false;
            this._closedResolver = null;

            const schedulePull = () => {
                if (this._controller._shouldPull()) {
                    queueMicrotask(() => {
                        if (this._reader) {
                            this._reader._tryPull();
                        } else {
                            // No reader yet; pull internally to fill the queue.
                            this._doPull();
                        }
                    });
                }
            };

            this._doPull = () => {
                if (!this._controller._shouldPull()) return;
                const source = this._underlyingSource;
                if (typeof source.pull === "function") {
                    this._pulling = true;
                    try {
                        const result = source.pull(this._controller);
                        if (result && typeof result.then === "function") {
                            result.then(
                                () => {
                                    this._pulling = false;
                                    schedulePull();
                                },
                                (e) => { this._pulling = false; this._controller.error(e); }
                            );
                        } else {
                            this._pulling = false;
                            schedulePull();
                        }
                    } catch (e) {
                        this._pulling = false;
                        this._controller.error(e);
                    }
                }
            };

            if (typeof underlyingSource.start === "function") {
                try {
                    const result = underlyingSource.start(this._controller);
                    if (result && typeof result.then === "function") {
                        result.then(
                            () => { schedulePull(); },
                            (e) => { this._controller.error(e); }
                        ).catch((e) => { this._controller.error(e); });
                    } else {
                        schedulePull();
                    }
                } catch (e) {
                    this._controller.error(e);
                }
            } else {
                schedulePull();
            }
        }
        getReader() {
            if (this._reader) {
                throw new TypeError("ReadableStream already has a reader");
            }
            this._reader = new ReadableStreamDefaultReader(this);
            return this._reader;
        }
        get locked() {
            return this._reader !== null;
        }
        cancel(reason) {
            this._closed = true;
            if (this._reader) {
                this._reader._stream = null;
                this._reader = null;
            }
            if (this._underlyingSource.cancel) {
                try {
                    this._underlyingSource.cancel(reason);
                } catch (e) {}
            }
            return Promise.resolve(undefined);
        }
    }

    class WritableStreamDefaultController {
        constructor(stream, strategy = {}) {
            this._stream = stream;
            this._highWaterMark = strategy.highWaterMark ?? 1;
            this._size = strategy.size ?? (() => 1);
        }
        error(e) {
            if (this._stream) {
                this._stream._errored = true;
                this._stream._error = e;
                this._stream._rejectPending(e);
            }
        }
    }

    class WritableStreamDefaultWriter {
        constructor(stream) {
            this._stream = stream;
            this._readyResolve = null;
            this._readyPromise = Promise.resolve(undefined);
        }
        get ready() {
            return this._readyPromise;
        }
        _updateReady() {
            const stream = this._stream;
            if (!stream) return;
            if (stream._queueSize < stream._controller._highWaterMark) {
                if (this._readyResolve) {
                    const resolve = this._readyResolve;
                    this._readyResolve = null;
                    this._readyPromise = Promise.resolve(undefined);
                    resolve(undefined);
                }
            } else if (!this._readyResolve) {
                this._readyPromise = new Promise(resolve => {
                    this._readyResolve = resolve;
                });
            }
        }
        write(chunk) {
            const stream = this._stream;
            if (!stream) {
                return Promise.reject(new TypeError("This writer has been released"));
            }
            if (stream._errored) {
                return Promise.reject(stream._error);
            }
            if (stream._closed) {
                return Promise.reject(new TypeError("WritableStream is closed"));
            }
            let resolveRecord, rejectRecord;
            const promise = new Promise((resolve, reject) => {
                resolveRecord = resolve;
                rejectRecord = reject;
            });
            stream._queue.push({ chunk, resolve: resolveRecord, reject: rejectRecord });
            stream._queueSize += stream._controller._size(chunk);
            this._updateReady();
            stream._processQueue();
            return promise;
        }
        close() {
            const stream = this._stream;
            if (!stream) {
                return Promise.reject(new TypeError("This writer has been released"));
            }
            if (stream._errored) {
                return Promise.reject(stream._error);
            }
            if (stream._closePromise) {
                return stream._closePromise;
            }
            stream._closeRequested = true;
            stream._closePromise = new Promise((resolve, reject) => {
                stream._closeResolve = resolve;
                stream._closeReject = reject;
                stream._processQueue();
            });
            return stream._closePromise;
        }
        abort(reason) {
            const stream = this._stream;
            if (!stream) {
                return Promise.reject(new TypeError("This writer has been released"));
            }
            return stream._abort(reason);
        }
        releaseLock() {
            if (this._stream) {
                this._stream._writer = null;
            }
            this._stream = null;
        }
        get closed() {
            const stream = this._stream;
            if (!stream) return Promise.resolve(undefined);
            if (stream._closed || stream._errored) return Promise.resolve(undefined);
            return new Promise(resolve => {
                stream._closedResolver = resolve;
            });
        }
    }

    class WritableStream {
        constructor(underlyingSink = {}, strategy = {}) {
            this._underlyingSink = underlyingSink;
            this._controller = new WritableStreamDefaultController(this, strategy);
            this._writer = null;
            this._closed = false;
            this._errored = false;
            this._error = undefined;
            this._closedResolver = null;
            this._queue = [];
            this._queueSize = 0;
            this._writing = false;
            this._currentWrite = null;
            this._closeRequested = false;
            this._closePromise = null;
            this._closeResolve = null;
            this._closeReject = null;

            if (typeof underlyingSink.start === "function") {
                try {
                    underlyingSink.start(this._controller);
                } catch (e) {
                    this._errored = true;
                    this._error = e;
                }
            }
        }
        _rejectPending(e) {
            if (this._currentWrite) {
                this._currentWrite.reject(e);
                this._currentWrite = null;
            }
            while (this._queue.length > 0) {
                this._queue.shift().reject(e);
            }
            this._queueSize = 0;
            if (this._closeReject) {
                this._closeReject(e);
                this._closeReject = null;
                this._closeResolve = null;
                this._closePromise = null;
            }
            if (this._closedResolver) {
                this._closedResolver();
                this._closedResolver = null;
            }
        }
        _processQueue() {
            if (this._closeRequested && !this._writing && this._queue.length === 0) {
                this._closeRequested = false;
                const finishClose = () => {
                    this._closed = true;
                    if (this._closeResolve) {
                        this._closeResolve(undefined);
                        this._closeResolve = null;
                        this._closeReject = null;
                        this._closePromise = null;
                    }
                    if (this._closedResolver) {
                        this._closedResolver();
                        this._closedResolver = null;
                    }
                };
                const failClose = (e) => {
                    this._errored = true;
                    this._error = e;
                    if (this._closeReject) {
                        this._closeReject(e);
                        this._closeReject = null;
                        this._closeResolve = null;
                        this._closePromise = null;
                    }
                    if (this._closedResolver) {
                        this._closedResolver();
                        this._closedResolver = null;
                    }
                };
                if (typeof this._underlyingSink.close === "function") {
                    try {
                        const result = this._underlyingSink.close(this._controller);
                        if (result && typeof result.then === "function") {
                            result.then(finishClose, failClose);
                        } else {
                            finishClose();
                        }
                    } catch (e) {
                        failClose(e);
                    }
                } else {
                    finishClose();
                }
                return;
            }
            if (this._writing || this._queue.length === 0) return;
            if (this._closed && !this._closeRequested) return;
            this._writing = true;
            const record = this._queue.shift();
            this._currentWrite = record;
            const chunk = record.chunk;
            const done = () => {
                this._currentWrite = null;
                this._queueSize -= this._controller._size(chunk);
                if (this._queueSize < 0) this._queueSize = 0;
                if (this._writer) this._writer._updateReady();
                record.resolve(undefined);
                this._writing = false;
                this._processQueue();
            };
            const fail = (e) => {
                this._currentWrite = null;
                this._errored = true;
                this._error = e;
                record.reject(e);
                this._rejectPending(e);
                this._writing = false;
            };
            if (typeof this._underlyingSink.write === "function") {
                try {
                    const result = this._underlyingSink.write(chunk, this._controller);
                    if (result && typeof result.then === "function") {
                        result.then(done, fail);
                    } else {
                        done();
                    }
                } catch (e) {
                    fail(e);
                }
            } else {
                done();
            }
        }
        _abort(reason) {
            this._closed = true;
            if (this._writer) {
                this._writer._stream = null;
                this._writer = null;
            }
            const error = reason || new TypeError("Aborted");
            this._rejectPending(error);
            if (typeof this._underlyingSink.abort === "function") {
                try {
                    const result = this._underlyingSink.abort(reason);
                    if (result && typeof result.then === "function") {
                        return result;
                    }
                } catch (e) {
                    return Promise.reject(e);
                }
            }
            return Promise.resolve(undefined);
        }
        getWriter() {
            if (this._writer) {
                throw new TypeError("WritableStream already has a writer");
            }
            this._writer = new WritableStreamDefaultWriter(this);
            return this._writer;
        }
        get locked() {
            return this._writer !== null;
        }
        abort(reason) {
            return this._abort(reason);
        }
        close() {
            this._closed = true;
            if (this._writer) {
                return this._writer.close();
            }
            return Promise.resolve(undefined);
        }
    }

    class TransformStreamDefaultController {
        constructor(stream) {
            this._stream = stream;
        }
        enqueue(chunk) {
            if (this._stream && this._stream._readableController) {
                this._stream._readableController.enqueue(chunk);
            }
        }
        terminate() {
            if (this._stream && this._stream._readableController) {
                this._stream._readableController.close();
            }
        }
        error(e) {
            if (this._stream && this._stream._readableController) {
                this._stream._readableController.error(e);
            }
        }
    }

    class TransformStream {
        constructor(transformer = {}, writableStrategy = {}, readableStrategy = {}) {
            this._transformer = transformer;
            this._readableController = null;
            this._writableController = null;
            const self = this;

            this._readable = new ReadableStream({
                start(controller) {
                    self._readableController = controller;
                }
            });

            this._writable = new WritableStream({
                start(controller) {
                    self._writableController = controller;
                },
                write(chunk) {
                    if (typeof self._transformer.transform === "function") {
                        self._transformer.transform(chunk, self._controller);
                    } else {
                        self._controller.enqueue(chunk);
                    }
                },
                close() {
                    if (typeof self._transformer.flush === "function") {
                        self._transformer.flush(self._controller);
                    }
                    if (self._readableController) {
                        self._readableController.close();
                    }
                }
            });

            this._controller = new TransformStreamDefaultController(this);
        }
        get readable() {
            return this._readable;
        }
        get writable() {
            return this._writable;
        }
    }

    class ByteLengthQueuingStrategy {
        constructor(options = {}) {
            this.highWaterMark = options.highWaterMark ?? 0;
        }
        size(chunk) {
            return chunk.byteLength ?? 0;
        }
    }

    class CountQueuingStrategy {
        constructor(options = {}) {
            this.highWaterMark = options.highWaterMark ?? 1;
        }
        size() {
            return 1;
        }
    }

    class TextEncoderStream {
        constructor() {
            this._encoder = new TextEncoder();
            this._pending = new Uint8Array(0);
            const self = this;
            this._transform = new TransformStream({
                transform(chunk, controller) {
                    const encoded = self._encoder.encode(chunk);
                    controller.enqueue(encoded);
                }
            });
        }
        get readable() {
            return this._transform.readable;
        }
        get writable() {
            return this._transform.writable;
        }
        get encoding() {
            return "utf-8";
        }
    }

    class TextDecoderStream {
        constructor(label = "utf-8", options = {}) {
            this._decoder = new TextDecoder(label, options);
            const self = this;
            this._transform = new TransformStream({
                transform(chunk, controller) {
                    const decoded = self._decoder.decode(chunk, { stream: true });
                    if (decoded) {
                        controller.enqueue(decoded);
                    }
                },
                flush(controller) {
                    const decoded = self._decoder.decode(new Uint8Array(0), { stream: false });
                    if (decoded) {
                        controller.enqueue(decoded);
                    }
                }
            });
        }
        get readable() {
            return this._transform.readable;
        }
        get writable() {
            return this._transform.writable;
        }
        get encoding() {
            return this._decoder.encoding;
        }
    }

    globalThis.ReadableStream = ReadableStream;
    globalThis.ReadableStreamDefaultReader = ReadableStreamDefaultReader;
    globalThis.ReadableStreamDefaultController = ReadableStreamDefaultController;
    globalThis.WritableStream = WritableStream;
    globalThis.WritableStreamDefaultWriter = WritableStreamDefaultWriter;
    globalThis.WritableStreamDefaultController = WritableStreamDefaultController;
    globalThis.TransformStream = TransformStream;
    globalThis.TransformStreamDefaultController = TransformStreamDefaultController;
    globalThis.ByteLengthQueuingStrategy = ByteLengthQueuingStrategy;
    globalThis.CountQueuingStrategy = CountQueuingStrategy;
    globalThis.TextEncoderStream = TextEncoderStream;
    globalThis.TextDecoderStream = TextDecoderStream;
})();
"#)?;
    Ok(())
}

fn compression_encode<'js>(
    ctx: Ctx<'js>,
    format: String,
    data: TypedArray<'js, u8>,
) -> Result<TypedArray<'js, u8>> {
    let bytes = if let Some(raw) = data.as_raw() {
        unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }.to_vec()
    } else {
        Vec::new()
    };

    let compressed = match format.as_str() {
        "gzip" => {
            let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(&bytes).map_err(|_| rquickjs::Error::Exception)?;
            encoder.finish().map_err(|_| rquickjs::Error::Exception)?
        }
        "deflate" => {
            let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(&bytes).map_err(|_| rquickjs::Error::Exception)?;
            encoder.finish().map_err(|_| rquickjs::Error::Exception)?
        }
        "deflate-raw" => {
            let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(&bytes).map_err(|_| rquickjs::Error::Exception)?;
            encoder.finish().map_err(|_| rquickjs::Error::Exception)?
        }
        _ => return Err(rquickjs::Error::Exception),
    };

    TypedArray::new_copy(ctx, compressed)
}

fn compression_decode<'js>(
    ctx: Ctx<'js>,
    format: String,
    data: TypedArray<'js, u8>,
) -> Result<TypedArray<'js, u8>> {
    let bytes = if let Some(raw) = data.as_raw() {
        unsafe { std::slice::from_raw_parts(raw.ptr.as_ptr(), raw.len) }.to_vec()
    } else {
        Vec::new()
    };

    let decompressed = match format.as_str() {
        "gzip" => {
            let mut decoder = flate2::read::GzDecoder::new(&bytes[..]);
            let mut out = Vec::new();
            std::io::Read::read_to_end(&mut decoder, &mut out).map_err(|_| rquickjs::Error::Exception)?;
            out
        }
        "deflate" => {
            let mut decoder = flate2::read::ZlibDecoder::new(&bytes[..]);
            let mut out = Vec::new();
            std::io::Read::read_to_end(&mut decoder, &mut out).map_err(|_| rquickjs::Error::Exception)?;
            out
        }
        "deflate-raw" => {
            let mut decoder = flate2::read::DeflateDecoder::new(&bytes[..]);
            let mut out = Vec::new();
            std::io::Read::read_to_end(&mut decoder, &mut out).map_err(|_| rquickjs::Error::Exception)?;
            out
        }
        _ => return Err(rquickjs::Error::Exception),
    };

    TypedArray::new_copy(ctx, decompressed)
}


pub struct CompressionRawModule;

impl ModuleDef for CompressionRawModule {
    fn declare<'js>(decl: &Declarations<'js>) -> rquickjs::Result<()> {
        decl.declare("__compression_encode")?;
        decl.declare("__compression_decode")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> rquickjs::Result<()> {
        exports.export("__compression_encode", Function::new(ctx.clone(), compression_encode))?;
        exports.export("__compression_decode", Function::new(ctx.clone(), compression_decode))?;
        Ok(())
    }
}
