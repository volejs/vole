import {
  __std_process_exit,
  __std_process_abort,
  __std_process_cwd,
  __std_process_chdir,
  __std_process_uptime,
  __std_process_memoryUsage,
  __std_process_args,
  __std_process_argv0,
  __std_process_pid,
  __std_process_ppid,
  __std_process_platform,
  __std_process_arch,
  __std_process_version,
  __std_process_versions,
  __std_process_stdout_write,
  __std_process_stdout_writeAsync,
  __std_process_stdout_flush,
  __std_process_stderr_write,
  __std_process_stderr_writeAsync,
  __std_process_stderr_flush,
} from "vole:raw:process";

export function exit(code) {
  return __std_process_exit(code);
}
export function abort() {
  return __std_process_abort();
}
export function cwd() {
  return __std_process_cwd();
}
export function chdir(path) {
  return __std_process_chdir(path);
}
export function uptime() {
  return __std_process_uptime();
}
export function memoryUsage() {
  return __std_process_memoryUsage();
}
export const args = __std_process_args();
export const argv0 = __std_process_argv0();
export const pid = __std_process_pid();
export const ppid = __std_process_ppid();
export const platform = __std_process_platform();
export const arch = __std_process_arch();
export const version = __std_process_version();
export const versions = __std_process_versions();

export const stdout = {
  write(data) { return __std_process_stdout_write(data); },
  writeAsync(data) { return __std_process_stdout_writeAsync(data); },
  flush() { return __std_process_stdout_flush(); },
};

export const stderr = {
  write(data) { return __std_process_stderr_write(data); },
  writeAsync(data) { return __std_process_stderr_writeAsync(data); },
  flush() { return __std_process_stderr_flush(); },
};
