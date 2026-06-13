import { __ffi_open, __ffi_close, __ffi_call } from "vole:raw:ffi";

const handles = new Map();

export function open(path) {
  const handle = __ffi_open(path);
  const id = BigInt(handle);
  handles.set(id, true);
  return id;
}

export function close(handle) {
  if (!handles.has(handle)) throw new Error("Invalid FFI library handle");
  handles.delete(handle);
  return __ffi_close(Number(handle));
}

export function call(handle, symbol, args, options) {
  if (!handles.has(handle)) throw new Error("Invalid FFI library handle");
  const parameters = (options && options.parameters) || [];
  const result = (options && options.result) || "void";
  return __ffi_call(Number(handle), symbol, parameters, result, args || []);
}

export function define(handle, symbol, options) {
  if (!handles.has(handle)) throw new Error("Invalid FFI library handle");
  const parameters = (options && options.parameters) || [];
  const result = (options && options.result) || "void";
  return function (...args) {
    return __ffi_call(Number(handle), symbol, parameters, result, args);
  };
}

export default { open, close, call, define };
