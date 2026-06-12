import {
  __std_socket_connect,
  __std_socket_read,
  __std_socket_write,
  __std_socket_close,
  __std_socket_listen,
  __std_socket_server_accept,
  __std_socket_server_close,
} from "std:raw:socket";

export async function connect(path) {
  const info = await __std_socket_connect(path);
  return {
    _id: info._id,
    path: info.path,
    read: (size) => __std_socket_read(info._id, size),
    write: (data) => __std_socket_write(info._id, data),
    close: () => __std_socket_close(info._id),
  };
}

export async function listen(path) {
  const info = await __std_socket_listen(path);
  return {
    _id: info._id,
    path: info.path,
    accept: () => __std_socket_server_accept(info._id).then(s => ({
      _id: s._id,
      read: (size) => __std_socket_read(s._id, size),
      write: (data) => __std_socket_write(s._id, data),
      close: () => __std_socket_close(s._id),
    })),
    close: () => __std_socket_server_close(info._id),
  };
}

export default { connect, listen };
