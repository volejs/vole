/**
 * Process information and stdio helpers exposed by `vole:process`.
 *
 * This module provides process-level metadata and minimal stdout/stderr writers.
 * It is intentionally smaller than Node's `process` object.
 *
 * @example
 * ```ts
 * import { cwd, chdir, platform, stdout } from "vole:process";
 *
 * stdout.write(`running on ${platform} in ${cwd()}\n`);
 * chdir("..");
 * ```
 *
 * @remarks
 * `exit` and `abort` terminate the current process and do not give pending
 * managed `WaitUntil` scopes a chance to drain. Prefer returning from the entry
 * point normally when graceful shutdown matters.
 */
declare module "vole:process" {
  /** Snapshot of process memory counters. */
  export interface MemoryUsage {
    /** Resident set size in bytes. */
    rss: number;
    /** Total heap size in bytes, when available. */
    heapTotal: number;
    /** Used heap size in bytes, when available. */
    heapUsed: number;
    /** External/native allocation counter in bytes, when available. */
    external: number;
    /** Additional runtime-specific counters. */
    [key: string]: number;
  }

  /** Minimal writable stream-like object used for stdout and stderr. */
  export interface WritableStreamLike {
    /** Write text synchronously. */
    write(data: string): void;
    /** Write text asynchronously. */
    writeAsync(data: string): void;
    /** Flush buffered output if the backend supports flushing. */
    flush(): void;
  }

  /** Exit the current process with an optional status code. */
  export function exit(code?: number): never;
  /** Abort the current process immediately. */
  export function abort(): never;
  /** Return the current working directory. */
  export function cwd(): string;
  /** Change the current working directory. */
  export function chdir(path: string): void;
  /** Return process uptime in seconds. */
  export function uptime(): number;
  /** Return memory usage counters. */
  export function memoryUsage(): MemoryUsage;

  /** CLI arguments passed to the current process. */
  export const args: string[];
  /** Original executable argument, when available. */
  export const argv0: string;
  /** Current process id. */
  export const pid: number;
  /** Parent process id. */
  export const ppid: number;
  /** Platform identifier reported by vole. */
  export const platform: string;
  /** CPU architecture identifier reported by vole. */
  export const arch: string;
  /** Vole version string. */
  export const version: string;
  /** Version map for vole and selected dependencies. */
  export const versions: Record<string, string>;
  /** Standard output writer. */
  export const stdout: WritableStreamLike;
  /** Standard error writer. */
  export const stderr: WritableStreamLike;
}
