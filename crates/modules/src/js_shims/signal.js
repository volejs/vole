import {
  __std_signal_is_supported,
  __std_signal_list,
  __std_signal_send,
  SignalStream,
} from "vole:raw:signal";

function assertPid(value, argName) {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new TypeError(`${argName} requires a numeric pid`);
  }
}

export function sendSignal(pid, signal = "SIGTERM") {
  assertPid(pid, "signal.sendSignal");
  return __std_signal_send(pid, String(signal));
}

export const killProcess = sendSignal;

export function listSignals() {
  return __std_signal_list();
}

export function isSupported(signalName) {
  return __std_signal_is_supported(String(signalName));
}

export function stream(signals) {
  return new SignalStream(signals);
}

export { SignalStream };

export const signal = {
  sendSignal,
  killProcess,
  listSignals,
  isSupported,
  stream,
  SignalStream,
};

export default signal;
