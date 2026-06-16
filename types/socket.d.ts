/**
 * Unix domain socket helpers exposed by `vole:socket`.
 *
 * This is a low-level byte-stream API. On non-Unix platforms behavior is
 * backend-defined and may reject at runtime.
 */
declare module "vole:socket" {
  /** Connected Unix socket handle. */
  export interface UnixSocket {
    /** Internal native socket id (opaque). */
    readonly _id: number;
    /** Socket path for client-created sockets when available. */
    readonly path?: string;
    /** Read up to `size` bytes from the socket. */
    read(size?: number): Promise<Uint8Array>;
    /** Write bytes and resolve the number of bytes written. */
    write(data: Uint8Array): Promise<number>;
    /** Close the socket. */
    close(): Promise<void>;
  }

  /** Unix socket server handle. */
  export interface UnixSocketServer {
    /** Internal native server id (opaque). */
    readonly _id: number;
    /** Filesystem path the server is bound to. */
    readonly path: string;
    /** Wait for and accept one incoming connection. */
    accept(): Promise<UnixSocket>;
    /** Close the listening socket. */
    close(): Promise<void>;
  }

  /** Connect to a Unix domain socket. */
  export function connect(path: string): Promise<UnixSocket>;
  /** Listen on a Unix domain socket path. */
  export function listen(path: string): Promise<UnixSocketServer>;

  /** Default object export. */
  const socket: {
    connect: typeof connect;
    listen: typeof listen;
  };

  export default socket;
}
