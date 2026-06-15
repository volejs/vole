/**
 * TCP client/server helpers exposed by `vole:net/tcp`.
 *
 * The API exposes small async socket handles with `read`, `write`, and `close`.
 * Reads resolve with `Uint8Array`; writes resolve with the number of bytes written.
 *
 * @example
 * ```ts
 * import { connect, listen } from "vole:net/tcp";
 *
 * const server = await listen(0, "127.0.0.1");
 * console.log(server.address.port);
 *
 * const socket = await connect("127.0.0.1", server.address.port);
 * await socket.write(new TextEncoder().encode("ping"));
 * await socket.close();
 * await server.close();
 * ```
 *
 * @remarks
 * This is a low-level socket API. It does not provide framing, TLS, backpressure
 * abstractions, or stream adapters yet. Build protocol handling explicitly on top
 * of the byte-oriented methods.
 */
declare module "vole:net/tcp" {
  /** Host/port address object returned by listener creation. */
  export interface NetAddress {
    /** Bound host or IP address. */
    host: string;
    /** Bound TCP port. */
    port: number;
  }

  /** Connected TCP socket handle. */
  export interface TcpSocket {
    /** Internal native socket id. Treat as opaque diagnostics only. */
    readonly _id: number;
    /** Read up to `size` bytes from the socket. */
    read(size?: number): Promise<Uint8Array>;
    /** Write bytes to the socket and resolve with the number of bytes written. */
    write(data: Uint8Array): Promise<number>;
    /** Close the socket. */
    close(): Promise<void>;
  }

  /** TCP server handle. */
  export interface TcpServer {
    /** Internal native server id. Treat as opaque diagnostics only. */
    readonly _id: number;
    /** Address selected by the OS after binding. */
    readonly address: NetAddress;
    /** Wait for and accept one incoming connection. */
    accept(): Promise<TcpSocket>;
    /** Close the listening socket. */
    close(): Promise<void>;
  }

  /** Connect to a TCP server. */
  export function connect(host: string, port: number): Promise<TcpSocket>;
  /** Listen for TCP connections. */
  export function listen(port: number, host?: string): Promise<TcpServer>;

  /** Default object export containing all TCP helpers. */
  const tcp: {
    /** See `connect`. */
    connect: typeof connect;
    /** See `listen`. */
    listen: typeof listen;
  };
  export default tcp;
}
