/**
 * Unix domain socket helpers exposed by `vole:net/socket`.
 *
 * This module is available as a public API on all platforms, but the current
 * backend only supports Unix domain sockets on Unix-like operating systems.
 * Unsupported platforms reject at runtime.
 *
 * @example
 * ```ts
 * import { listen, connect } from "vole:net/socket";
 *
 * const server = await listen("/tmp/vole.sock");
 * const client = await connect("/tmp/vole.sock");
 * await client.write(new TextEncoder().encode("hello"));
 * await client.close();
 * await server.close();
 * ```
 *
 * @remarks
 * Socket files live in the filesystem namespace. Clean up old socket paths when
 * a process crashes or exits before `close` completes.
 */
declare module "vole:net/socket" {
  /** Connected Unix socket handle. */
  export interface UnixSocket {
    /** Internal native socket id. Treat as opaque diagnostics only. */
    readonly _id: number;
    /** Socket path for client-created sockets when available. */
    readonly path?: string;
    /** Read up to `size` bytes from the socket. */
    read(size?: number): Promise<Uint8Array>;
    /** Write bytes to the socket and resolve with the number of bytes written. */
    write(data: Uint8Array): Promise<number>;
    /** Close the socket. */
    close(): Promise<void>;
  }

  /** Unix socket server handle. */
  export interface UnixSocketServer {
    /** Internal native server id. Treat as opaque diagnostics only. */
    readonly _id: number;
    /** Filesystem path the server is bound to. */
    readonly path: string;
    /** Wait for and accept one incoming connection. */
    accept(): Promise<UnixSocket>;
    /** Close the listening socket. */
    close(): Promise<void>;
  }

  /** Connect to a Unix domain socket. Rejects on unsupported platforms. */
  export function connect(path: string): Promise<UnixSocket>;
  /** Listen on a Unix domain socket path. Rejects on unsupported platforms. */
  export function listen(path: string): Promise<UnixSocketServer>;

  /** Default object export containing all Unix socket helpers. */
  const socket: {
    /** See `connect`. */
    connect: typeof connect;
    /** See `listen`. */
    listen: typeof listen;
  };
  export default socket;
}
