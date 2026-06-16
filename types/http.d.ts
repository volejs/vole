/**
 * HTTP client/server helpers exposed by `vole:http`.
 *
 * The client helpers delegate to Web `fetch`.
 * The server helper accepts Web API shaped handlers: `Request -> Response | Promise<Response>`.
 */
declare module "vole:http" {
  /** Server binding options. */
  export interface ServerOptions {
    /** TCP port to bind. Defaults to `8080`. */
    port?: number;
    /** Hostname or IP address to bind. Defaults to `0.0.0.0`. */
    hostname?: string;
  }

  /** Request handler shape used by the current HTTP server shim. */
  export type ServerHandler = (request: Request) => Response | Promise<Response>;

  /** Convenience client request helper backed by global `fetch`. */
  export function request(url: string | URL, options?: RequestInit): Promise<Response>;
  /** Convenience alias for global `fetch`. */
  export function fetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response>;

  /** HTTP server handle returned by `serve`. */
  export class Server {
    /** Create a server handle. Call `listen()` to bind socket. */
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

  /** Default export object. */
  const http: {
    request: typeof request;
    fetch: typeof fetch;
    Server: typeof Server;
    serve: typeof serve;
  };

  export default http;
}
