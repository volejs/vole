import {
  __compression_encode,
  __compression_decode,
} from "std:raw:compression";

export class CompressionStream {
  constructor(format = "deflate") {
    this._format = format;
    const self = this;
    this._transform = new TransformStream({
      transform(chunk, controller) {
        const encoded = __compression_encode(self._format, chunk);
        controller.enqueue(encoded);
      },
    });
  }
  get readable() {
    return this._transform.readable;
  }
  get writable() {
    return this._transform.writable;
  }
}

export class DecompressionStream {
  constructor(format = "deflate") {
    this._format = format;
    const self = this;
    this._transform = new TransformStream({
      transform(chunk, controller) {
        const decoded = __compression_decode(self._format, chunk);
        controller.enqueue(decoded);
      },
    });
  }
  get readable() {
    return this._transform.readable;
  }
  get writable() {
    return this._transform.writable;
  }
}

export default { CompressionStream, DecompressionStream };
