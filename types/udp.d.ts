/**
 * UDP socket helpers exposed by `vole:udp`.
 *
 * `send` writes one datagram; `recv` resolves with one received datagram plus
 * remote address metadata.
 */
declare module "vole:udp" {
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
    /** Internal native socket id (opaque). */
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

  /** Default object export. */
  const udp: {
    createSocket: typeof createSocket;
  };

  export default udp;
}
