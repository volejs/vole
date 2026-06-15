/**
 * UDP socket helpers exposed by `vole:net/udp`.
 *
 * UDP is message-oriented and connectionless. `send` writes one datagram;
 * `recv` resolves with one received datagram plus its remote address.
 *
 * @example
 * ```ts
 * import { createSocket } from "vole:net/udp";
 *
 * const socket = await createSocket(0, "127.0.0.1");
 * const address = await socket.localAddress();
 * await socket.send(new TextEncoder().encode("ping"), address.host, address.port);
 * const message = await socket.recv();
 * console.log(new TextDecoder().decode(message.data));
 * await socket.close();
 * ```
 *
 * @remarks
 * UDP delivery is not guaranteed and message ordering is not guaranteed. Large
 * datagrams may be dropped or fragmented by the network stack.
 */
declare module "vole:net/udp" {
  /** UDP address metadata. */
  export interface UdpAddress {
    /** IP address string. */
    host: string;
    /** UDP port number. */
    port: number;
    /** Address family. */
    family: "IPv4" | "IPv6";
  }

  /** Message returned by `recv`. */
  export interface UdpMessage {
    /** Datagram payload bytes. */
    data: Uint8Array;
    /** Remote sender address. */
    address: UdpAddress;
  }

  /** Bound UDP socket handle. */
  export interface UdpSocket {
    /** Internal native socket id. Treat as opaque diagnostics only. */
    readonly _id: number;
    /** Address selected at bind time. */
    readonly address: UdpAddress;
    /** Send one datagram to a remote host and port. */
    send(data: Uint8Array, remoteHost: string, remotePort: number): Promise<number>;
    /** Receive one datagram, reading at most `size` bytes. */
    recv(size?: number): Promise<UdpMessage>;
    /** Close the socket. */
    close(): Promise<void>;
    /** Return the current local socket address. */
    localAddress(): Promise<UdpAddress>;
  }

  /** Create and bind a UDP socket. */
  export function createSocket(port?: number, host?: string): Promise<UdpSocket>;

  /** Default object export containing all UDP helpers. */
  const udp: {
    /** See `createSocket`. */
    createSocket: typeof createSocket;
  };
  export default udp;
}
