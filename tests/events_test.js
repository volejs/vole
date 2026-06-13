import { test } from "vole:test";

const assert = {
  equal(actual, expected) {
    if (actual !== expected) {
      throw new Error(`expected ${expected}, got ${actual}`);
    }
  },
};

test("EventTarget can add, dispatch and remove listeners", () => {
  const target = new EventTarget();
  let called = 0;
  const listener = () => called++;
  target.addEventListener("foo", listener);
  target.dispatchEvent(new Event("foo"));
  assert.equal(called, 1);
  target.removeEventListener("foo", listener);
  target.dispatchEvent(new Event("foo"));
  assert.equal(called, 1);
});

test("EventTarget once listener only fires once", () => {
  const target = new EventTarget();
  let called = 0;
  target.addEventListener("foo", () => called++, { once: true });
  target.dispatchEvent(new Event("foo"));
  target.dispatchEvent(new Event("foo"));
  assert.equal(called, 1);
});

test("globalThis is an EventTarget", () => {
  assert.equal(typeof globalThis.addEventListener, "function");
  assert.equal(typeof globalThis.removeEventListener, "function");
  assert.equal(typeof globalThis.dispatchEvent, "function");
  assert.equal(globalThis instanceof EventTarget, true);
});

test("globalThis can dispatch and receive events", () => {
  let called = 0;
  const listener = (e) => {
    called++;
    assert.equal(e.type, "hello");
  };
  globalThis.addEventListener("hello", listener);
  globalThis.dispatchEvent(new Event("hello"));
  globalThis.removeEventListener("hello", listener);
  assert.equal(called, 1);
});

test("reportError dispatches error event on globalThis", () => {
  let received;
  const listener = (e) => {
    received = e.error;
    e.preventDefault();
  };
  globalThis.addEventListener("error", listener);
  const err = new Error("reported");
  reportError(err);
  globalThis.removeEventListener("error", listener);
  assert.equal(received, err);
});

test("unhandledrejection event is dispatched", async () => {
  let reason;
  const listener = (e) => {
    reason = e.reason;
    e.preventDefault();
  };
  globalThis.addEventListener("unhandledrejection", listener);
  Promise.reject("rejected-value");
  await new Promise((resolve) => setTimeout(resolve, 50));
  globalThis.removeEventListener("unhandledrejection", listener);
  assert.equal(reason, "rejected-value");
});
