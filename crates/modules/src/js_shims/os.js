import {
  __std_os_exit,
  __std_os_abort,
  __std_os_cwd,
  __std_os_chdir,
  __std_os_uptime,
  __std_os_memoryUsage,
  __std_os_args,
  __std_os_argv0,
  __std_os_pid,
  __std_os_ppid,
  __std_os_platform,
  __std_os_arch,
  __std_os_version,
  __std_os_versions,
  __std_os_getExitCode,
  __std_os_setExitCode,
  __std_os_stdout_write,
  __std_os_stdout_writeAsync,
  __std_os_stdout_flush,
  __std_os_stderr_write,
  __std_os_stderr_writeAsync,
  __std_os_stderr_flush,
} from "vole:raw:os";

export function exit(code) {
  return __std_os_exit(code);
}

export function abort() {
  return __std_os_abort();
}

export function cwd() {
  return __std_os_cwd();
}

export function chdir(path) {
  return __std_os_chdir(path);
}

export function uptime() {
  return __std_os_uptime();
}

export function getExitCode() {
  return __std_os_getExitCode();
}

export function setExitCode(code) {
  return __std_os_setExitCode(code);
}

export function memoryUsage() {
  return __std_os_memoryUsage();
}

export const args = __std_os_args();
export const argv0 = __std_os_argv0();
export const pid = __std_os_pid();
export const ppid = __std_os_ppid();
export const platform = __std_os_platform();
export const arch = __std_os_arch();
export const version = __std_os_version();
export const versions = __std_os_versions();

export const stdout = {
  write(data) {
    return __std_os_stdout_write(data);
  },
  writeAsync(data) {
    return __std_os_stdout_writeAsync(data);
  },
  flush() {
    return __std_os_stdout_flush();
  },
};

export const stderr = {
  write(data) {
    return __std_os_stderr_write(data);
  },
  writeAsync(data) {
    return __std_os_stderr_writeAsync(data);
  },
  flush() {
    return __std_os_stderr_flush();
  },
};

const os = {
  exit,
  abort,
  cwd,
  chdir,
  uptime,
  getExitCode,
  setExitCode,
  memoryUsage,
  args,
  argv0,
  pid,
  ppid,
  platform,
  arch,
  version,
  versions,
  stdout,
  stderr,
};

export default os;
