import {
  __wasm_compile,
  __wasm_instantiate,
  __wasm_instance_exports,
  __wasm_call_function,
  __wasm_memory_read,
  __wasm_memory_write,
} from "std:raw:wasm";

class CompileError extends Error {
  constructor(message) {
    super(message);
    this.name = "CompileError";
  }
}

class RuntimeError extends Error {
  constructor(message) {
    super(message);
    this.name = "RuntimeError";
  }
}

class Module {
  constructor(bytes) {
    if (!(bytes instanceof Uint8Array || bytes instanceof ArrayBuffer)) {
      throw new TypeError("WebAssembly.Module bytes must be a Uint8Array or ArrayBuffer");
    }
    const data = bytes instanceof ArrayBuffer ? new Uint8Array(bytes) : bytes;
    this._id = __wasm_compile(data);
  }
}

class Instance {
  constructor(module, importObject = {}) {
    if (!(module instanceof Module)) {
      throw new TypeError("First argument must be a WebAssembly.Module");
    }
    this._id = __wasm_instantiate(module._id, importObject);
    this.exports = {};
    const info = __wasm_instance_exports(this._id);
    for (let i = 0; i < info.functions.length; i++) {
      const fn = info.functions[i];
      this.exports[fn.name] = (...args) => {
        return __wasm_call_function(this._id, fn.name, args);
      };
    }
    for (let i = 0; i < info.memories.length; i++) {
      const mem = info.memories[i];
      this.exports[mem.name] = new Memory(this._id, mem.name);
    }
  }
}

class Memory {
  constructor(instanceId, name) {
    this._instanceId = instanceId;
    this._name = name;
  }
  read(offset, length) {
    return __wasm_memory_read(this._instanceId, this._name, offset, length);
  }
  write(offset, data) {
    __wasm_memory_write(this._instanceId, this._name, offset, data);
  }
}

const WebAssembly = {
  Module,
  Instance,
  Memory,
  CompileError,
  RuntimeError,
  instantiate(bytes, importObject) {
    return Promise.resolve(new Instance(new Module(bytes), importObject));
  },
  compile(bytes) {
    return Promise.resolve(new Module(bytes));
  },
};

globalThis.WebAssembly = WebAssembly;

export default WebAssembly;
export { Module, Instance, Memory, CompileError, RuntimeError };
