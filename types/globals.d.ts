/**
 * Vole-specific global declarations.
 *
 * Standard Web APIs such as `fetch`, `URL`, `Blob`, `ReadableStream`,
 * `CompressionStream`, `crypto`, and `structuredClone` are intentionally not
 * redeclared here. They come from TypeScript's built-in DOM libraries referenced
 * by `index.d.ts`.
 *
 * @example
 * ```ts
 * const handle = setImmediate((name) => {
 *   console.log(`hello ${name}`);
 * }, "vole");
 *
 * clearImmediate(handle);
 * ```
 *
 * @remarks
 * `setImmediate` is a vole timer extension, not part of the WinterTC minimum
 * API set. Prefer promises or `queueMicrotask` for portable Web API code.
 */
declare global {
  /**
   * Schedule a callback to run after the current turn of the event loop.
   *
   * @param handler Callback invoked when the immediate task fires.
   * @param args Arguments forwarded to `handler`.
   * @returns Numeric handle accepted by `clearImmediate`.
   */
  function setImmediate(handler: (...args: unknown[]) => void, ...args: unknown[]): number;

  /**
   * Cancel a callback previously scheduled with `setImmediate`.
   *
   * Calling this with an unknown handle is a no-op.
   */
  function clearImmediate(id: number): void;
}

export {};
