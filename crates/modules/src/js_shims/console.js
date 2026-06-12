import {
  __console_log,
  __console_error,
  __console_warn,
  __console_info,
  __console_debug,
} from "std:raw:console";

export function log(...args) {
  __console_log(...args);
}
export function error(...args) {
  __console_error(...args);
}
export function warn(...args) {
  __console_warn(...args);
}
export function info(...args) {
  __console_info(...args);
}
export function debug(...args) {
  __console_debug(...args);
}

export default { log, error, warn, info, debug };
