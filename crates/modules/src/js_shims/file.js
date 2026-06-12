import {
  __std_file_open,
  __std_file_read,
  __std_file_write,
  __std_file_seek,
  __std_file_close,
} from "std:raw:file";

export async function open(path, options) {
  const info = await __std_file_open(path, options ?? {});
  return {
    _id: info._id,
    read: (size) => __std_file_read(info._id, size),
    write: (data) => __std_file_write(info._id, data),
    seek: (offset, whence) => __std_file_seek(info._id, offset, whence ?? "start"),
    close: () => __std_file_close(info._id),
  };
}
