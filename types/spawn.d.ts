/**
 * Child process spawn helper exposed by `vole:spawn`.
 *
 * Unlike `vole:shell`, this API starts a command directly and passes arguments
 * without shell interpolation.
 *
 * @example
 * ```ts
 * import { spawn } from "vole:spawn";
 *
 * const child = spawn("echo", ["hello", "vole"]);
 * console.log(child.pid);
 * ```
 *
 * @remarks
 * The current handle is intentionally minimal. The child is detached by the
 * backend and `kill()` currently reports whether termination was requested, not
 * whether the process has fully exited.
 */
declare module "vole:spawn" {
  /** Spawn options. */
  export interface SpawnOptions {
    /** Working directory used by the child process. */
    cwd?: string;
  }

  /** Minimal detached child process handle. */
  export interface ChildProcess {
    /** Operating-system process id. */
    pid: number;
    /**
     * Request child termination.
     *
     * @param signal Optional signal name. Support is backend/platform dependent.
     * @returns Whether the termination request was accepted by the current backend.
     */
    kill(signal?: string): boolean;
  }

  /**
   * Spawn a child process without shell interpolation.
   *
   * @param command Executable path or name resolved by the host OS.
   * @param args Argument vector passed directly to the child process.
   * @param options Spawn options.
   * @returns Detached child process handle.
   */
  export function spawn(
    command: string,
    args?: readonly string[],
    options?: SpawnOptions,
  ): ChildProcess;
}
