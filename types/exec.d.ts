/**
 * Shell execution and process spawn helpers exposed by `vole:exec`.
 *
 * - `exec` / `execSync`: execute command lines through shell.
 * - `spawn`: start a child process and return its pid + kill handle.
 */
declare module "vole:exec" {
  /** Options for shell-style execution helpers. */
  export interface ExecOptions {
    /** Working directory for the command. */
    cwd?: string;
  }

  /** Result shape returned by `exec`/`execSync`. */
  export interface ExecResult {
    /** Captured stdout text. */
    stdout: string;
    /** Captured stderr text. */
    stderr: string;
    /** Child exit status object. */
    status: {
      /** Whether process exit code is zero. */
      success: boolean;
      /** Numeric exit code. */
      code: number;
    };
  }

  /** Lightweight child process handle returned by `spawn`. */
  export interface ChildProcess {
    /** Native process id. */
    readonly pid: number;
    /** Send optional signal to the process. */
    kill(signal?: string): boolean;
  }

  /** Execute command line in shell synchronously. */
  export function exec(command: string, options?: ExecOptions): ExecResult;
  /** Same as `exec`, with a Node-like naming alias. */
  export function execSync(command: string, options?: ExecOptions): ExecResult;
  /** Spawn a child process asynchronously through shell/process APIs and return handle. */
  export function spawn(command: string, args?: readonly string[], options?: ExecOptions): ChildProcess;

  /** Default object export for convenience. */
  const exec: {
    exec: typeof exec;
    execSync: typeof execSync;
    spawn: typeof spawn;
  };

  export default exec;
}
