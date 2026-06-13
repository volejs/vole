import { test } from "vole:test";

const assert = {
  equal(actual, expected) {
    if (actual !== expected) {
      throw new Error(`expected ${expected}, got ${actual}`);
    }
  },
};

test("DOMException is a native class extending Error", () => {
  assert.equal(typeof DOMException, "function");
  const err = new DOMException("network failed", "NetworkError");
  assert.equal(err instanceof Error, true);
  assert.equal(err instanceof DOMException, true);
});

test("DOMException properties", () => {
  const err = new DOMException("timeout", "TimeoutError");
  assert.equal(err.message, "timeout");
  assert.equal(err.name, "TimeoutError");
  assert.equal(err.code, 23);
});

test("DOMException defaults", () => {
  const err = new DOMException();
  assert.equal(err.message, "");
  assert.equal(err.name, "Error");
  assert.equal(err.code, 0);
});

test("DOMException code mapping", () => {
  assert.equal(new DOMException("", "NotFoundError").code, 8);
  assert.equal(new DOMException("", "AbortError").code, 20);
  assert.equal(new DOMException("", "UnknownError").code, 0);
});
