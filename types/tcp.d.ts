/**
 * TCP client/server helpers exposed by `vole:tcp`.
 *
 * The API exposes simple byte-oriented sockets with `read`, `write`, and
 * `close` methods.
 */
declare module "vole:tcp" {
  /** Host/port address object returned by listener creation. */
  export interface NetAddress {
    /** Bound host or IP address. */
    host: string;
    /** Bound TCP port. */
    port: number;
  }

  /** Connected TCP socket handle. */
  export interface TcpSocket {
    /** Internal native socket id (opaque). */
    readonly _id: number;
    /** Read up to `size` bytes from the socket. */
    read(size?: number): Promise<Uint8Array>;
    /** Write bytes and resolve the number of bytes written. */
    write(data: Uint8Array): Promise<number>;
    /** Close the socket. */
    close(): Promise<void>;
  }

  /** TCP server handle. */
  export interface TcpServer {
    /** Internal native server id (opaque). */
    readonly _id: number;
    /** Address selected by OS after binding. */
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

  /** Default object export. */
  const tcp: {
    connect: typeof connect;
    listen: typeof listen;
  };

  export default tcp;
}
