import {
  __until_begin,
  __until_close,
  __until_create,
  __until_drain,
  __until_finish,
  __until_is_closed,
  __until_is_idle,
  __until_is_managed,
  __until_pending_count,
  __until_registry_close,
  __until_registry_drain,
  __until_registry_is_idle,
  __until_registry_pending_count,
  __until_registry_scope_count,
} from "vole:raw:until";

const kHandle = Symbol("WaitUntil.handle");

function handleOf(target) {
  if (!(target instanceof WaitUntil)) {
    throw new TypeError("Expected a WaitUntil instance");
  }
  return target[kHandle];
}

function reportWaitUntilError(error) {
  if (typeof globalThis.reportError === "function") {
    globalThis.reportError(error);
    return;
  }

  queueMicrotask(() => {
    throw error;
  });
}

function managedFromOptions(options) {
  if (options === undefined) {
    return true;
  }
  if (options === null || typeof options !== "object") {
    throw new TypeError("WaitUntil options must be an object");
  }
  return options.managed !== false;
}

export class WaitUntil {
  constructor(options) {
    Object.defineProperty(this, kHandle, {
      value: __until_create(managedFromOptions(options)),
      enumerable: false,
      configurable: false,
      writable: false,
    });
  }

  waitUntil(promise) {
    waitUntil(this, promise);
  }

  drain(timeout) {
    return drain(this, timeout);
  }

  close() {
    close(this);
  }

  get pendingCount() {
    return pendingCount(this);
  }

  get isIdle() {
    return isIdle(this);
  }

  get isClosed() {
    return isClosed(this);
  }

  get managed() {
    return isManaged(this);
  }
}

export function waitUntil(target, promise) {
  const guard = __until_begin(handleOf(target));
  Promise.resolve(promise)
    .catch(reportWaitUntilError)
    .finally(() => {
      __until_finish(guard);
    });
}

export function drain(target, timeout) {
  return __until_drain(handleOf(target), timeout);
}

export function close(target) {
  __until_close(handleOf(target));
}

export function pendingCount(target) {
  return __until_pending_count(handleOf(target));
}

export function isIdle(target) {
  return __until_is_idle(handleOf(target));
}

export function isClosed(target) {
  return __until_is_closed(handleOf(target));
}

export function isManaged(target) {
  return __until_is_managed(handleOf(target));
}

export function drainManaged(timeout) {
  return __until_registry_drain(timeout);
}

export function closeManaged() {
  __until_registry_close();
}

export function managedPendingCount() {
  return __until_registry_pending_count();
}

export function managedScopeCount() {
  return __until_registry_scope_count();
}

export function managedIsIdle() {
  return __until_registry_is_idle();
}

export default {
  WaitUntil,
  waitUntil,
  drain,
  close,
  pendingCount,
  isIdle,
  isClosed,
  isManaged,
  drainManaged,
  closeManaged,
  managedPendingCount,
  managedScopeCount,
  managedIsIdle,
};
