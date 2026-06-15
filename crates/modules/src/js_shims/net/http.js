import {
  __std_http_serve,
  __std_http_server_shutdown,
} from "vole:raw:net/http";

export async function request(url, options) {
  return fetch(url, options);
}

export async function fetch(input, init) {
  return globalThis.fetch(input, init);
}

// Private handler registry used by the Rust HTTP server backend.
// Kept as module-local state; only the dispatcher below is exposed to Rust
// through a single private global symbol.
const handlers = {};
let nextHandlerId = 1;

function registerHandler(handler) {
  const id = String(nextHandlerId++);
  handlers[id] = handler;
  return id;
}

function unregisterHandler(id) {
  delete handlers[id];
}

// Dispatcher invoked by the Rust backend for each incoming request.
globalThis.__vole_http_dispatcher = async function (handlerId, method, url, headersArray, bodyBuf) {
  const handler = handlers[handlerId];
  if (typeof handler !== "function") {
    throw new Error(`HTTP handler ${handlerId} not found`);
  }

  const headers = new Headers();
  if (Array.isArray(headersArray)) {
    for (const pair of headersArray) {
      if (Array.isArray(pair) && pair.length >= 2) {
        headers.append(pair[0], pair[1]);
      }
    }
  }

  const init = { method, headers };
  if (bodyBuf && bodyBuf.length > 0) {
    init.body = bodyBuf;
  }

  const host = headers.get("host") || "localhost";
  const requestUrl = /^[a-zA-Z][a-zA-Z\d+\-.]*:/.test(url)
    ? url
    : `http://${host}${url.startsWith("/") ? url : `/${url}`}`;
  const request = new Request(requestUrl, init);
  const handlerResult = await handler(request);
  const response = handlerResult instanceof Response ? handlerResult : new Response(handlerResult);

  const respHeaders = [];
  if (response.headers) {
    response.headers.forEach((value, name) => {
      respHeaders.push([name, value]);
    });
  }

  const respBodyBuf = response.body === null
    ? new Uint8Array(0)
    : new Uint8Array(await response.arrayBuffer());

  return {
    status: response.status || 200,
    headers: respHeaders,
    body: respBodyBuf,
  };
};

export class Server {
  constructor(handler, options) {
    if (typeof handler !== "function") {
      throw new TypeError("vole:net/http.Server handler must be a function");
    }
    this.handler = handler;
    this.port = options?.port ?? 8080;
    this.hostname = options?.hostname ?? "0.0.0.0";
    this._id = null;
    this._handlerId = null;
  }

  async listen() {
    this._handlerId = registerHandler(this.handler);
    const info = await __std_http_serve(this.hostname, this.port, this._handlerId);
    this._id = info._id;
  }

  async shutdown() {
    if (this._id !== null) {
      await __std_http_server_shutdown(this._id);
      this._id = null;
    }
    if (this._handlerId !== null) {
      unregisterHandler(this._handlerId);
      this._handlerId = null;
    }
  }
}

export function serve(handler, options) {
  return new Server(handler, options);
}

export default { request, fetch, Server, serve };
