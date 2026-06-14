use reqwest::Client;
use rquickjs::{Ctx, Function, Object, Promise, Result, TypedArray};
use std::sync::OnceLock;

static CLIENT: OnceLock<Client> = OnceLock::new();

fn client() -> &'static Client {
    CLIENT.get_or_init(Client::new)
}

pub fn init(ctx: &Ctx) -> Result<()> {
    let globals = ctx.globals();
    globals.set("_fetch_raw", Function::new(ctx.clone(), fetch_impl)?)?;

    ctx.eval::<(), _>(r#"
(function() {
    "use strict";

    class Headers {
        constructor(init) {
            this._map = new Map();
            if (init) {
                if (typeof Headers !== "undefined" && init instanceof Headers) {
                    init.forEach((v, k) => this._map.set(k, v));
                } else if (Array.isArray(init)) {
                    for (const [k, v] of init) {
                        this._map.set(String(k).toLowerCase(), String(v));
                    }
                } else if (typeof init === "object") {
                    for (const [k, v] of Object.entries(init)) {
                        this._map.set(k.toLowerCase(), String(v));
                    }
                }
            }
        }
        append(name, value) {
            name = String(name).toLowerCase();
            value = String(value);
            if (this._map.has(name)) {
                this._map.set(name, this._map.get(name) + ", " + value);
            } else {
                this._map.set(name, value);
            }
        }
        delete(name) {
            this._map.delete(String(name).toLowerCase());
        }
        get(name) {
            return this._map.get(String(name).toLowerCase()) ?? null;
        }
        getSetCookie() { return []; }
        has(name) {
            return this._map.has(String(name).toLowerCase());
        }
        set(name, value) {
            this._map.set(String(name).toLowerCase(), String(value));
        }
        forEach(callback, thisArg) {
            for (const [k, v] of this._map) {
                callback.call(thisArg, v, k, this);
            }
        }
        entries() { return this._map.entries(); }
        keys() { return this._map.keys(); }
        values() { return this._map.values(); }
        [Symbol.iterator]() { return this._map.entries(); }
    }

    class Request {
        constructor(input, init) {
            init = init ?? {};
            if (input instanceof Request) {
                this.url = input.url;
                this.method = init.method || input.method;
                this.headers = init.headers ? new Headers(init.headers) : new Headers(input.headers);
                this.body = init.body !== undefined ? init.body : input.body;
                this.signal = init.signal || input.signal || null;
            } else {
                this.url = String(input);
                this.method = init.method || "GET";
                this.headers = new Headers(init.headers);
                this.body = init.body;
                this.signal = init.signal || null;
            }
            this.mode = init.mode || "cors";
            this.credentials = init.credentials || "same-origin";
            this.cache = init.cache || "default";
            this.redirect = init.redirect || "follow";
            this.referrer = init.referrer || "about:client";
            this.integrity = init.integrity || "";
        }
        text() { return Promise.resolve(""); }
        json() { return Promise.resolve(null); }
        arrayBuffer() { return Promise.resolve(new ArrayBuffer(0)); }
        blob() { return Promise.resolve(new Blob([])); }
        clone() {
            return new Request(this.url, {
                method: this.method,
                headers: this.headers,
                body: this.body,
                signal: this.signal,
                mode: this.mode,
                credentials: this.credentials,
                cache: this.cache,
                redirect: this.redirect,
                referrer: this.referrer,
                integrity: this.integrity
            });
        }
    }

    class Response {
        constructor(body, init = {}) {
            this.status = init.status ?? 200;
            this.ok = init.ok ?? (this.status >= 200 && this.status < 300);
            this.statusText = init.statusText ?? "";
            this.url = init.url ?? "";
            this.type = init.type ?? "default";
            this.redirected = init.redirected ?? false;
            this.headers = new Headers(init.headers);
            this._bodyStream = body;
            this._bodyBuf = init._bodyBuf;
            this.bodyUsed = false;
        }
        get body() {
            return this._bodyStream;
        }
        _consumeBody() {
            if (this.bodyUsed) {
                throw new TypeError("Body has already been read");
            }
            this.bodyUsed = true;
        }
        text() {
            this._consumeBody();
            if (this._bodyBuf) {
                return Promise.resolve(new TextDecoder().decode(this._bodyBuf));
            }
            return Promise.resolve("");
        }
        json() {
            this._consumeBody();
            if (this._bodyBuf) {
                return Promise.resolve(JSON.parse(new TextDecoder().decode(this._bodyBuf)));
            }
            return Promise.resolve(null);
        }
        arrayBuffer() {
            this._consumeBody();
            if (this._bodyBuf) {
                return Promise.resolve(this._bodyBuf.buffer.slice(this._bodyBuf.byteOffset, this._bodyBuf.byteOffset + this._bodyBuf.byteLength));
            }
            return Promise.resolve(new ArrayBuffer(0));
        }
        blob() {
            this._consumeBody();
            if (this._bodyBuf) {
                return Promise.resolve(new Blob([this._bodyBuf]));
            }
            return Promise.resolve(new Blob([]));
        }
        static error() {
            return new Response(null, { status: 0 });
        }
        static redirect(url, status = 302) {
            if (status < 300 || status > 399) {
                throw new RangeError("Invalid status code");
            }
            return new Response(null, { status, url, headers: { location: url } });
        }
    }

    async function fetch(input, init) {
        init = init ?? {};
        if (typeof input === "object" && input instanceof Request) {
            init = {
                method: init.method || input.method,
                headers: init.headers || input.headers,
                body: init.body !== undefined ? init.body : input.body,
                signal: init.signal || input.signal,
            };
            input = input.url;
        }
        if (init.signal && init.signal.aborted) {
            throw init.signal.reason || new DOMException("The operation was aborted.", "AbortError");
        }
        const url = String(input);
        const method = init.method || "GET";
        const body = init.body;
        const headers = [];
        if (init.headers && typeof init.headers === "object") {
            if (typeof Headers !== "undefined" && init.headers instanceof Headers) {
                init.headers.forEach((v, k) => headers.push([k, v]));
            } else {
                for (const [k, v] of Object.entries(init.headers)) {
                    headers.push([k, String(v)]);
                }
            }
        }
        const result = await _fetch_raw(url, method, headers, body);
        const bodyBuf = result.body;
        const CHUNK_SIZE = 16384;
        const bodyStream = new ReadableStream({
            start(controller) {
                controller._bodyBuf = bodyBuf;
                controller._offset = 0;
            },
            pull(controller) {
                if (controller._offset >= controller._bodyBuf.length) {
                    controller.close();
                    return;
                }
                const end = Math.min(controller._offset + CHUNK_SIZE, controller._bodyBuf.length);
                controller.enqueue(new Uint8Array(controller._bodyBuf.subarray(controller._offset, end)));
                controller._offset = end;
            }
        });
        return new Response(bodyStream, {
            status: result.status,
            ok: result.ok,
            statusText: result.statusText,
            url: result.url,
            type: "basic",
            redirected: false,
            headers: new Headers(result._headers || []),
            _bodyBuf: bodyBuf
        });
    }

    class FormData {
        constructor() {
            this._data = new Map();
        }
        append(name, value, filename) {
            const key = String(name);
            if (!this._data.has(key)) {
                this._data.set(key, []);
            }
            this._data.get(key).push({ value, filename });
        }
        delete(name) {
            this._data.delete(String(name));
        }
        get(name) {
            const items = this._data.get(String(name));
            return items ? items[0].value : null;
        }
        getAll(name) {
            const items = this._data.get(String(name));
            return items ? items.map(i => i.value) : [];
        }
        has(name) {
            return this._data.has(String(name));
        }
        set(name, value, filename) {
            this._data.set(String(name), [{ value, filename }]);
        }
        forEach(callback, thisArg) {
            for (const [key, items] of this._data) {
                for (const item of items) {
                    callback.call(thisArg, item.value, key, this);
                }
            }
        }
        entries() {
            const entries = [];
            for (const [key, items] of this._data) {
                for (const item of items) {
                    entries.push([key, item.value]);
                }
            }
            return entries[Symbol.iterator]();
        }
        keys() { return this._data.keys(); }
        values() {
            const values = [];
            for (const items of this._data.values()) {
                for (const item of items) {
                    values.push(item.value);
                }
            }
            return values[Symbol.iterator]();
        }
        [Symbol.iterator]() { return this.entries(); }
    }

    globalThis.Headers = Headers;
    globalThis.Request = Request;
    globalThis.Response = Response;
    globalThis.FormData = FormData;
    globalThis.fetch = fetch;
})();
"#)?;

    Ok(())
}

fn fetch_impl<'js>(
    ctx: Ctx<'js>,
    url: String,
    method: Option<String>,
    headers: Option<Vec<Vec<String>>>,
    body: Option<String>,
) -> Result<Promise<'js>> {
    let (promise, resolve, reject) = ctx.promise()?;
    let ctx_clone = ctx.clone();

    vole_global_timers::task_begin();

    ctx.spawn(async move {
        struct TaskGuard;
        impl Drop for TaskGuard {
            fn drop(&mut self) {
                vole_global_timers::task_end();
            }
        }
        let _guard = TaskGuard;
        let method = method.unwrap_or_else(|| "GET".to_string());
        let method = method.parse().unwrap_or(reqwest::Method::GET);

        let mut req = client().request(method, &url);

        if let Some(headers) = headers {
            for pair in headers {
                if pair.len() >= 2 {
                    req = req.header(&pair[0], &pair[1]);
                }
            }
        }

        if let Some(body) = body {
            req = req.body(body);
        }

        match req.send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let ok = status >= 200 && status < 300;
                let status_text = resp.status().canonical_reason().unwrap_or("").to_string();
                let url = resp.url().to_string();
                let header_entries: Vec<Vec<String>> = resp
                    .headers()
                    .iter()
                    .map(|(k, v)| vec![k.to_string(), v.to_str().unwrap_or("").to_string()])
                    .collect();
                match resp.bytes().await {
                    Ok(bytes) => {
                        let result = Object::new(ctx_clone.clone()).unwrap();
                        let _ = result.set("status", status);
                        let _ = result.set("ok", ok);
                        let _ = result.set("statusText", status_text);
                        let _ = result.set("url", url);
                        let _ = result.set(
                            "body",
                            TypedArray::new_copy(ctx_clone.clone(), bytes.to_vec()).unwrap(),
                        );
                        let _ = result.set("_headers", header_entries);
                        let _ = resolve.call::<_, ()>((result,));
                    }
                    Err(e) => {
                        eprintln!("fetch body error: {}", e);
                        let _ = reject.call::<_, ()>((ctx_clone.catch(),));
                    }
                }
            }
            Err(e) => {
                eprintln!("fetch error: {}", e);
                let _ = reject.call::<_, ()>((ctx_clone.catch(),));
            }
        }
    });

    Ok(promise)
}
