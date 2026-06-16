import {
  __std_udp_create_socket,
  __std_udp_socket_send,
  __std_udp_socket_recv,
  __std_udp_socket_close,
  __std_udp_socket_local_address,
} from "vole:raw:udp";

export async function createSocket(port, host) {
  const info = await __std_udp_create_socket(port, host);
  return {
    _id: info._id,
    address: info.address,
    send: (data, remoteHost, remotePort) =>
      __std_udp_socket_send(info._id, data, remoteHost, remotePort),
    recv: (size) => __std_udp_socket_recv(info._id, size),
    close: () => __std_udp_socket_close(info._id),
    localAddress: () => __std_udp_socket_local_address(info._id),
  };
}

export default { createSocket };
