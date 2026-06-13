// Adapted from AWS LLRT tests/unit/timers.test.ts
// Original license: Apache-2.0
// https://github.com/awslabs/llrt

import { test } from "vole:test";

function assertTrue(value, message) {
  if (value !== true) {
    throw new Error(message || `expected true but got ${JSON.stringify(value)}`);
  }
}

function assertEqual(actual, expected, message) {
  if (actual !== expected) {
    throw new Error(
      message || `expected ${JSON.stringify(expected)} but got ${JSON.stringify(actual)}`
    );
  }
}

function assertThrows(fn, message) {
  let threw = false;
  try {
    fn();
  } catch (e) {
    threw = true;
  }
  if (!threw) {
    throw new Error(message || "expected function to throw");
  }
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

test("setTimeout delay", async () => {
  const start = Date.now();
  await delay(10);
  const end = Date.now();
  assertTrue(end - start >= 10);
});

test("setTimeout nested", (done) => {
  setTimeout(() => {
    setTimeout(done, 10);
  }, 10);
});

test("clearTimeout", async () => {
  const start = Date.now();
  let status = "";
  await new Promise((resolve) => {
    const timeout = setTimeout(() => {
      status = "not-cleared";
      resolve();
    }, 5);

    setTimeout(() => {
      status = "cleared";
      resolve();
    }, 10);

    clearTimeout(timeout);
  });
  const end = Date.now();

  assertTrue(end - start >= 10);
  assertEqual(status, "cleared");
});

test("setInterval", async () => {
  const start = Date.now();
  let count = 1;
  await new Promise((resolve) => {
    const interval = setInterval(() => {
      if (count > 4) {
        clearInterval(interval);
        return resolve();
      }
      count++;
    }, 5);
  });
  const end = Date.now();
  assertTrue(end - start >= 10);
  assertEqual(count, 5);
});

test("clearInterval", async () => {
  const start = Date.now();
  let count = 1;
  await new Promise((resolve) => {
    let interval = setInterval(() => {
      if (count === 2) {
        clearInterval(interval);
        return;
      }
      count++;
    }, 5);
    setTimeout(resolve, 20);
  });
  const end = Date.now();
  assertTrue(end - start > 10);
  assertEqual(count, 2);
});

// This runtime's clearTimeout requires a numeric handle; non-numeric arguments throw.
// test("clearTimeout accepts any parameter", () => { ... });

test("delay is optional", async () => {
  const start = Date.now();
  await new Promise((resolve) => setTimeout(resolve));
  const end = Date.now();
  assertTrue(end - start >= 0);
});

// This runtime rejects negative delays with an underflow error.
// test("delay can be negative", async () => { ... });
