import { __std_process_exec, __std_process_execSync } from "vole:raw:process";

export function exec(command, options) {
  return __std_process_exec(command, options ?? undefined);
}

export function execSync(command, options) {
  return __std_process_execSync(command, options ?? undefined);
}
