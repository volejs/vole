import {
  __std_tcp_connect,
  __std_tcp_socket_read,
  __std_tcp_socket_write,
  __std_tcp_socket_close,
  __std_tcp_listen,
  __std_tcp_server_accept,
  __std_tcp_server_close,
} from "vole:raw:tcp";

export async function connect(host, port) {
  const info = await __std_tcp_connect(host, port);
  return {
    _id: info._id,
    read: (size) => __std_tcp_socket_read(info._id, size),
    write: (data) => __std_tcp_socket_write(info._id, data),
    close: () => __std_tcp_socket_close(info._id),
  };
}

export async function listen(port, host) {
  const info = await __std_tcp_listen(port, host);
  return {
    _id: info._id,
    address: info.address,
    accept: () => __std_tcp_server_accept(info._id).then(s => ({
      _id: s._id,
      read: (size) => __std_tcp_socket_read(s._id, size),
      write: (data) => __std_tcp_socket_write(s._id, data),
      close: () => __std_tcp_socket_close(s._id),
    })),
    close: () => __std_tcp_server_close(info._id),
  };
}

export default { connect, listen };
