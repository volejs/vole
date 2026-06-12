import {
  __std_tracing_info,
  __std_tracing_warn,
  __std_tracing_error,
  __std_tracing_debug,
  __std_tracing_trace,
} from "std:raw:tracing";

function joinArgs(args) {
  return args.map((a) => {
    if (typeof a === "object") {
      try { return JSON.stringify(a); } catch { return String(a); }
    }
    return String(a);
  }).join(" ");
}

export function info(...args) {
  __std_tracing_info(joinArgs(args));
}

export function warn(...args) {
  __std_tracing_warn(joinArgs(args));
}

export function error(...args) {
  __std_tracing_error(joinArgs(args));
}

export function debug(...args) {
  __std_tracing_debug(joinArgs(args));
}

export function trace(...args) {
  __std_tracing_trace(joinArgs(args));
}

export default { info, warn, error, debug, trace };
