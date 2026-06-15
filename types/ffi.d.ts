/**
 * Foreign Function Interface helpers exposed by `vole:ffi`.
 *
 * The FFI API opens dynamic libraries and calls raw symbols through libffi. It
 * is intentionally low-level: vole does not validate that your JavaScript type
 * list matches the native ABI beyond basic type-name parsing.
 *
 * @example
 * ```ts
 * import { open, define, close } from "vole:ffi";
 *
 * const libc = open("/usr/lib/libSystem.B.dylib");
 * const strlen = define(libc, "strlen", {
 *   parameters: ["cstring"],
 *   result: "usize",
 * });
 *
 * console.log(strlen("vole"));
 * close(libc);
 * ```
 *
 * @remarks
 * Passing the wrong `parameters` or `result` type can crash the process or
 * corrupt memory. Prefer wrapping FFI calls in a narrow, tested module instead
 * of scattering raw symbol calls throughout application code.
 */
declare module "vole:ffi" {
  /**
   * Native value type names understood by vole's libffi bridge.
   *
   * `string` and `cstring` are passed as pointer-like C strings. Integer widths
   * must match the target function's ABI exactly.
   */
  export type FfiType =
    | "void"
    | "bool"
    | "i8"
    | "i16"
    | "i32"
    | "i64"
    | "u8"
    | "u16"
    | "u32"
    | "u64"
    | "f32"
    | "f64"
    | "pointer"
    | "string"
    | "cstring"
    | "usize"
    | "isize";

  /** Signature metadata used when calling or defining a native symbol. */
  export interface FfiCallOptions {
    /** Ordered parameter type list passed to the native function. */
    parameters?: readonly FfiType[];
    /** Native return type. Defaults to `"void"` at runtime. */
    result?: FfiType;
  }

  /** JavaScript values that can cross the FFI boundary. */
  export type FfiValue =
    | null
    | boolean
    | number
    | bigint
    | string
    | ArrayBuffer
    | ArrayBufferView;

  /**
   * Open a dynamic library and return an opaque library handle.
   *
   * @param path Path to a platform dynamic library, such as `.dylib`, `.so`, or `.dll`.
   * @returns Opaque handle used by `call`, `define`, and `close`.
   */
  export function open(path: string): bigint;

  /**
   * Close a previously opened dynamic library handle.
   *
   * @param handle Handle returned by `open`.
   */
  export function close(handle: bigint): void;

  /**
   * Call a native symbol from a library handle.
   *
   * @param handle Library handle returned by `open`.
   * @param symbol Exported symbol name.
   * @param args JavaScript arguments converted according to `options.parameters`.
   * @param options Native function signature metadata.
   * @returns Native return value converted back to JavaScript.
   */
  export function call(
    handle: bigint,
    symbol: string,
    args?: readonly FfiValue[],
    options?: FfiCallOptions,
  ): FfiValue;

  /**
   * Bind a native symbol to a reusable JavaScript function.
   *
   * @param handle Library handle returned by `open`.
   * @param symbol Exported symbol name.
   * @param options Native function signature metadata.
   * @returns JavaScript wrapper that forwards calls to the native symbol.
   */
  export function define(
    handle: bigint,
    symbol: string,
    options?: FfiCallOptions,
  ): (...args: FfiValue[]) => FfiValue;

  /** Default object export containing all FFI helpers. */
  const ffi: {
    /** See `open`. */
    open: typeof open;
    /** See `close`. */
    close: typeof close;
    /** See `call`. */
    call: typeof call;
    /** See `define`. */
    define: typeof define;
  };
  export default ffi;
}
