/**
 * Shell command helpers exposed by `vole:shell`.
 *
 * Commands are executed through the platform shell, which means shell expansion,
 * quoting, and injection risks follow the host operating system.
 *
 * @example
 * ```ts
 * import { exec, execSync } from "vole:shell";
 *
 * const asyncOut = await exec("printf hello");
 * console.log(asyncOut.stdout);
 *
 * const syncOut = execSync("pwd", { cwd: "/tmp" });
 * console.log(syncOut.status.code);
 * ```
 *
 * @remarks
 * Do not concatenate untrusted input into `command`. Use `vole:spawn` when
 * arguments should be passed without shell interpolation.
 */
declare module "vole:shell" {
  /** Exit status returned by shell command helpers. */
  export interface CommandStatus {
    /** Whether the command exited successfully. */
    success: boolean;
    /** Numeric exit code, or `-1` when the platform did not provide one. */
    code: number;
  }

  /** Captured stdout, stderr, and status for a command. */
  export interface CommandOutput {
    /** UTF-8 decoded standard output. */
    stdout: string;
    /** UTF-8 decoded standard error. */
    stderr: string;
    /** Exit status metadata. */
    status: CommandStatus;
  }

  /** Command execution options. */
  export interface ExecOptions {
    /** Working directory used by the spawned shell. */
    cwd?: string;
  }

  /** Execute a command through the platform shell asynchronously. */
  export function exec(command: string, options?: ExecOptions): Promise<CommandOutput>;
  /** Execute a command through the platform shell synchronously. */
  export function execSync(command: string, options?: ExecOptions): CommandOutput;
}
