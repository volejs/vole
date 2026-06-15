import { __std_os_exec, __std_os_execSync, __std_os_spawn } from "vole:raw:os/exec";

export function exec(command, options) {
  return __std_os_exec(command, options ?? undefined);
}

export function execSync(command, options) {
  return __std_os_execSync(command, options ?? undefined);
}

export function spawn(command, args, options) {
  return __std_os_spawn(command, args ?? [], options ?? {});
}

export default {
  exec,
  execSync,
  spawn,
};
