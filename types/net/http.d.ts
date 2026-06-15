/**
 * HTTP client/server helpers exposed by `vole:net/http`.
 *
 * The client helpers delegate to global Web `fetch`. The server helper accepts
 * a Web API shaped handler: `Request -> Response | Promise<Response>`.
 *
 * @example
 * ```ts
 * import { serve } from "vole:net/http";
 *
 * const server = serve(async (request) => {
 *   return new Response(`hello from ${new URL(request.url).pathname}`);
 * }, { port: 8080, hostname: "127.0.0.1" });
 *
 * await server.listen();
 * // await server.shutdown();
 * ```
 *
 * @remarks
 * This module is currently a thin shim over the existing backend. It is not a
 * Node `http` compatibility layer. Future versions are expected to lean further
 * into Web API request/response semantics and request-scoped `WaitUntil`.
 */
declare module "vole:net/http" {
  /** Server binding options. */
  export interface ServerOptions {
    /** TCP port to bind. Defaults to `8080`. */
    port?: number;
    /** Hostname or IP address to bind. Defaults to `"0.0.0.0"`. */
    hostname?: string;
  }

  /**
   * Request handler shape used by the current HTTP server shim.
   *
   * @param request Web API `Request` object for the incoming request.
   * @returns Web API `Response`, synchronously or asynchronously.
   */
  export type ServerHandler = (request: Request) => Response | Promise<Response>;

  /** Convenience client request helper backed by global `fetch`. */
  export function request(url: string | URL, options?: RequestInit): Promise<Response>;
  /** Convenience alias for global `fetch`. */
  export function fetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response>;

  /** HTTP server handle returned by `serve`. */
  export class Server {
    /** Create a server handle. Call `listen()` to bind the socket. */
    constructor(handler: ServerHandler, options?: ServerOptions);
    /** Handler invoked for each incoming request. */
    readonly handler: ServerHandler;
    /** Configured TCP port. */
    readonly port: number;
    /** Configured bind hostname. */
    readonly hostname: string;
    /** Start listening. Resolves once the backend server has started. */
    listen(): Promise<void>;
    /** Stop accepting requests and release the backend server. */
    shutdown(): Promise<void>;
  }

  /** Create a server instance. Call `listen()` to start it. */
  export function serve(handler: ServerHandler, options?: ServerOptions): Server;

  /** Default object export containing all HTTP helpers. */
  const http: {
    /** See `request`. */
    request: typeof request;
    /** See `fetch`. */
    fetch: typeof fetch;
    /** HTTP server constructor. */
    Server: typeof Server;
    /** See `serve`. */
    serve: typeof serve;
  };
  export default http;
}
