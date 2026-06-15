/**
 * WebAssembly shim module exposed by `vole:wasm`.
 *
 * The runtime also exposes a global `WebAssembly` object. This module provides
 * the same current shim classes as importable values for code that prefers
 * explicit module imports.
 *
 * @example
 * ```ts
 * import { compile, instantiate } from "vole:wasm";
 *
 * const bytes = await fetch("module.wasm").then((r) => r.arrayBuffer());
 * const module = await compile(bytes);
 * const instance = await instantiate(bytes, {});
 * console.log(module, instance.exports);
 * ```
 *
 * @remarks
 * Vole's WebAssembly support is partial and currently backed by `wasmi`. Some
 * browser/Node WebAssembly APIs, such as streaming compilation, may be missing or
 * behave differently.
 */
declare module "vole:wasm" {
  /** WebAssembly compile error class exposed by vole's shim. */
  export class CompileError extends Error {}
  /** WebAssembly runtime error class exposed by vole's shim. */
  export class RuntimeError extends Error {}

  /** Compiled WebAssembly module wrapper. */
  export class Module {
    /** Compile WebAssembly bytes into a module wrapper. */
    constructor(bytes: Uint8Array | ArrayBuffer);
  }

  /** Instantiated WebAssembly module wrapper. */
  export class Instance {
    /** Instantiate a compiled module with an optional import object. */
    constructor(module: Module, importObject?: WebAssembly.Imports);
    /** Exported functions and memories. */
    readonly exports: WebAssembly.Exports;
  }

  /** Exported WebAssembly memory helper used by vole's current shim. */
  export class Memory {
    /** Read bytes from memory. */
    read(offset: number, length: number): Uint8Array;
    /** Write bytes into memory. */
    write(offset: number, data: Uint8Array): void;
  }

  /** Compile and instantiate bytes in one step. */
  export function instantiate(
    bytes: Uint8Array | ArrayBuffer,
    importObject?: WebAssembly.Imports,
  ): Promise<Instance>;

  /** Compile WebAssembly bytes into a `Module`. */
  export function compile(bytes: Uint8Array | ArrayBuffer): Promise<Module>;

  /** Default object export containing WebAssembly shim classes and helpers. */
  const wasm: {
    /** Module constructor. */
    Module: typeof Module;
    /** Instance constructor. */
    Instance: typeof Instance;
    /** Memory helper constructor. */
    Memory: typeof Memory;
    /** CompileError constructor. */
    CompileError: typeof CompileError;
    /** RuntimeError constructor. */
    RuntimeError: typeof RuntimeError;
    /** See `instantiate`. */
    instantiate: typeof instantiate;
    /** See `compile`. */
    compile: typeof compile;
  };
  export default wasm;
}
