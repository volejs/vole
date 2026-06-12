import { __std_process_spawn } from "std:raw:process";

export function spawn(command, args, options) {
  return __std_process_spawn(command, args ?? [], options ?? {});
}
