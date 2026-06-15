import { test as registerTest } from "vole:test";

let currentFile = "<unknown>";
let expectedFailures = new Set();

globalThis.self = globalThis;
globalThis.GLOBAL = globalThis.GLOBAL || {
  isWindow: () => false,
  isWorker: () => true,
  isServiceWorker: () => false,
  isShadowRealm: () => false,
};
self.GLOBAL = globalThis.GLOBAL;

export function setWptCurrentFile(file) {
  currentFile = file;
}

export function setWptExpectedFailures(entries) {
  expectedFailures = new Set(entries);
}

function fullTestName(name) {
  return `${currentFile}: ${name || "<unnamed>"}`;
}

function isExpectedFailure(name) {
  return expectedFailures.has(name) || expectedFailures.has(`${currentFile}: *`);
}

async function runWptTest(fn, providedContext) {
  const t = providedContext || makeTestContext();
  try {
    await fn(t);
  } finally {
    await t.cleanup();
  }
}

function registerWptTest(name, fn, providedContext) {
  const fullName = fullTestName(name);
  if (isExpectedFailure(fullName)) {
    registerTest(`${fullName} [expected failure]`, async () => {
      try {
        await runWptTest(fn, providedContext);
      } catch (_) {
        return;
      }
      throw new Error("Expected WPT failure passed");
    });
    return;
  }
  registerTest(fullName, async () => {
    await runWptTest(fn, providedContext);
  });
}

function makeTestContext(resolve, reject) {
  const cleanups = [];
  return {
    add_cleanup(fn) {
      cleanups.push(fn);
    },
    async cleanup() {
      while (cleanups.length > 0) {
        await cleanups.pop()();
      }
    },
    step(fn) {
      return fn();
    },
    step_func(fn) {
      return (...args) => {
        try {
          return fn(...args);
        } catch (err) {
          if (reject) reject(err);
          throw err;
        }
      };
    },
    step_func_done(fn = () => {}) {
      return (...args) => {
        try {
          const value = fn(...args);
          if (resolve) resolve(value);
          return value;
        } catch (err) {
          if (reject) reject(err);
          throw err;
        }
      };
    },
    unreached_func(message = "Reached unreachable code") {
      return () => {
        const err = new Error(message);
        if (reject) reject(err);
        throw err;
      };
    },
    done() {
      if (resolve) resolve();
    },
  };
}

globalThis.setup = function setup(fn) {
  if (typeof fn === "function") {
    fn();
  }
};
globalThis.done = function done() {};

globalThis.test = function test(fn, name) {
  registerWptTest(name, (t) => fn.call(t, t));
};

globalThis.promise_test = function promise_test(fn, name) {
  registerWptTest(name, async (t) => fn.call(t, t));
};

globalThis.async_test = function async_test(fn, name) {
  let resolve;
  let reject;
  const promise = new Promise((res, rej) => {
    resolve = res;
    reject = rej;
  });
  const t = makeTestContext(resolve, reject);
  registerWptTest(name || "async_test", async () => promise, t);
  if (typeof fn === "function") {
    t.step(() => fn.call(t, t));
  }
  return t;
};

function formatMessage(message) {
  return message ? `: ${message}` : "";
}

function fail(message) {
  throw new Error(message);
}

globalThis.assert_true = function assert_true(actual, message) {
  if (actual !== true) fail(`assert_true failed${formatMessage(message)}; got ${actual}`);
};

globalThis.assert_false = function assert_false(actual, message) {
  if (actual !== false) fail(`assert_false failed${formatMessage(message)}; got ${actual}`);
};

globalThis.assert_equals = function assert_equals(actual, expected, message) {
  if (!Object.is(actual, expected)) {
    fail(`assert_equals failed${formatMessage(message)}; expected ${String(expected)}, got ${String(actual)}`);
  }
};

globalThis.assert_not_equals = function assert_not_equals(actual, expected, message) {
  if (Object.is(actual, expected)) {
    fail(`assert_not_equals failed${formatMessage(message)}; both were ${String(actual)}`);
  }
};

globalThis.assert_array_equals = function assert_array_equals(actual, expected, message) {
  if (!actual || actual.length !== expected.length) {
    fail(`assert_array_equals length failed${formatMessage(message)}`);
  }
  for (let i = 0; i < expected.length; i++) {
    if (!Object.is(actual[i], expected[i])) {
      fail(`assert_array_equals failed at index ${i}${formatMessage(message)}; expected ${String(expected[i])}, got ${String(actual[i])}`);
    }
  }
};

globalThis.assert_nested_array_equals = globalThis.assert_array_equals;

globalThis.assert_throws_js = function assert_throws_js(errorConstructor, fn, message) {
  try {
    fn();
  } catch (err) {
    if (err instanceof errorConstructor) return;
    fail(`assert_throws_js failed${formatMessage(message)}; got ${err && err.constructor && err.constructor.name}`);
  }
  fail(`assert_throws_js failed${formatMessage(message)}; function did not throw`);
};

globalThis.assert_throws_dom = function assert_throws_dom(name, fn, message) {
  try {
    fn();
  } catch (err) {
    if (err && (err.name === name || err.constructor?.name === name)) return;
    fail(`assert_throws_dom failed${formatMessage(message)}; got ${err && err.name}`);
  }
  fail(`assert_throws_dom failed${formatMessage(message)}; function did not throw`);
};

globalThis.assert_unreached = function assert_unreached(message = "Reached unreachable code") {
  fail(message);
};

globalThis.promise_rejects_js = async function promise_rejects_js(_t, errorConstructor, promise, message) {
  try {
    await promise;
  } catch (err) {
    if (err instanceof errorConstructor) return;
    fail(`promise_rejects_js failed${formatMessage(message)}; got ${err && err.constructor && err.constructor.name}`);
  }
  fail(`promise_rejects_js failed${formatMessage(message)}; promise resolved`);
};
