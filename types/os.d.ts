/**
 * OS / process metadata and stdio helpers exposed by `vole:os`.
 *
 * This module is intentionally small and focuses on process/runtime metadata,
 * exit control, and stdout/stderr writers.
 *
 * @example
 * ```ts
 * import { cwd, chdir, pid, platform, exit } from "vole:os";
 *
 * console.log(`pid=${pid}, cwd=${cwd()}`);
 * chdir("/tmp");
 * if (pid % 2 === 0) {
 *   exit(1);
 * }
 * ```
 */
declare module "vole:os" {
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
    /** Write text asynchronously (implemented with compatible sync semantics currently). */
    writeAsync(data: string): void;
    /** Flush buffered output if backend supports flushing. */
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
  /** Read the pending process exit code (defaults to `0`). */
  export function getExitCode(): number;
  /** Set the process exit code used when no explicit exit value is provided. */
  export function setExitCode(code: number): void;

  /** CLI arguments passed to current process (excluding executable). */
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
